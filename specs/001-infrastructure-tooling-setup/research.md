# Research: Infrastructure and Tooling Setup

**Feature**: `001-infrastructure-tooling-setup`
**Date**: 2026-03-14
**Stack**: Rust 1.88, Emscripten 4.0.13, Swiss Ephemeris (aloistr/swisseph), target `wasm32-unknown-emscripten`, vanilla JS, no wasm-bindgen.

---

## Q1 — Rust + wasm32-unknown-emscripten: Linking against a C static library via Emscripten

### Decision

Use `emcc` as the linker for the `wasm32-unknown-emscripten` target by declaring it in `.cargo/config.toml`. Pass the pre-built `.a` from a `build.rs` script using `cargo:rustc-link-lib` and `cargo:rustc-link-search` directives.

### Rationale

Rustc does not know how to finalise a `wasm32-unknown-emscripten` binary on its own; it must delegate the final link step to `emcc`. Setting the linker in `.cargo/config.toml` makes this transparent to `cargo build`. The build script is the canonical way to inject library search paths and `-l` flags without reaching into `RUSTFLAGS`, keeping the build repeatable.

### Key configuration

**`.cargo/config.toml`**
```toml
[target.wasm32-unknown-emscripten]
linker = "emcc"
# Pass emcc linker flags through rustc's -C link-arg
rustflags = [
  "-C", "link-arg=-sENVIRONMENT=web",
  "-C", "link-arg=-sEXPORTED_FUNCTIONS=_bridge",   # single generic bridge fn — never changes
  "-C", "link-arg=--preload-file", "-C", "link-arg=ephe/@/ephe/",
  "-C", "link-arg=-sALLOW_MEMORY_GROWTH=1",
  # NOTE: no EXPORTED_RUNTIME_METHODS needed — we use _malloc/_free directly
]
```

**`build.rs`** (lives at the Rust crate root)
```rust
fn main() {
    println!("cargo:rustc-link-search=native=../lib");  // dir containing libswe.a
    println!("cargo:rustc-link-lib=static=swe");        // links libswe.a
}
```

**Build command**
```sh
cargo build --target wasm32-unknown-emscripten --release
```

> **Note:** Do **not** set `RUSTFLAGS` as an environment variable at the same time as `target.<triple>.rustflags` in config — they are mutually exclusive; the env var wins and silently discards the config entry.

---

## Q2 — Emscripten virtual filesystem preloading: `--preload-file` vs `--embed-file`

### Decision

Use `--preload-file` for the `.se1` ephemeris data files.

### Rationale

| Flag | Storage | Best for |
|------|---------|----------|
| `--preload-file` | Side-car `.data` bundle, loaded via XHR at startup | Large binary blobs (5–10 MB); can be cached by the browser separately from the JS/WASM |
| `--embed-file` | Baked directly into the `.js` glue | Small text assets; no extra HTTP round-trip |

The two required ephemeris files (`sepl_18.se1`, `semo_18.se1`) are several MB each. `seas_18.se1` (asteroid data) is explicitly excluded — it is not needed for Vedic astrology. `--preload-file` allows the browser to cache the `.data` file independently and avoids bloating the JS/WASM bundle. It is the Emscripten-recommended approach for data files of this size.

### Path convention

Swiss Ephemeris uses `swe_set_ephe_path()` to locate `.se1` files. The virtual path must match what you specify at compile time. The canonical pattern for this project is:

```sh
emcc ... --preload-file ephe/@/ephe/
```

This maps the local `ephe/` directory (relative to the compile working directory) to `/ephe/` inside the WASM virtual FS at runtime. Then in Rust/C:

```c
swe_set_ephe_path("/ephe");
```

### Key flags

```sh
# Compile-time
--preload-file ephe/@/ephe/    # local dir @ virtual path

# Verify at runtime (browser console)
Module.FS.analyzePath('/ephe/sepl_18.se1')
```

---

## Q3 — Swiss Ephemeris C library compilation with emcc

### Decision

Compile the 9 core swisseph source files into `libswe.a` using `emar`. Skip `swejpl.c` only if JPL ephemeris support (DE431) is not required; include it for completeness since it adds minimal size.

### Rationale

The official `Makefile` in `aloistr/swisseph` defines the library object set as:

