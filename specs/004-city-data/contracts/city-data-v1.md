# WASM API Contract: city-data-v1

**Version**: 1.0.0  
**Status**: Active  
**Bridge**: wasm-api-v2 (bridge function signature unchanged — see [wasm-api-v2](../../003-arch-layout/contracts/wasm-api-v2.md))  
**Feature**: `004-city-data`  
**Date**: 2026-03-15

---

## Overview

This contract defines the `list_cities` operation added to the existing wasm-api-v2 bridge. The bridge function signature (`bridge(op_ptr, input_ptr, output_ptr, output_max_len)`) and buffer management rules are unchanged and inherited from wasm-api-v2. Only a new dispatch arm is added.

This is an **additive** change to the bridge — wasm-api-v2 MAJOR version does not change. The new operation is documented here as a standalone contract named `city-data-v1`.

---

## Bridge Function Signature

Inherited from wasm-api-v2 (no changes):

```c
int bridge(
    const char* op_ptr,        // "list_cities", null-terminated UTF-8
    const char* input_ptr,     // JSON request payload, null-terminated UTF-8
    char*       output_ptr,    // caller-allocated output buffer
    int         output_max_len // size of output buffer in bytes
);
```

Return values and Option B retry pattern are identical to wasm-api-v2. See [wasm-api-v2](../../003-arch-layout/contracts/wasm-api-v2.md) for full buffer management rules.

---

## Operation: `list_cities`

Returns a language-filtered, sort-ordered list of cities compiled into the WASM binary.

### Request

`op_ptr` = `"list_cities"` (null-terminated)

`input_ptr` JSON payload:

```json
{
  "operation": "list_cities",
  "lang": "te"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `operation` | string | yes | MUST equal `"list_cities"` (must match `op_ptr` per dispatch rules) |
| `lang` | string | yes | IETF-style language tag, e.g. `"en"` or `"te"` |

### Response — Success

```json
{
  "cities": [
    {
      "lang": "te",
      "cityId": 11705,
      "timeZone": "Asia/Kolkata",
      "canonicalName": "Hyderabad",
      "cityName": "హైదరాబాద్",
      "region1": "తెలంగాణ",
      "region2": "భారతదేశం",
      "lat": 17.97,
      "lng": 77.34
    },
    {
      "lang": "te",
      "cityId": 11834,
      "timeZone": "Asia/Kolkata",
      "canonicalName": "Vijayawada",
      "cityName": "విజయవాడ",
      "region1": "ఆంధ్రప్రదేశ్",
      "region2": "భారతదేశం",
      "lat": 15.28,
      "lng": 80.16
    }
  ]
}
```

**Notes**:
- `cities` may be an empty array `[]` — valid success response when no cities have a translation for the requested language.
- Cities without a translation entry for the requested `lang` are excluded.
- Array is sorted by `region2Order` ascending → `region1Order` ascending → `cityName` ascending (all from the requested language's translation). Sort keys are internal and never present in the response.
- `canonicalName` is always in English regardless of `lang`.
- `lang` echoes the value from the request.

| Field | Type | Description |
|-------|------|-------------|
| `lang` | string | Echoed language code from request |
| `cityId` | integer | 16-bit quadkey at zoom level 7 (`tile_x × 128 + tile_y`, Web Mercator) — unique city identifier |
| `timeZone` | string | IANA timezone identifier |
| `canonicalName` | string | City name in English (language-invariant) |
| `cityName` | string | City name in requested language |
| `region1` | string | State/province in requested language |
| `region2` | string | Country in requested language |
| `lat` | number | Tile-centre latitude in decimal degrees, decoded from `cityId` in Rust |
| `lng` | number | Tile-centre longitude in decimal degrees, decoded from `cityId` in Rust |

### Response — Error

```json
{
  "error": "<human-readable message>"
}
```

Returned when:
- Request JSON is malformed (parse error)
- `operation` field in JSON does not match `op_ptr` (operation mismatch)
- `lang` field is missing from the request

An **unknown but syntactically valid** language code (e.g., `"hi"`) is **not** an error — it returns `{"cities": []}`.

### Return Codes

| Return value | Meaning |
|---|---|
| `> 0` and `<= output_max_len` | Bytes written; parse response JSON |
| `> output_max_len` | Buffer too small; retry with this size (Option B) |
| `-2` | JSON parse error, missing `lang` field, or `operation` mismatch |

> Note: `-3` (calculation error) is not applicable for this operation — no Swiss Ephemeris calls are made.

---

## Localization Keys

No localization keys are returned. All translated strings (`cityName`, `region1`, `region2`) are data — they come directly from the hardcoded translation entries in the Rust data store and are returned verbatim.

---

## JS Caller Example

```javascript
// web/data.js — follows the getSunLongitude pattern (synchronous)
function listCities(lang) {
  var response = bridge('list_cities', JSON.stringify({ operation: 'list_cities', lang: lang }));
  if (response.error) throw new Error(response.error);
  return response.cities;
}
```

---

## Versioning

| Version | Date | Change |
|---------|------|--------|
| 1.0.0 | 2026-03-15 | Initial definition |
