# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
<!-- SPECKIT END -->

## What This Is

A Vedic astrology web app ("shubharambham" = auspicious beginning in Telugu). The computation core is a Rust library compiled to WebAssembly via Emscripten, wrapping the Swiss Ephemeris C library. The frontend is React + TypeScript built with Vite. Supported languages: English (`en`) and Telugu (`te`).

## Build Commands

**Full build (all phases):**
```bash
./build.sh
```

**Skip slow phases during development:**
```bash
./build.sh --skip-rust-tests          # skip cargo test
./build.sh --skip-playwright-tests    # skip E2E tests
./build.sh --skip-rust-tests --skip-playwright-tests
```

**Individual phases:**
```bash
# Rust unit tests only (no Emscripten needed)
cd astro-wasm && cargo test

# Run a single Rust test
cd astro-wasm && cargo test test_bridge_list_cities_en

# Frontend build only (requires public/ artifacts from Rust build)
npm run build

# E2E tests only (requires dist/ from Vite build, serves on :4173)
npm test
```

## Build Prerequisites

Before `./build.sh` will work:
1. Emscripten ≥ 4.0.0 on PATH — `source $EMSDK/emsdk_env.sh`
2. Swiss Ephemeris submodule — `git submodule update --init`
3. Rust WASM target — `rustup target add wasm32-unknown-emscripten`
4. Ephemeris data in `ephe/` — `semo_18.se1` and `sepl_18.se1` (download from astro.com/ftp/swisseph/ephe/)

## Architecture

### Build pipeline (build.sh phases)
1. Compile `vendor/swisseph/*.c` → `lib/libswe.a` using `emcc`
2. `cargo test` (native, compiles swisseph via `cc` crate — no Emscripten needed)
3. `cargo build --target wasm32-unknown-emscripten --release` → writes `public/astro.js`, `public/astro.wasm`, `public/astro.data`
4. `npm run build` (Vite: `web/` source → `dist/`, copies `public/` verbatim)
5. `npm test` (Playwright E2E against `dist/` served on port 4173)

### Rust WASM crate (`astro-wasm/`)
- **`bridge.rs`** — sole `#[no_mangle]` WASM export: `fn bridge(op, input, output, max_len) -> i32`. Routes operation strings to handlers. Return: `>0` = bytes written, `-1` = unknown op, `-2` = parse/validation error, `-3` = SWE calculation error.
- **`engines/horoscope.rs`** — planetary positions via Swiss Ephemeris with True Chitrapaksha Ayanamsa
- **`engines/vimsottari.rs`** — Vimsottari Dasa/Antardasa period calculation from Moon's nakshatra
- **`swe_wrappers.rs`** — raw FFI bindings to `libswe.a`
- **`data/`** — city data; `build.rs` codegen reads `data/cities.csv` → `$OUT_DIR/cities_generated.rs` at compile time
- **`localization.rs`**, **`locales/`** — English and Telugu string tables

The `build.rs` dual-path pattern: when `TARGET` contains `emscripten`, it links the pre-built `lib/libswe.a` (from Phase 2) and emits emcc flags via `cargo:rustc-link-arg`; otherwise it compiles swisseph sources directly via the `cc` crate so `cargo test` works without Emscripten.

### Frontend (`web/`)
- **`web/horoscope/`** — main React app with MUI components
- **`web/horoscope/astro-glue.ts`** — typed TypeScript wrapper; calls `window.Module._bridge(...)` by allocating C strings in WASM heap, invoking the bridge, and JSON-decoding the output buffer. Three exported functions: `listCities`, `getHoroscopePositions`, `getVimsottariDasa`.
- **`web/astro-glue.js`** / **`web/data.js`** / **`web/components.js`** — plain JS globals for the legacy landing page; Vite copies these verbatim to `dist/` without bundling.
- WASM module loads as `window.Module` via Emscripten's generated `astro.js`; the React app waits for `Module.onRuntimeInitialized` before enabling the form.

### Buffer sizing (astro-glue.ts)
- `list_cities`: 512 KiB (full city list serialised to JSON)
- `horoscope_positions` / `vimsottari_dasa`: 64 KiB

## Key Invariants

- `canonicalName` in city records is always ASCII English regardless of `lang` — used as stable city key across languages.
- The `lang` field in all requests selects both the city name script (`te` = Telugu script) and planet/nakshatra label translations.
- Ephemeris path differs by compilation target: `/ephe/` in WASM (Emscripten virtual FS preload), `../ephe/` in native `cargo test`.
- `RUSTFLAGS` env var silently overrides `.cargo/config.toml` — do not set it when building for WASM.