```makefile
SWEOBJ = swedate.o swehouse.o swejpl.o swemmoon.o swemplan.o sweph.o \
         swephlib.o swecl.o swehel.o
```

These 9 files are all required for the full `swe_calc` path (`SEFLG_SWIEPH`). The mobile/Android reference build (`contrib/android/jni/Android.mk`) uses the same set minus `swehel.c` and minus `swejpl.c`. For this project's minimum viable set (Sun longitude via Swiss Ephemeris format `.se1` files only):

| File | Purpose | Required? |
|------|---------|-----------|
| `swedate.c` | Julian date conversion | Yes |
| `swehouse.c` | House cusps (needed by sweph.c) | Yes |
| `swejpl.c` | JPL binary ephemeris I/O | Optional (omit if SEFLG_JPLEPH not used) |
| `swemmoon.c` | Moon Moshier algorithm | Yes |
| `swemplan.c` | Planetary Moshier algorithm (SEFLG_MOSEPH) | Yes |
| `sweph.c` | `swe_calc` main dispatcher | Yes |
| `swephlib.c` | Math utilities, delta-T | Yes |
| `swecl.c` | Eclipse + heliacal events | Optional for bare `swe_calc` |
| `swehel.c` | Heliacal visibility | Optional for bare `swe_calc` |

For the initial feature (sun longitude only), the **minimum set** is:
`swedate.c swehouse.c swemmoon.c swemplan.c sweph.c swephlib.c`

### Key commands

```sh
# Step 1: compile each .c to an Emscripten .o
EMCC=/astro/wasm/emsdk/upstream/emscripten/emcc
SWEDIR=vendor/swisseph

SWE_SRCS="swedate.c swehouse.c swemmoon.c swemplan.c sweph.c swephlib.c"

for src in $SWE_SRCS; do
  $EMCC -O2 -c "$SWEDIR/$src" -o "build/${src%.c}.o" \
    -I"$SWEDIR" \
    -DNOT_WINDOWS \
    -DEMSCRIPTEN
done

# Step 2: pack into a static archive
/astro/wasm/emsdk/upstream/emscripten/emar rcs lib/libswe.a build/*.o
```

### Known Emscripten compatibility issues with swisseph 2.x

- **`MSDOS` macro**: swisseph checks `#if MSDOS` for Windows-specific code paths. The `-DNOT_WINDOWS` flag (or `-DMSDOS=0`) prevents compilation of `<tchar.h>` / `<windows.h>` includes that are absent in the Emscripten sysroot.
- **`popen` / `pclose`**: These POSIX functions are unavailable in Emscripten (no subprocess support). They appear only in `swetest.c` (not in the library itself), so as long as you do not compile `swetest.c`, this is a non-issue.
- **`setjmp` / `longjmp`**: Used internally in some paths; Emscripten supports these.
- **File I/O**: swisseph uses `fopen`/`fread` to read `.se1` files. Emscripten's virtual FS fully supports these calls after preloading, so no changes are needed.
- **`sys/stat.h`**: Included in `swehel.c`. Available in the Emscripten sysroot, no issue.

---

## Q4 — Exporting functions from wasm32-unknown-emscripten: single `bridge` design

### Decision

Export exactly **one** function: `bridge(op_ptr, input_ptr, output_ptr, output_max_len) → i32`.
Never add per-operation exports. All operations are routed inside Rust. `-sEXPORTED_FUNCTIONS=_bridge` is set once in `.cargo/config.toml` and never changed.

### Rationale

Adding a new `-sEXPORTED_FUNCTIONS` entry every time a new calculation is added is fragile — it requires coordinated changes to `build.sh`, `.cargo/config.toml`, and the JS layer. Instead, a single `bridge` function acts as the sole entry point. `op` identifies the operation (like an API route), `input` carries a JSON payload, and `output` is a pre-allocated buffer owned and freed by the JS caller.

**Memory ownership principle: "you make the mess, you clean it up."**  
Both input and output buffers are allocated by JS via `Module._malloc` and freed by JS via `Module._free`, always in a `finally` block. Rust reads from input and writes to output. Rust never allocates anything visible to JS.

### Bridge signature

