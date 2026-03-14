# WASM API Contract: v1.0.0

**Feature**: 001-infrastructure-tooling-setup  
**Version**: 1.0.0  
**Status**: Draft  
**Module file**: `dist/astro.js` (Emscripten JS glue) + `dist/astro.wasm`  
**Contract governance**: Per Constitution Principle III — breaking changes increment MAJOR version and require a migration plan.

---

## Overview

This contract defines the sole exported function in the initial WASM module. Its purpose is
to verify the full Rust → Emscripten → swisseph pipeline. It is not a production API; it
will be superseded by the horoscope calculation contracts in subsequent features.

---

## Module Initialization

The Emscripten runtime must be fully initialized before any exported function may be called.
The JS caller MUST use the `onRuntimeInitialized` callback pattern:

```js
// MUST appear before <script src="astro.js">
var Module = {
  onRuntimeInitialized: function () {
    // safe to call bridge(...) here
  }
};
```

**Error behaviour**: If the WASM module fails to load (e.g., WASM unsupported, file missing),
`onRuntimeInitialized` will not fire. The JS glue emits a console error in this case; the
caller MUST NOT assume the module is available outside this callback.

---

## Bridge Design Principles

- **JS owns all memory.** Input and output buffers are allocated by JS via `Module._malloc` and freed by JS via `Module._free`, always inside a `try/finally` block. Rust reads inputs and writes outputs. Rust never allocates heap memory visible to JS.
- **Single permanent export.** `-sEXPORTED_FUNCTIONS=_bridge` is set once in `.cargo/config.toml` and never changed. Adding a new operation requires only a new Rust handler — no changes to `build.sh`, linker flags, or JS.
- **Option B retry protocol.** JS starts with a 4 KB output buffer. If `bridge` returns a value greater than the buffer size, JS reallocates to that exact size and retries. This keeps typical memory usage minimal for users on low-RAM devices.

## Exported Functions

### `_bridge` — sole permanent WASM export

The single entry point for all operations. Routes `op` to the appropriate Rust handler,
passing JSON input and writing JSON output into a caller-owned buffer.

#### Signature

```
Module._bridge(op_ptr, input_ptr, output_ptr, output_max_len) → i32
```

| Parameter | Type | Null-terminated? | Owned by |
|-----------|------|-----------------|----------|
| `op_ptr` | `*const c_char` | Yes | JS — allocated and freed by JS |
| `input_ptr` | `*const c_char` | Yes | JS — allocated and freed by JS |
| `output_ptr` | `*mut c_char` | Written by Rust | JS — allocated and freed by JS |
| `output_max_len` | `i32` | — | JS passes the allocated size |

#### Return value

| Return value | Meaning |
|---|---|
| `> 0` and `≤ output_max_len` | Success — bytes written to `output_ptr` |
| `> output_max_len` | Buffer too small — value is the required size; JS MUST retry with this exact size |
| `-1` | Unknown operation |
| `-2` | JSON input parse error |
| `-3` | Calculation error (e.g., ephemeris files not found) |

#### Rust source declaration

```rust
// astro-wasm/src/lib.rs
#[no_mangle]
pub extern "C" fn bridge(
    op_ptr: *const std::os::raw::c_char,
    input_ptr: *const std::os::raw::c_char,
    output_ptr: *mut std::os::raw::c_char,
    output_max_len: i32,
) -> i32 { ... }
```

#### Linker export flag (set once, permanent)

```
-sEXPORTED_FUNCTIONS=_bridge
```

#### JS caller boilerplate (written once, never changes)

```js
const INITIAL_OUTPUT_SIZE = 4096; // 4 KB — covers all expected results on first try

function bridge(op, inputJson) {
  const enc = new TextEncoder();
  const opBytes    = enc.encode(op + '\0');
  const inputBytes = enc.encode(inputJson + '\0');

  const opPtr  = Module._malloc(opBytes.length);
  const inPtr  = Module._malloc(inputBytes.length);
  Module.HEAPU8.set(opBytes,    opPtr);
  Module.HEAPU8.set(inputBytes, inPtr);

  let outPtr  = Module._malloc(INITIAL_OUTPUT_SIZE);
  let outSize = INITIAL_OUTPUT_SIZE;

  try {
    let written = Module._bridge(opPtr, inPtr, outPtr, outSize);

    if (written > outSize) {
      // Buffer too small — Rust returned required size; retry once with exact size
      Module._free(outPtr);
      outPtr  = Module._malloc(written);
      outSize = written;
      written = Module._bridge(opPtr, inPtr, outPtr, outSize);
    }

    if (written < 0) throw new Error('bridge error code: ' + written);

    const bytes = Module.HEAPU8.subarray(outPtr, outPtr + written);
    return JSON.parse(new TextDecoder().decode(bytes));
  } finally {
    Module._free(opPtr);  // JS always frees its own allocations
    Module._free(inPtr);
    Module._free(outPtr);
  }
}
```

---

## Registered Operations

Each operation is a string key routed inside Rust. Operations are documented here as the
catalogue grows. Adding a new operation is a **MINOR** version change to this contract.

### `sun_longitude`

Calculates the Sun's apparent ecliptic longitude (tropical, no ayanamsha) for a given
Julian Day Number in UTC, using Swiss Ephemeris file-based data (`swe_calc_ut`).

#### Input payload

```json
{ "tjd": 2451545.0 }
```

| Field | Type | Description |
|-------|------|-------------|
| `tjd` | number | Julian Day Number in UTC (Universal Time) |

#### Output payload (success)

```json
{ "longitude": 280.459234 }
```

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `longitude` | number | 0.0 – 360.0 | Ecliptic longitude in decimal degrees (tropical) |

#### Output payload (error)

```json
{ "error": "ephemeris files not found at /ephe/" }
```

#### Expected reference value

For J2000.0 (`tjd = 2451545.0`): **280.459°** (±0.001°). Tropical, no ayanamsha applied.

---

## Runtime Prerequisites

| Requirement | Detail |
|-------------|--------|
| WASM support | Browser must support WebAssembly (Chrome 57+, Firefox 53+, Safari 11+) |
| Ephemeris data | `semo_18.se1` and `sepl_18.se1` must be preloaded at virtual path `/ephe/` |
| `swe_set_ephe_path` | Called with `"/ephe"` inside Rust before `swe_calc` to ensure file-based (not Moshier) ephemeris |

---

## Versioning Policy

| Change type | Version bump | Action required |
|-------------|-------------|-----------------|
| Add new exported function | MINOR | Update this file; add function section |
| Change parameter types or count | MAJOR | New contract version; migration plan required |
| Change return type | MAJOR | New contract version; migration plan required |
| Change error return value (−1.0) | MAJOR | New contract version |
| Rename function | MAJOR | New contract version |
| Internal implementation change (same signature) | PATCH | No contract update required |

**Version**: 1.0.0 | **Created**: 2026-03-14 | **Last amended**: 2026-03-14
