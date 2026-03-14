---
description: "Task list for feature 003-arch-layout"
---

# Tasks: Modular Architecture Layout

**Feature**: `003-arch-layout`  
**Input**: Design documents from `/specs/003-arch-layout/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/wasm-api-v2.md ✅, quickstart.md ✅

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no blocking dependencies on incomplete tasks)
- **[US1]**: Developer can add a new engine without touching other layers (P1)
- **[US2]**: User submits date/time and language and receives localized sun position (P2)
- Exact file paths are included in every task description

---

## Phase 1: Setup (Baseline Verification)

**Purpose**: Confirm the crate is in a known-good state before any files are modified. Establishes a reference sun longitude value to verify the refactor is numerically transparent (SC-001).

- [ ] T001 Run `cargo test` in `astro-wasm/` and record the `sun_longitude` result (the exact longitude value will be the SC-001 reference); confirm all tests pass before any changes

**Checkpoint**: Clean baseline — all existing tests pass.

---

## Phase 2: Foundational — Rust Module File Scaffold

**Purpose**: Create all new Rust source files as minimal stubs so that `astro-wasm/src/lib.rs` can declare them and the crate still compiles. No functional logic is added here — that happens in Phase 3.

**⚠️ CRITICAL**: All tasks in this phase must be complete (and `cargo check` must pass) before US1 implementation tasks can begin.

- [ ] T002 Create `astro-wasm/src/swe_wrappers.rs` — paste the entire `extern "C"` FFI block (`swe_set_ephe_path`, `swe_calc_ut`) verbatim from the current `lib.rs` into this file; add `allow(dead_code)` attribute; do NOT remove it from `lib.rs` yet
- [ ] T003 [P] Create `astro-wasm/src/locales/mod.rs` with contents: `pub mod en; pub mod te;`
- [ ] T004 [P] Create `astro-wasm/src/locales/en.rs` with contents: `pub const STRINGS: &[(&str, &str)] = &[("planet.sun", "Sun")];`
- [ ] T005 [P] Create `astro-wasm/src/locales/te.rs` with contents: `pub const STRINGS: &[(&str, &str)] = &[("planet.sun", "సూర్యుడు")];`
- [ ] T006 [P] Create `astro-wasm/src/localization.rs` — stub: `pub fn get_string(_key: &str, _lang: &str) -> &'static str { "" }`
- [ ] T007 [P] Create `astro-wasm/src/utils.rs` — stubs: `pub fn iso_to_jd(_dt: &str) -> Result<f64, String> { Ok(0.0) }` and `pub fn normalize_degrees(deg: f64) -> f64 { deg % 360.0 }`
- [ ] T008 [P] Create `astro-wasm/src/engines/mod.rs` with contents: `pub mod sun;`
- [ ] T009 [P] Create `astro-wasm/src/engines/sun.rs` — define `pub struct SunEngineResult { pub label: &'static str, pub longitude: f64 }` (named `SunEngineResult` to distinguish from the JS-side `SunResult` response shape) and stub `pub fn handle_sun_longitude(_jd: f64, _lang: &str) -> Result<SunEngineResult, String> { Err("stub".into()) }`
- [ ] T010 [P] Create `astro-wasm/src/bridge.rs` — stub a private (non-exported) `fn bridge_dispatch(_op: *const u8, _inp: *const u8, _out: *mut u8, _max: i32) -> i32 { -1 }` — **no `#[no_mangle]`**; the real `#[no_mangle]` export is introduced only in T016, after the old definition is removed from `lib.rs` in T017, to prevent duplicate symbol errors
- [ ] T011 Add `mod swe_wrappers; mod utils; mod localization; mod locales; mod engines; mod bridge;` to top of `astro-wasm/src/lib.rs`; run `cargo check` and confirm zero errors (existing bridge logic still lives in lib.rs at this point)

**Checkpoint**: `cargo check` passes — all new module files are declared and the crate structure is valid.

---

## Phase 3: User Story 1 — Developer Can Add a New Engine Without Touching Other Layers (Priority: P1) 🎯 MVP

**Goal**: Implement all Rust logic in the new module files, remove old code from `lib.rs`, and validate that adding a stub second engine requires zero changes to any existing module.

**Independent Test**: `cargo test` passes; `sun_longitude` result is within 0.01° of the T001 baseline (SC-001); a stub second engine (`engines/stub.rs`) is registered in `bridge.rs` with fewer than 15 lines of new code and zero changes to `swe_wrappers.rs`, `utils.rs`, or `localization.rs` (SC-004).

### Implementation for User Story 1