```
bridge(op_ptr: *const c_char,
       input_ptr: *const c_char,
       output_ptr: *mut c_char,
       output_max_len: i32) → i32
```

| Return value | Meaning |
|---|---|
| `> 0` and `≤ output_max_len` | Success; bytes written to output buffer |
| `> output_max_len` | Buffer too small; return value is the required size — retry with that size |
| `< 0` | Error (`-1` = unknown op, `-2` = JSON parse error, `-3` = calc error) |

### Rust declaration

```rust
#[no_mangle]
pub extern "C" fn bridge(
    op_ptr: *const std::os::raw::c_char,
    input_ptr: *const std::os::raw::c_char,
    output_ptr: *mut std::os::raw::c_char,
    output_max_len: i32,
) -> i32 {
    // Safety: JS owns and manages the lifetime of all three buffers.
    // Rust must not free, reallocate, or store these pointers beyond this call.
    let op = unsafe { std::ffi::CStr::from_ptr(op_ptr) }.to_str().unwrap_or("");
    let input = unsafe { std::ffi::CStr::from_ptr(input_ptr) }.to_str().unwrap_or("{}");
    // ... route op → handler, serialize result → output_ptr
}
```

### Linker flag (set once, permanent)

```
-sEXPORTED_FUNCTIONS=_bridge
```

### JS boilerplate (written once, never changes)

```js
const INITIAL_OUTPUT_SIZE = 4096; // 4 KB covers most results on first try

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
      // Buffer was too small — Rust returned required size; retry once
      Module._free(outPtr);
      outPtr  = Module._malloc(written);
      outSize = written;
      written = Module._bridge(opPtr, inPtr, outPtr, outSize);
    }

    if (written < 0) throw new Error('bridge error: ' + written);

    const bytes = Module.HEAPU8.subarray(outPtr, outPtr + written);
    return JSON.parse(new TextDecoder().decode(bytes)); // TextDecoder = Web API, no Emscripten dep
  } finally {
    Module._free(opPtr);   // JS frees its own inputs — always runs
    Module._free(inPtr);
    Module._free(outPtr);
  }
}
```

**What `_malloc` and `_free` are**: standard Emscripten-exported heap allocators. They are always available when `ALLOW_MEMORY_GROWTH=1` is set; no extra `EXPORTED_RUNTIME_METHODS` entry needed.

### Invoking from application code (example)

```js
// In onRuntimeInitialized — every future feature looks exactly like this:
const result = bridge('sun_longitude', JSON.stringify({ tjd: 2451545.0 }));
console.log('Sun longitude (J2000): ' + result.longitude.toFixed(6) + '°');
```

### Complete export pipeline

```
Rust  #[no_mangle] pub extern "C" fn bridge(...)
         ↓
rustc → LLVM IR (symbol: bridge)
         ↓
emcc linker: -sEXPORTED_FUNCTIONS=_bridge  (set once, never changes)
         ↓
JS: Module._bridge(opPtr, inPtr, outPtr, outSize)
         ↓
All operation routing, calculation, serialization happens inside Rust
```

---

## Q5 — `onRuntimeInitialized` pattern for Emscripten JS glue (vanilla JS)

### Decision

Declare a `Module` global object **before** loading the Emscripten-generated `.js` file. Set `Module.onRuntimeInitialized` as a callback. All `bridge()` calls must happen inside this callback (or after it fires). No `ccall` or `cwrap` — we use `_malloc`/`_free` + `TextEncoder`/`TextDecoder` directly.

### Rationale

Emscripten compiles WASM asynchronously. `onRuntimeInitialized` fires exactly once, after WASM instantiation and all `--preload-file` data are loaded. Calling `Module._bridge()` before this fires will throw or misbehave. `ccall` and `cwrap` are not needed — our `bridge()` JS helper (see Q4) uses only `_malloc`, `_free`, and `HEAPU8`, which are always available.

