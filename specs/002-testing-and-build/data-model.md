# Data Model: Testing and Build Tooling (Feature 002)

**Branch**: `002-testing-and-build` | **Date**: 2026-03-14

This feature introduces no new business-logic entities (no new Rust data types, no new WASM API contracts, no new DOM structures). All changes are to the build pipeline, test infrastructure, and file layout.

---

## File Change Map

### New Files

| File | Type | Purpose |
|------|------|---------|
| `package.json` | Config | npm manifest: dev deps, `build`, `test`, `postinstall` scripts |
| `vite.config.ts` | Config | Vite 5 build config: `root='web'`, `publicDir='../public'`, `build.outDir='../dist'` |
| `playwright.config.ts` | Config | Playwright: headless Chromium, `webServer`, `baseURL`, `reuseExistingServer` |
| `tsconfig.json` | Config | TypeScript: scoped to config + test files only |
| `tests/astro.spec.ts` | Test | Playwright E2E: page body assertion + Sun longitude console capture |
| `public/` | Directory | Vite pass-through; created by `build.sh`; populated by Emscripten; gitignored |

### Modified Files

| File | Change Type | Description |
|------|-------------|-------------|
| `astro-wasm/Cargo.toml` | Addition | Add `[build-dependencies] cc = "1"` |
| `astro-wasm/build.rs` | Rewrite | TARGET-branch: emscripten → existing link args + `-o public/astro.js`; native → `cc::Build` for swisseph C sources |
| `astro-wasm/src/lib.rs` | Refactor + Addition | Remove `swe_set_ephe_path` call from `handle_sun_longitude`; add `#[cfg(test)]` module with unit tests for all public functions |
| `build.sh` | Addition | Add npm prerequisite check; add `cargo test` step (Phase 2.5); replace `cp` with `npm run build`; add `npm test` as final step; add `--skip-rust-tests` / `--skip-playwright-tests` flag parsing |
| `.gitignore` | Addition | `node_modules/`, `public/astro.*`, `.vite/`, `playwright-report/`, `test-results/` |

### Deleted Files

| File | Reason |
|------|--------|
| `test-wasm.mjs` | Replaced by Playwright E2E tests; Node.js polyfill harness no longer needed |

---

## `build.sh` Phase Sequence (after this feature)

```
Phase 0  — Flag parsing (--skip-rust-tests, --skip-playwright-tests)
Phase 1  — Prerequisite checks (emcc, npm, vendor/, ephe/, Rust target)
Phase 2  — Compile Swiss Ephemeris C sources with emcc → lib/libswe.a  [unchanged]
Phase 2.5— Rust native tests:
             unset LIBSWE_DIR
             cargo test  (cc crate compiles libswe natively; ephe path from CARGO_MANIFEST_DIR)
             export LIBSWE_DIR=lib/
Phase 3  — Rust WASM build → public/astro.{js,wasm,data}              [path change: dist→public]
Phase 4  — npm install (if node_modules/ absent)
Phase 5  — npm run build (Vite: web/ → dist/, public/ copied verbatim)
Phase 6  — npm test (Playwright: serves dist/ on :4173, headless Chromium)
```

---

## `build.rs` Branch Structure

```
TARGET env var
├── contains "emscripten"
│   ├── cargo:rustc-link-search=native=$LIBSWE_DIR
│   ├── cargo:rustc-link-lib=static=swe
│   ├── cargo:rustc-link-arg=--preload-file  <ephe>@/ephe/
│   └── cargo:rustc-link-arg=-o  public/astro.js          ← path updated from dist/
└── (native)
    └── cc::Build
        ├── .include("../vendor/swisseph")
        ├── .define("NOT_WINDOWS", None)
        ├── .opt_level(2)
        ├── .warnings(false)
        ├── .file("../vendor/swisseph/swedate.c")
        ├── ... (7 source files)
        └── .compile("swe")   → emits link-lib + link-search automatically
```

---

## `lib.rs` Structural Changes

### Before
```
handle_sun_longitude()
  └─ calls swe_set_ephe_path("/ephe\0")  ← hardcoded Emscripten virtual path
  └─ calls swe_calc_ut(...)
```

### After
```
bridge()
  └─ calls swe_set_ephe_path("/ephe\0")  ← one-time setup per invocation (moved here)
  └─ dispatches to handle_sun_longitude()

handle_sun_longitude()
  └─ calls swe_calc_ut(...)              ← no longer sets ephe path

#[cfg(test)]
mod tests {
  use super::*;
  // shared setup fn: calls swe_set_ephe_path with concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe\0")
  // test_bridge_unknown_op()     → bridge("unknown_op", ...) returns -1
  // test_sun_longitude_j2000()   → handle_sun_longitude at TJD 2451545.0 ≈ 280.37°
  // ... one test per public function
}
```

---

## `package.json` Key Shape

```json
{
  "scripts": {
    "build": "vite build",
    "test": "playwright test",
    "postinstall": "npx playwright install chromium"
  },
  "devDependencies": {
    "@playwright/test": ">=1.51.0",
    "vite": "^5.0.0",
    "typescript": "^5.0.0"
  }
}
```

---

## `.gitignore` Additions

```
# npm
node_modules/

# Vite
.vite/

# Playwright
playwright-report/
test-results/

# WASM build artifacts (Emscripten output in public/)
public/astro.js
public/astro.wasm
public/astro.data
```

Note: `dist/` is already gitignored (feature 001). Verify before adding a duplicate entry.

---

## No New WASM API Contracts

The `bridge()` function signature is unchanged. The `wasm-api-v1.md` contract remains valid. FR-016 (move `swe_set_ephe_path` call to `bridge()`) is an internal refactor invisible to JS callers — the contract surface is unchanged.
