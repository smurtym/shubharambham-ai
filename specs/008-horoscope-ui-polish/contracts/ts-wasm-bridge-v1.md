# TypeScript WASM Bridge Contract v1

**Version**: 1.0.0  
**Layer**: TypeScript glue (`web/horoscope/astro-glue.ts`)  
**Underlying WASM API**: `horoscope-api-v1.md` (unchanged — no new WASM operations)  
**Feature**: `008-horoscope-ui-polish`  
**Status**: Active

---

## Overview

This contract documents the **TypeScript-typed wrapper layer** around the existing Emscripten WASM bridge. The WASM binary (`public/astro.js`) and the binary protocol are unchanged; this contract describes only the TypeScript interface that React components consume.

The underlying C bridge signature (from `horoscope-api-v1.md`) is:
```c
int bridge(const char* op, const char* input_json,
           char* output_buf, int output_max_len);
```

---

## Emscripten Module Contract

### Global Declaration

`window.Module` must be set as an inline `<script>` in `web/horoscope/index.html` BEFORE the `<script src="/astro.js">` tag:

```html
<script>
  window.Module = {
    onRuntimeInitialized: function () {
      document.dispatchEvent(new Event('wasm-ready'));
    }
  };
</script>
<script src="/astro.js"></script>
```

### TypeScript Ambient Type (`emscripten.d.ts`)
```typescript
declare global {
  interface Window {
    Module: {
      _malloc: (size: number) => number;
      _bridge: (
        opPtr: number,
        inPtr: number,
        outPtr: number,
        outSize: number
      ) => number;
      _free: (ptr: number) => void;
      HEAPU8: Uint8Array;
      onRuntimeInitialized?: () => void;
    };
  }
}
export {};
```

### `wasm-ready` Event Contract

| Property | Value |
|---|---|
| Event name | `'wasm-ready'` |
| Target | `document` |
| Dispatched by | `window.Module.onRuntimeInitialized` callback |
| Timing | Fires once after WASM heap and exports are fully initialised |
| Listener | `App.tsx` via `document.addEventListener('wasm-ready', ...)` |

---

## `bridge()` — Low-level Function

```typescript
function bridge(op: string, input: object): object
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `op` | `string` | Operation name: `"list_cities"` or `"horoscope_positions"` |
| `input` | `object` | Request object (serialised to JSON internally) |

### Returns
Parsed JSON response object on success.

### Throws `WasmError`
```typescript
interface WasmError {
  code: number;    // -1 | -2 | -3 (see table below)
  message: string; // human-readable error string from WASM or bridge error
}
```

| Code | Meaning |
|------|---------|
| `-1` | Unknown operation name |
| `-2` | JSON parse error or field mismatch |
| `-3` | Calculation error (Swiss Ephemeris failure) |

### Buffer Sizes

| Operation | Output buffer size |
|-----------|-------------------|
| `list_cities` | `524288` bytes (512 KiB) |
| `horoscope_positions` | `65536` bytes (64 KiB) |

### Implementation Notes
- Allocate input string with `Module._malloc`; write bytes with `Module.HEAPU8.set`
- Allocate output buffer with `Module._malloc(bufferSize)`
- Call `Module._bridge(opPtr, inPtr, outPtr, bufferSize)`
- Return value `> 0` = bytes written; parse with `TextDecoder`
- Free both pointers with `Module._free` in a `finally` block
- Return value `> bufferSize`: buffer too small (should not occur with declared sizes)

---

## `listCities()` — High-level Function

```typescript
async function listCities(lang: Lang): Promise<{ cities: CityRecord[] }>
```

### Parameters

| Parameter | Type | Constraints |
|-----------|------|-------------|
| `lang` | `Lang` (`'en' \| 'te'`) | Passed as `"lang"` in WASM request |

### WASM Request (sent internally)
```json
{
  "operation": "list_cities",
  "lang": "en"
}
```

### Returns
```typescript
{ cities: CityRecord[] }
// CityRecord fields: see data-model.md
```

### Throws
`WasmError` on any negative bridge return code.

---

## `getHoroscopePositions()` — High-level Function

```typescript
async function getHoroscopePositions(
  cityId: number,
  localTime: string,
  lang: Lang
): Promise<HoroscopeResponse>
```

### Parameters

| Parameter | Type | Constraints |
|-----------|------|-------------|
| `cityId` | `number` (u32) | Must exist in compiled city store |
| `localTime` | `string` | Format: `YYYY-MM-DDTHH:MM:SS` (no timezone suffix) |
| `lang` | `Lang` | `'en' \| 'te'` |

### WASM Request (sent internally)
```json
{
  "operation": "horoscope_positions",
  "cityId": 466083476,
  "localTime": "2026-03-20T15:05:00",
  "lang": "te"
}
```

### Returns
`HoroscopeResponse` — see `data-model.md` for full shape.

### Throws
`WasmError` on any negative bridge return code, including city-not-found (`-2`).

---

## Versioning

This contract versions the **TypeScript glue interface**, not the WASM binary.

| Change | Version bump |
|--------|-------------|
| New exported function or parameter | MINOR |
| Rename, remove, or type change | MAJOR |
| Comment or documentation fix | PATCH |

Breaking changes to the underlying WASM API are governed by `horoscope-api-v1.md` independently.
