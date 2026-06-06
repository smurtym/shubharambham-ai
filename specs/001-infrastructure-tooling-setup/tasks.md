# Tasks: Infrastructure and Tooling Setup

**Feature**: `001-infrastructure-tooling-setup`  
**Date**: 2026-03-14  
**Input**: plan.md, spec.md, research.md, contracts/wasm-api-v1.md, quickstart.md  
**Stack**: Rust 1.88, Emscripten 4.0.13, Swiss Ephemeris (vendor/swisseph), `wasm32-unknown-emscripten`, vanilla JS

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[US1/US2/US3]**: Maps to user story from spec.md
- All paths relative to repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the repository skeleton, submodule, Rust config, and ephemeris data that every later phase depends on.

- [X] T001 Create repository skeleton: `lib/`, `build/swe_objs/`, `web/`, `ephe/` directories; add `ephe/`, `dist/`, `lib/`, `build/`, `astro-wasm/target/` to `.gitignore`
- [X] T002 Add `vendor/swisseph` as a Git submodule pinned to the latest tag: `git submodule add https://github.com/aloistr/swisseph vendor/swisseph && git submodule update --init`
- [X] T003 [P] Create `.cargo/config.toml` with `[target.wasm32-unknown-emscripten]` section: `linker = "emcc"` and `rustflags` array containing `-sENVIRONMENT=web`, `-sEXPORTED_FUNCTIONS=_bridge`, `--preload-file ephe/@/ephe/`, `-sALLOW_MEMORY_GROWTH=1` (see research.md Q1 for exact TOML)
- [X] T004 [P] Initialize `astro-wasm` Rust library crate: create `astro-wasm/Cargo.toml` with `crate-type = ["cdylib"]` and an empty `astro-wasm/src/lib.rs` placeholder
- [X] T005 [P] Download `semo_18.se1` and `sepl_18.se1` into `ephe/` from the Astrodienst FTP (see research.md Q2 and quickstart.md Step 2 for URLs); verify both files are present and non-zero in size

**Checkpoint**: Submodule populated, `.cargo/config.toml` set, `ephe/` contains both `.se1` files

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build orchestrator shell with prerequisite validation. Must exist before any build steps in later phases can run.

**⚠️ CRITICAL**: No user story build can be invoked until this phase is complete.

- [X] T006 Create `build.sh` at repo root with prerequisite checks only (no build steps yet): verify `emcc` is on PATH and Emscripten ≥ 4.0.0, verify `vendor/swisseph` is populated (`.c` files present), verify `ephe/semo_18.se1` and `ephe/sepl_18.se1` exist, verify `rustup target list --installed` contains `wasm32-unknown-emscripten`; exit with a clear human-readable error if any check fails (see research.md Q6 for `set -euo pipefail` pattern and env variable conventions)

**Checkpoint**: `./build.sh` runs to completion (prereq-check-only) without errors on a correctly configured machine

---

## Phase 3: User Story 1 — Static Placeholder Webpage (Priority: P1) 🎯 MVP

**Goal**: A developer opens `dist/index.html` via a local HTTP server and sees "Work in Progress" with no JavaScript errors.

**Independent Test**: Run `./build.sh`, then `python3 -m http.server 8080 --directory dist/`, open `http://localhost:8080` — page renders, no broken resources, no console errors.

- [X] T007 [P] [US1] Create `web/style.css`: minimal mobile-first stylesheet — `box-sizing: border-box`, responsive `max-width` body container, legible base font, `<meta name="viewport">` -compatible layout (no JS, no external fonts)
- [X] T008 [P] [US1] Create `web/index.html`: valid HTML5 with `<!DOCTYPE html>`, `<meta charset="UTF-8">`, `<meta name="viewport" content="width=device-width, initial-scale=1.0">`, `<link rel="stylesheet" href="style.css">`, and a visible `<p>Work in Progress</p>` element; no WASM script elements yet
- [X] T009 [US1] Add dist copy step to `build.sh`: append a section that runs `mkdir -p dist/ && cp web/index.html web/style.css dist/` after prerequisite checks
- [X] T010 [US1] Verify US1 end-to-end: run `./build.sh`, serve `dist/` with `python3 -m http.server 8080 --directory dist/`, open `http://localhost:8080` in a browser and confirm "Work in Progress" is visible, no JavaScript errors appear in the console, and no resources return HTTP 404

**Checkpoint**: US1 fully functional and independently verifiable — static HTML pipeline is end-to-end

---

## Phase 4: User Story 2 — Swiss Ephemeris Library Compiles (Priority: P2)

**Goal**: `./build.sh` produces `lib/libswe.a` and `dist/astro.data` with no compiler errors; ephemeris files accessible in the WASM virtual filesystem.

**Independent Test**: Run `./build.sh`; confirm `lib/libswe.a` exists and `dist/astro.data` exists; build log contains no errors or warnings treated as errors.