- [ ] T012 [US1] Implement `utils::iso_to_jd(datetime: &str) -> Result<f64, String>` in `astro-wasm/src/utils.rs` using Meeus proleptic Gregorian calendar arithmetic (no SWE dependency); add unit test: `"2000-01-01T12:00:00Z"` → `2451545.0` exactly; add unit test for a malformed string returning `Err`
- [ ] T013 [US1] Implement `utils::normalize_degrees(deg: f64) -> f64` in `astro-wasm/src/utils.rs` *(after T012 — both functions live in the same file)* — wrap to [0.0, 360.0) using `((deg % 360.0) + 360.0) % 360.0`; add unit tests: `360.0 → 0.0`, `400.0 → 40.0`, `-10.0 → 350.0`
- [ ] T014 [P] [US1] Implement `localization::get_string(key: &str, lang: &str) -> &'static str` in `astro-wasm/src/localization.rs` — import `crate::locales::{en, te}`; linear search `STRINGS` for matching key; if `lang` unknown fall back to `en`; if key missing in target lang fall back to `en`; if key missing in `en` return `"[missing]"`
- [ ] T015 [US1] Implement `engines::sun::handle_sun_longitude(jd: f64, lang: &str) -> Result<SunEngineResult, String>` in `astro-wasm/src/engines/sun.rs` — call `swe_wrappers::calc_sun_longitude(jd)` (rename the raw FFI call to a safe wrapper), call `localization::get_string("planet.sun", lang)`, call `utils::normalize_degrees()`, return `SunEngineResult { label, longitude }`; add safe wrapper `pub fn calc_sun_longitude(jd: f64) -> Result<f64, String>` in `astro-wasm/src/swe_wrappers.rs`
- [ ] T016 [US1] Implement `bridge()` in `astro-wasm/src/bridge.rs` — deserialize `SunRequest { operation, datetime, lang }` from `input_ptr`; call `utils::iso_to_jd(&datetime)` on error return `-2`; `match operation.as_str() { "sun_longitude" => ... }` dispatch; call `engines::sun::handle_sun_longitude(jd, &lang)`; serialize `SunResponse { label, longitude }` on success; serialize `ErrorResponse { error }` on failure; return `-1` for unknown operation; **on all error return codes (-1, -2, -3), write a best-effort `ErrorResponse { error }` JSON to the output buffer before returning the code** so JS callers can display a human-readable message rather than silently failing; follow Option B retry pattern for output buffer; if `op_ptr` and the JSON `"operation"` field differ, return `-2`
- [ ] T017 [US1] Remove the old `extern "C"` FFI block, `bridge()` function, `handle_sun_longitude()` function, and `use` imports from `astro-wasm/src/lib.rs`; update `lib.rs` to contain only `mod` declarations and one `pub use bridge::bridge;` re-export; run `cargo test` and confirm sun longitude is within 0.01° of T001 baseline (SC-001)
- [ ] T018 [US1] Add `astro-wasm/src/engines/stub.rs` with a stub engine function (under 10 lines); add `pub mod stub;` to `astro-wasm/src/engines/mod.rs`; add one `match` arm `"stub_op" => { ... }` to `bridge.rs`; confirm total new code is fewer than 15 lines and zero lines changed in `swe_wrappers.rs`, `utils.rs`, or `localization.rs` (SC-004); run `cargo test` once more

**Checkpoint**: `cargo test` passes. US1 is verified. SC-001 and SC-004 satisfied.

---

## Phase 4: User Story 2 — User Submits Date/Time and Language and Receives Localized Sun Position (Priority: P2)

**Goal**: Implement the web layer changes so the full pipeline — web input → bridge → utils (ISO→JD) → engine (SWE) → localization → web output — works end-to-end. Update Playwright tests to assert localized labels.

**Independent Test**: Playwright submits `2000-01-01T12:00:00Z` with `lang=te`; result element contains `సూర్యుడు` and a numeric longitude within the 500 ms wall-clock timeout (SC-002, SC-006). Same test with `lang=en` produces `Sun` (SC-003). `web/index.html` contains no WASM boilerplate (SC-005).

**Note**: Depends on US1 completion for the compiled WASM binary with the wasm-api-v2 contract.

### Implementation for User Story 2

- [ ] T019 [US2] Create `web/astro-glue.js` — cut the entire WASM init block from `web/index.html` (Module object literal, `onRuntimeInitialized` callback, buffer helper functions `lengthBytesUTF8`, `stringToUTF8`, `_malloc`, `_free`) and paste into this file; add the `bridge(op, inputJson)` JS helper function that implements the Option B retry loop (the same helper currently inline in `index.html`)
- [ ] T020 [US2] Create `web/data.js` — implement `function getSunLongitude(isoDatetime, lang)` that calls `bridge("sun_longitude", JSON.stringify({ operation: "sun_longitude", datetime: isoDatetime, lang: lang }))` and returns `JSON.parse(result)` (a `SunResult`-shaped object); add a JSDoc comment documenting the return shape `{ label?, longitude?, error? }`
- [ ] T021 [P] [US2] Create `web/components.js` — stub file containing only a single comment: `// Reserved for reusable UI components (future features)`
- [ ] T022 [US2] Update `web/index.html` — (a) verify `web/index.html` contains no remaining WASM boilerplate (Module object, buffer helpers, bridge helper) — these were already extracted to `astro-glue.js` in T019; (b) add `<script src="astro-glue.js"></script>`, `<script src="data.js"></script>`, `<script src="components.js"></script>` before closing `</body>`; (c) replace any hardcoded invocation with a form containing: `<input type="datetime-local" id="datetime">` (with local datetime value `2000-01-01T12:00` pre-filled), a `<select id="lang">` with `<option value="en">English</option>` and `<option value="te">తెలుగు</option>`, a `<button id="submit">Calculate</button>`, and a `<div id="result"></div>`; (d) add inline `<script>` that on submit: reads the datetime-local value and converts it to a full ISO 8601 UTC string by appending `:00Z` (e.g., `const isoUtc = datetimeInput.value + ":00Z"`), disables the button, calls `getSunLongitude(isoUtc, lang)`, displays result in `#result`, then re-enables the button — the `:00Z` conversion is **required** because `<input type="datetime-local">` yields `YYYY-MM-DDTHH:MM` without timezone, but `utils::iso_to_jd` expects a full `Z`-suffixed UTC string (SC-005)
- [ ] T023 [US2] Update `tests/astro.spec.ts` — replace the console-log assertion test with: navigate to page; fill `#datetime` with `"2000-01-01T12:00"` (the form submit handler will append `:00Z` to produce `"2000-01-01T12:00:00Z"` before calling `getSunLongitude` — see T022); select `"te"` in `#lang`; click `#submit`; wait for `#submit` to be enabled (timeout 500 ms — SC-006); assert `#result` text contains `"సూర్యుడు"` and matches `/\d+\.\d+/` for numeric longitude (SC-002)
- [ ] T024 [US2] Add second Playwright test scenario in `tests/astro.spec.ts` — same steps with `lang=en`; assert `#result` text contains `"Sun"` and the same numeric longitude (SC-003)