### Minimal `index.html` pattern

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Shubharambham</title>
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <p>Work in Progress</p>

  <script>
    // bridge() helper — written once, never changes (see Q4 for full version)
    const INITIAL_OUTPUT_SIZE = 4096;
    function bridge(op, inputJson) { /* ... see Q4 ... */ }

    // 1. Declare Module BEFORE the script tag that loads the emcc output.
    var Module = {
      print:    function(t) { console.log('[wasm]', t); },
      printErr:  function(t) { console.error('[wasm]', t); },
      noExitRuntime: true,   // prevent runtime shutdown if main() returns
      // 2. Fires once WASM + preloaded .data are fully ready.
      onRuntimeInitialized: function() {
        // 3. Verify pipeline: call bridge with sun_longitude op
        // J2000.0 ≈ 2451545.0 (Julian Day for 2000-01-01 12:00 UTC)
        const result = bridge('sun_longitude', JSON.stringify({ tjd: 2451545.0 }));
        console.log('Sun longitude (J2000): ' + result.longitude.toFixed(6) + '°');
      }
    };
  </script>

  <!-- 4. Load emcc-generated glue AFTER Module declaration -->
  <script src="astro.js"></script>
</body>
</html>
```

### Notes

- `noExitRuntime: true` is required when the Rust entry point is a library (no `main`), to prevent the Emscripten runtime from tearing down after `main` returns.
- The `.data` preload bundle is fetched relative to `astro.js`. Keep all dist files in the same directory.
- `Module.locateFile` can redirect `.wasm`/`.data` to a CDN or versioned path if needed in future — no change to `index.html` structure required.

---

## Q6 — `build.sh` ordering

### Decision

The correct sequence is: **(a) compile swisseph → (b) cargo build → (c) copy dist**. emcc must produce `libswe.a` before `cargo build` can link against it.

### Rationale

`cargo build` with `build.rs` resolves `rustc-link-search` paths at the start of its link step, so the `.a` file must exist before `cargo build` runs. The dist copy is last because the WASM artifacts do not exist until cargo finishes.

### Annotated `build.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

# ── Environment ──────────────────────────────────────────────────────────────

# Emscripten toolchain (sourced from emsdk_env.sh, or set directly)
EMSDK_ROOT="${EMSDK_ROOT:-/astro/wasm/emsdk}"
EMCC="$EMSDK_ROOT/upstream/emscripten/emcc"
EMAR="$EMSDK_ROOT/upstream/emscripten/emar"

# Cache for Emscripten's system library builds (speeds up repeat builds)
export EM_CACHE="$EMSDK_ROOT/upstream/emscripten/cache"

SWEDIR="vendor/swisseph"
LIBDIR="lib"
BUILDDIR="build/swe_objs"
DISTDIR="dist"

mkdir -p "$BUILDDIR" "$LIBDIR" "$DISTDIR"

# ── (a) Compile swisseph C sources into libswe.a ─────────────────────────────

echo "==> Compiling Swiss Ephemeris..."

SWE_SRCS="swedate.c swehouse.c swemmoon.c swemplan.c sweph.c swephlib.c"

for src in $SWE_SRCS; do
  "$EMCC" -O2 -c "$SWEDIR/$src" -o "$BUILDDIR/${src%.c}.o" \
    -I"$SWEDIR" \
    -DNOT_WINDOWS
done