- [X] T011 [US2] Add swisseph compilation step to `build.sh`: compile the 6 required C source files (`swedate.c swehouse.c swemmoon.c swemplan.c sweph.c swephlib.c`) from `vendor/swisseph/` using `emcc -O2 -c` with `-I vendor/swisseph -DNOT_WINDOWS` flags, output `.o` files to `build/swe_objs/`, then pack with `emar rcs lib/libswe.a build/swe_objs/*.o` (see research.md Q3 for exact commands and flag rationale)
- [X] T012 [US2] Create `astro-wasm/build.rs`: emit `cargo:rustc-link-search=native=${LIBSWE_DIR}` (reading the `LIBSWE_DIR` env var set by `build.sh`) and `cargo:rustc-link-lib=static=swe` to link `libswe.a` into the WASM module (see research.md Q1 for the exact `build.rs` snippet)
- [X] T012a [US2] Add `serde_json` to `astro-wasm/Cargo.toml`: `serde_json = { version = "1", default-features = false, features = ["alloc"] }` — required by T017 to parse the bridge JSON input payload inside Rust; the `alloc` feature is `no_std`-compatible and keeps the WASM binary small (Constitution I: prefer low-footprint crates)
- [X] T013 [US2] Write a compilable stub in `astro-wasm/src/lib.rs`: `#[no_mangle] pub extern "C" fn bridge(op_ptr: *const std::os::raw::c_char, input_ptr: *const std::os::raw::c_char, output_ptr: *mut std::os::raw::c_char, output_max_len: i32) -> i32 { 0 }` — sufficient to verify the Rust → Emscripten link succeeds against `libswe.a`
- [X] T014 [US2] Add Rust WASM build step and artifact copy to `build.sh`: export `LIBSWE_DIR="$(pwd)/lib"`, run `cargo build --target wasm32-unknown-emscripten --release` inside `astro-wasm/`, then copy `astro-wasm/target/wasm32-unknown-emscripten/release/astro_wasm.js`, `astro_wasm.wasm`, and `astro_wasm.data` into `dist/` (renaming to `astro.js`, `astro.wasm`, `astro.data`) (see research.md Q6 for full annotated `build.sh` and env variable table)
- [X] T015 [US2] Verify US2 end-to-end: run `./build.sh`, confirm `lib/libswe.a` is present, confirm `dist/astro.data` is present and non-zero, confirm build log emitted no `error[E…]` or `warning[…]` lines treated as errors; optionally open browser console and run `Module.FS.analyzePath('/ephe/sepl_18.se1')` to confirm object is found

**Checkpoint**: US2 fully functional — `libswe.a` compiles, WASM module links, ephemeris data preloaded

---

## Phase 5: User Story 3 — Rust Sun Ephemeris Wrapper Executes (Priority: P3)

**Goal**: Browser console shows `Sun longitude (J2000): 280.XXXX°` (within 0.001° of 280.459°) when the page loads.

**Independent Test**: Run `./build.sh`, open `http://localhost:8080`, confirm the console log line appears with a value in [280.458°, 280.460°] and no runtime exceptions are thrown.

- [X] T016 [US3] Add swisseph FFI declarations to `astro-wasm/src/lib.rs`: `extern "C"` block declaring `fn swe_set_ephe_path(path: *const std::os::raw::c_char)`, `fn swe_calc_ut(tjd_ut: f64, ipl: i32, iflag: i32, xx: *mut f64, serr: *mut std::os::raw::c_char) -> i32`; add `const SE_SUN: i32 = 0;` and `const SEFLG_SWIEPH: i32 = 2;` (see research.md Q7 and contracts/wasm-api-v1.md for the `sun_longitude` operation spec)
- [X] T017 [US3] Implement `bridge()` routing in `astro-wasm/src/lib.rs`: replace the stub — read `op` and `input` from their `CStr` pointers; match `"sun_longitude"` op; call `swe_set_ephe_path(b"/ephe\0".as_ptr() as _)` then `swe_calc_ut(tjd_ut, SE_SUN, SEFLG_SWIEPH, xx, errbuf)` with `tjd_ut` parsed from the JSON input; on success, write `{"longitude":<value>}` into `output_ptr` up to `output_max_len` bytes and return bytes written; if buffer too small, return required size without writing; on error return `-3`; for unknown op return `-1` (see contracts/wasm-api-v1.md for exact return value semantics)
- [X] T018 [US3] Update `web/index.html`: add the `bridge()` JS helper function (with `INITIAL_OUTPUT_SIZE = 4096`, Option B retry, `TextEncoder`/`TextDecoder`, `_malloc`/`_free` in `try/finally`) and the `var Module = { onRuntimeInitialized: function() { ... } }` block inside a `<script>` element; the `onRuntimeInitialized` callback must call `bridge('sun_longitude', JSON.stringify({ tjd: 2451545.0 }))` and log the result longitude with `console.log('Sun longitude (J2000): ' + result.longitude.toFixed(6) + '°')` (see contracts/wasm-api-v1.md and research.md Q4/Q5 for exact boilerplate)
- [X] T019 [US3] Add `<script src="astro.js"></script>` to `web/index.html` immediately after the `Module` declaration `<script>` block and before `</body>` — order is required: `Module` must be defined before Emscripten glue loads
- [X] T020 [US3] Verify US3 end-to-end: run `./build.sh`, open `http://localhost:8080`, confirm browser console shows `Sun longitude (J2000): 280.459XXX°`; value must be within 0.001° of 280.459°; no unhandled exceptions thrown

