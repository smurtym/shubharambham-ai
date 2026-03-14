# WASM API Contract: wasm-api-v2

**Version**: 2.0.0  
**Status**: Active  
**Supersedes**: `wasm-api-v1` (retired — no backward compatibility)  
**Feature**: `003-arch-layout`  
**Date**: 2026-03-14

---

## Bridge Function Signature

The single exported WASM function remains unchanged from v1:

```c
int bridge(
    const char* op_ptr,        // operation name, null-terminated UTF-8
    const char* input_ptr,     // JSON request payload, null-terminated UTF-8
    char*       output_ptr,    // caller-allocated output buffer
    int         output_max_len // size of output buffer in bytes
);
```

### Return Values

| Return value | Meaning |
|---|---|
| `> 0` and `<= output_max_len` | Success — bytes written to `output_ptr` |
| `> output_max_len` | Buffer too small — value is the required size; caller MUST retry with a larger buffer (Option B retry pattern) |
| `-1` | Unknown operation name |
| `-2` | JSON parse error or invalid input field |
| `-3` | Calculation error (SWE returned error) |

### Buffer Management

The Option B retry pattern is mandatory for all callers:
1. Allocate `output_ptr` with an initial size (e.g., 4096 bytes).
2. Call `bridge()`.
3. If return value > `output_max_len`, free and re-allocate at the returned size, then call `bridge()` once more.
4. Free all three buffers (`op_ptr`, `input_ptr`, `output_ptr`) in a `finally` block regardless of outcome.

### Dispatch Rules

`op_ptr` is the **authoritative** operation name used for dispatch inside `bridge()`. The `"operation"` field in the JSON body MUST match `op_ptr`. If they differ, `bridge()` returns `-2` (parse error) and writes an `ErrorResponse { "error": "operation mismatch" }` to the output buffer before returning.

On **all** negative return codes (-1, -2, -3), `bridge()` MUST write a best-effort `ErrorResponse { "error": "<human-readable message>" }` JSON to the output buffer before returning, so the JS caller can display a meaningful error without inspecting return codes directly. The buffer write follows the same Option B retry pattern.

---

## Operations

### `sun_longitude`

Compute the ecliptic longitude of the Sun for a given UTC datetime and return a localized label with the result.

#### Request

JSON payload passed as `input_ptr`:

```json
{
  "operation": "sun_longitude",
  "datetime": "2000-01-01T12:00:00Z",
  "lang": "en"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `operation` | string | yes | Must be `"sun_longitude"` |
| `datetime` | string | yes | ISO 8601 UTC datetime (e.g., `"2000-01-01T12:00:00Z"`). Always UTC — timezone offsets not yet supported. |
| `lang` | string | yes | Locale identifier. Accepted: `"en"`, `"te"`. Unknown values fall back silently to `"en"`. |

#### Response — Success

```json
{
  "label": "Sun",
  "longitude": 280.46
}
```

| Field | Type | Description |
|-------|------|-------------|
| `label` | string | Localized planet name. `"Sun"` for `en`; `"సూర్యుడు"` for `te`. |
| `longitude` | number | Ecliptic longitude in decimal degrees (0–360). |

#### Response — Error

```json
{
  "error": "invalid datetime: missing T separator"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `error` | string | Human-readable error description. `label` and `longitude` are absent. |

#### Validation

- `datetime` must be parseable as an ISO 8601 UTC string. Malformed input returns an error response (bridge return code `-2`).
- `lang` unknown values silently fall back to `"en"` — no error.

#### Reference Values

| Input datetime | Expected longitude | Tolerance |
|---|---|---|
| `2000-01-01T12:00:00Z` (J2000.0) | ≈ 280.46° | ± 0.01° |

#### Coordinate System & Ayanamsha

`sun_longitude` returns the **tropical ecliptic longitude** of the Sun. No ayanamsha is applied. The internal `swe_calc_ut` call uses flag `SEFLG_SPEED` (tropical coordinates; the `SEFLG_SIDEREAL` flag is explicitly **not** set). Ayanamsha offset for Vedic sidereal longitude is out of scope for this feature and will be addressed in a future contract revision with a dedicated `ayanamsha` field in the request.

---

## Localization Strings (v2)

| Key | `en` | `te` |
|-----|------|------|
| `planet.sun` | `Sun` | `సూర్యుడు` |

---

## Migration from wasm-api-v1

`wasm-api-v1` sent `{ "tjd": <number> }` as input and received `{ "longitude": <number> }` as output. That contract is **retired**. Callers must be updated to the v2 request shape — no shim or fallback is provided.

| Change | v1 | v2 |
|--------|----|----|
| Input field | `tjd` (Julian Day number) | `datetime` (ISO 8601 string) + `lang` |
| Output field | `longitude` only | `label` + `longitude` (or `error`) |
| Error shape | Bridge return code only | `{ "error": string }` in JSON output |
| Localization | None | `lang` in request; `label` in response |