"$EMAR" rcs "$LIBDIR/libswe.a" "$BUILDDIR"/*.o
echo "    libswe.a written to $LIBDIR/"

# ── (b) Compile Rust to wasm32-unknown-emscripten ────────────────────────────

echo "==> Building Rust WASM module..."

# Tell the build.rs where to find libswe.a  (can also be read from env in build.rs)
export LIBSWE_DIR="$(pwd)/$LIBDIR"

cargo build \
  --target wasm32-unknown-emscripten \
  --release 2>&1

echo "    WASM build complete."

# ── (c) Copy artifacts to dist/ ──────────────────────────────────────────────

echo "==> Copying dist artifacts..."

WASM_OUT="target/wasm32-unknown-emscripten/release"

cp "$WASM_OUT"/*.wasm  "$DISTDIR/"
cp "$WASM_OUT"/*.js    "$DISTDIR/"
cp src/index.html      "$DISTDIR/"

echo "==> Build complete. Serve dist/ with: python3 -m http.server --directory dist/"
```

### Required environment variables

| Variable | Purpose | Set by |
|----------|---------|--------|
| `EM_CACHE` | Persistent cache for emcc syslib builds (avoids recompiling libc etc.) | You (or emsdk_env.sh) |
| `EMSDK_ROOT` | Root of the emsdk install | You (or emsdk_env.sh) |
| `LIBSWE_DIR` | Path to dir containing `libswe.a`, read in `build.rs` | `build.sh` before `cargo build` |
| `EMCC_CFLAGS` | Optional extra flags injected into every `emcc` invocation | Only needed for debug/sanitizer overrides |

> **Do not set `EMCC_CFLAGS` in normal builds.** It injects flags globally and can conflict with per-file flags.

---

## Q7 — swisseph ephemeris data path at runtime in WASM

### Decision

Pass the string `"/ephe"` to `swe_set_ephe_path()`. This must be called before the first `swe_calc_ut()` call.

### Rationale

Swiss Ephemeris uses `swe_set_ephe_path()` to set the directory where it searches for `.se1` binary ephemeris files. When running inside Emscripten's virtual FS, the path is a POSIX-style absolute path within that virtual FS. If files are preloaded with `--preload-file ephe/@/ephe/`, they appear at `/ephe/sepl_18.se1`, `/ephe/semo_18.se1`, etc. The path argument to `swe_set_ephe_path` should be the directory, without a trailing slash, matching the virtual FS mount point.

If `swe_set_ephe_path(NULL)` is called (or not called), swisseph falls back to the environment variable `SE_EPHE_PATH`, then to a compiled-in default (typically `/usr/share/ephe` on Linux, which does not exist in WASM). This will cause `swe_calc` to fall back silently to the built-in Moshier algorithm (`SEFLG_MOSEPH`), which has lower precision. Always call `swe_set_ephe_path` explicitly.

### Rust code pattern

```rust
use std::ffi::CString;

extern "C" {
    fn swe_set_ephe_path(path: *const std::os::raw::c_char);
    fn swe_calc(tjd: f64, ipl: i32, iflag: i32, xx: *mut f64, serr: *mut std::os::raw::c_char) -> i32;
    fn swe_close();
}

#[no_mangle]
pub extern "C" fn sun_longitude(tjd_tt: f64) -> f64 {
    let path = CString::new("/ephe").unwrap();
    unsafe {
        swe_set_ephe_path(path.as_ptr());
        let mut xx = [0f64; 6];
        let mut serr = [0i8; 256];
        // SE_SUN = 0, SEFLG_SWIEPH = 2
        let _ret = swe_calc(tjd_tt, 0, 2, xx.as_mut_ptr(), serr.as_mut_ptr());
        xx[0]  // ecliptic longitude in degrees
    }
}
```

### preload-file flag that matches this path

```sh
emcc ... --preload-file ephe/@/ephe/
#                       ^^^^  ^^^^^^
#   local directory ────┘      └── virtual FS path (matches swe_set_ephe_path arg)
```

### Ephemeris files needed (Swiss Ephemeris format, not JPL)

| File | Covers | Notes |
|------|--------|-------|
| `sepl_18.se1` | Sun, Moon, planets | Main planet file |
| `semo_18.se1` | Moon (detailed) | Required for `SE_MOON` |
| `seas_18.se1` | Asteroids | Only if computing asteroids |

For sun-only calculations: only `sepl_18.se1` is strictly required. Load `semo_18.se1` as well to support Moon calculations later without a rebuild.

---

## Summary Table

| # | Question | Key Decision |
|---|----------|-------------|
| 1 | Cargo linker config | `linker = "emcc"` in `.cargo/config.toml` + `build.rs` for `-lswe` |
| 2 | File preloading | `--preload-file ephe/@/ephe/` for `.se1` files (not `--embed-file`) |
| 3 | swisseph compilation | 6–9 C sources → `emar rcs libswe.a`; `-DNOT_WINDOWS` required |
| 4 | Rust/JS bridge design | Single `bridge(op,in,out,max_len)→i32`; JS owns all buffers; Option B retry pattern; `_bridge` is the only permanent WASM export |
| 5 | JS initialization | `Module.onRuntimeInitialized` + `bridge()` helper using `_malloc/_free` + `TextEncoder/TextDecoder`; no `ccall` |
| 6 | build.sh order | swisseph `.a` → cargo build → copy dist |
| 7 | Ephemeris path | `swe_set_ephe_path("/ephe")` matching the `--preload-file` virtual path |