**Checkpoint**: Full pipeline verified — Rust → Emscripten → swisseph → browser console

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Satisfy the remaining FRs (FR-008, FR-002 size constraint) and validate the developer experience.

- [X] T021 [P] Add WASM-not-supported detection to `web/index.html` (FR-008): before the `Module` declaration, add `if (typeof WebAssembly === 'undefined') { console.error('WebAssembly is not supported in this browser. Calculations are unavailable.'); }` — the "Work in Progress" text must still render
- [X] T022 [P] Validate initial page payload size: run `wc -c dist/index.html dist/style.css dist/astro.js` and confirm combined HTML + CSS + JS glue is under 500 KB uncompressed (spec US1 acceptance scenario 2); log the measured sizes
- [X] T023 Run quickstart.md step-by-step validation on a clean state: follow every step in `specs/001-infrastructure-tooling-setup/quickstart.md` exactly as written and confirm it produces the expected output at each checkpoint; update quickstart.md if any step is incorrect

**Checkpoint**: All FRs satisfied, developer documentation verified

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately; T003, T004, T005 are parallel
- **Phase 2 (Foundational)**: Depends on Phase 1 complete — blocks all build invocations
- **Phase 3 (US1)**: Depends on Phase 2 complete; T007 and T008 are parallel within the phase
- **Phase 4 (US2)**: Depends on Phase 2 complete; T012, T012a, and T013 can be written before T011 finishes (different files), but T014 requires T011 + T012 + T012a + T013
- **Phase 5 (US3)**: Depends on Phase 4 complete (working WASM build); T016, T018, T019 can be drafted while T017 is in progress
- **Phase 6 (Polish)**: Depends on Phase 5 complete; T021 and T022 are parallel

### User Story Dependencies

- **US1 (P1)**: Depends on Phase 2 only — no dependency on US2 or US3
- **US2 (P2)**: Depends on Phase 2 + US1 build.sh copy step (T009) being in place (so dist/ exists)
- **US3 (P3)**: Depends on US2 complete (working WASM link with `libswe.a`)

### Parallel Opportunities

```bash
# Phase 1 — launch all three in parallel:
T003  Create .cargo/config.toml
T004  Initialize astro-wasm crate
T005  Download ephe/ data files

# Phase 3 — write HTML and CSS together:
T007  Create web/style.css
T008  Create web/index.html (markup only)

# Phase 4 — write Rust build config while C sources compile:
T012   Create astro-wasm/build.rs
T012a  Add serde_json to astro-wasm/Cargo.toml
T013   Write bridge() stub in astro-wasm/src/lib.rs

# Phase 6 — run size check and add WASM guard together:
T021  Add WebAssembly-not-supported detection
T022  Validate payload size
```

---

## Implementation Strategy

### MVP Scope (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T005)
2. Complete Phase 2: Foundational (T006)
3. Complete Phase 3: US1 (T007–T010)
4. **STOP and validate**: "Work in Progress" renders in browser — pipeline proven
5. Proceed to Phase 4 (US2) once HTML/CSS pipeline is confirmed

### Incremental Delivery

| Milestone | Tasks | Deliverable |
|-----------|-------|-------------|
| Repo skeleton ready | T001–T006 | `./build.sh` runs prereq checks |
| US1 MVP | T007–T010 | Static "Work in Progress" page in browser |
| US2 milestone | T011–T015, T012a | `libswe.a` + `astro.data` produced |
| US3 milestone | T016–T020 | Console shows sun longitude |
| Done | T021–T023 | All FRs satisfied, quickstart verified |

---

## Summary

| Metric | Count |
|--------|-------|
| Total tasks | 24 |
| Phase 1 Setup | 5 (T001–T005) |
| Phase 2 Foundational | 1 (T006) |
| US1 tasks | 4 (T007–T010) |
| US2 tasks | 6 (T011–T015 + T012a) |
| US3 tasks | 5 (T016–T020) |
| Polish tasks | 3 (T021–T023) |
| Parallelisable [P] | 9 tasks |
| MVP scope | Phase 1 + 2 + 3 (10 tasks) |