**Checkpoint**: Playwright tests pass for both `en` and `te`. Full pipeline verified. All six success criteria satisfied.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: End-to-end build validation and documentation verification.

- [ ] T025 Run `./build.sh` from the repository root; confirm WASM artifact is produced, no compile errors, and the output file is present in `web/`
- [ ] T026 Follow the add-new-engine walkthrough in `specs/003-arch-layout/quickstart.md` step-by-step; confirm all commands succeed and the documented file paths match the implemented structure

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup; BLOCKS all user story work
- **US1 (Phase 3)**: Depends on Foundational completion
- **US2 (Phase 4)**: Depends on US1 completion (WASM binary with wasm-api-v2 contract must exist)
- **Polish (Phase 5)**: Depends on US1 + US2 completion

### User Story Dependencies

- **US1 (P1)**: No dependency on another user story — starts after Foundational
- **US2 (P2)**: Depends on US1 for the compiled WASM binary; JS file creation (T019–T021) can be authored in parallel with late US1 tasks, but integration and Playwright tests require the WASM

### Within Each User Story

- Module stubs (Phase 2) before logic implementation (Phase 3)
- `utils` and `localization` (T012–T014) before `engines/sun` (T015)
- `engines/sun` (T015) before `bridge` (T016)
- `bridge` (T016) before lib.rs cleanup (T017)
- lib.rs cleanup (T017) before stub engine validation (T018)
- `astro-glue.js` (T019) before `index.html` update (T022)
- `data.js` (T020) before Playwright tests (T023–T024)

---

## Parallel Opportunities

### Phase 2 (Foundational)

Tasks T003–T010 create independent files with no cross-dependencies and can all launch together:

```
T002 (swe_wrappers.rs — must be first, copies FFI block)
  then in parallel:
    T003 (locales/mod.rs)
    T004 (locales/en.rs)
    T005 (locales/te.rs)
    T006 (localization.rs stub)
    T007 (utils.rs stubs)
    T008 (engines/mod.rs)
    T009 (engines/sun.rs stub)
    T010 (bridge.rs stub)
  then:
    T011 (update lib.rs declarations + cargo check)
```

### Phase 3 (US1)

Once T012 (`iso_to_jd`) is complete, T013 (`normalize_degrees`) and T014 (`localization`) can run in parallel with each other:

```
T012 (iso_to_jd — serial first, needed by bridge)
  then in parallel:
    T013 (normalize_degrees)
    T014 (localization::get_string)
  then:
    T015 (engines/sun + swe_wrappers safe wrapper)
    T016 (bridge implementation)
    T017 (lib.rs cleanup + cargo test)
    T018 (stub engine SC-004 validation)
```

### Phase 4 (US2)

JS files are independent of each other; write them in parallel, then integrate:

```
In parallel:
  T019 (web/astro-glue.js)
  T020 (web/data.js)
  T021 (web/components.js)
then:
  T022 (web/index.html update)
  T023 (Playwright te test)
  T024 (Playwright en test)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete **Phase 1**: Baseline verification
2. Complete **Phase 2**: Rust module stubs (`cargo check` passes)
3. Complete **Phase 3**: US1 — full Rust implementation + stub engine proof
4. **STOP and VALIDATE**: `cargo test` passes; sun longitude within 0.01°; SC-004 verified
5. The refactored WASM is now production-ready for the Rust layer

### Incremental Delivery

1. Phase 1 → Phase 2 → Phase 3 → **WASM MVP** (module boundaries enforced, all Rust tests pass)
2. Phase 4 → **Full E2E** (web + Playwright, localized labels, 500ms bound)
3. Phase 5 → **Build & docs verified**

Each phase produces a runnable, testable increment.
