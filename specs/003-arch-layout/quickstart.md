# Quickstart: Modular Architecture Layout

**Feature**: `003-arch-layout`  
**Date**: 2026-03-14

---

## Prerequisites

All existing prerequisites from feature 002 remain required:

```bash
# 1. Emscripten toolchain (>= 4.0.0) in PATH
source $EMSDK/emsdk_env.sh

# 2. Swiss Ephemeris submodule
git submodule update --init

# 3. Ephemeris data files present in ephe/
ls ephe/semo_18.se1 ephe/sepl_18.se1

# 4. Rust WASM target
rustup target add wasm32-unknown-emscripten
```

---

## Run Rust Unit Tests Only

Fast inner-loop check for the Rust layer during development. No Emscripten required.

```bash
cd astro-wasm
cargo test
```

Expected output includes:
```
test tests::test_bridge_unknown_op ... ok
test tests::test_sun_longitude_j2000 ... ok
test tests::test_utils_iso_to_jd ... ok
```

---

## Full Build and Test

```bash
./build.sh
```

This runs all six phases in sequence:
1. Prerequisite checks
2. Compile Swiss Ephemeris C → `lib/libswe.a`
3. `cargo test` (Rust unit tests)
4. `cargo build --target wasm32-unknown-emscripten --release` → `public/`
5. `npm run build` (Vite → `dist/`)
6. `npm test` (Playwright E2E)

Skip flags available for faster iteration:

```bash
./build.sh --skip-rust-tests           # Skip cargo test (Phase 2.5)
./build.sh --skip-playwright-tests     # Skip npm test (Phase 6)
./build.sh --skip-rust-tests --skip-playwright-tests  # WASM build only
```

---

## Try the Page Manually

After a successful build, serve `dist/`:

```bash
# Any static server. Example with Python:
python3 -m http.server 8080 --directory dist

# Or with npx:
npx serve dist
```

Open `http://localhost:8080`, enter a date/time (e.g., `2000-01-01T12:00:00Z`), select a language, and click Submit. The result shows the localized planet label and longitude.

---

## Adding a New Engine

This feature establishes the pattern. Future engines follow the same steps:

1. **Create the engine module** — `astro-wasm/src/engines/<name>.rs`:
   ```rust
   use crate::swe_wrappers;
   use crate::localization::{Planet, Lang, get_label};

   pub struct <Name>Result {
       pub label: &'static str,
       pub longitude: f64,
   }

   pub fn handle_<name>(jd: f64, lang: Lang) -> Result<<Name>Result, String> {
       // call swe_wrappers::...
       // call get_label(Planet::<Name>, lang)
       todo!()
   }
   ```

2. **Declare the module** — add `pub mod <name>;` to `astro-wasm/src/engines/mod.rs`.

3. **Register the operation** — add one `match` arm in `astro-wasm/src/bridge.rs`:
   ```rust
   "<name>" => {
       let jd = ...;
       let lang = Lang::from_str(&req.lang);
       match engines::<name>::handle_<name>(jd, lang) {
           Ok(r) => serialize_success(r.label, r.longitude, output_ptr, output_max_len),
           Err(e) => serialize_error(&e, output_ptr, output_max_len),
       }
   }
   ```

4. **Add to `data.js`** — add a new named function following the `getSunLongitude` pattern.

5. **Write a unit test** — add `test_<name>_reference_value()` in `engines/<name>.rs`.

No changes needed in `swe_wrappers`, `utils`, or `localization` unless the new engine requires new SWE functions or new locale strings.

---

## File Map

| File | Purpose |
|------|---------|
| `astro-wasm/src/lib.rs` | Crate root — module declarations |
| `astro-wasm/src/swe_wrappers.rs` | All Swiss Ephemeris `extern "C"` declarations |
| `astro-wasm/src/utils.rs` | `iso_to_jd()`, `normalize_degrees()` |
| `astro-wasm/src/localization.rs` | `get_label(Planet, Lang)` |
| `astro-wasm/src/bridge.rs` | `bridge()` WASM export, JSON parse/dispatch/serialize |
| `astro-wasm/src/engines/mod.rs` | Sub-module declarations |
| `astro-wasm/src/engines/sun.rs` | `handle_sun_longitude(jd, lang)` |
| `web/astro-glue.js` | WASM init, buffer management, `bridge()` JS helper |
| `web/data.js` | `getSunLongitude(isoDatetime, lang)` |
| `web/components.js` | Stub — placeholder for future UI components |
| `web/index.html` | Datetime input + lang selector + result element |
| `tests/astro.spec.ts` | Playwright E2E: asserts `en` and `te` labels + longitude |
| `specs/003-arch-layout/contracts/wasm-api-v2.md` | WASM API contract |
