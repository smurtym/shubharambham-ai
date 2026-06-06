# Implementation Plan: Modular Architecture Layout

**Branch**: `003-arch-layout` | **Date**: 2026-03-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-arch-layout/spec.md`

## Summary

Refactor the monolithic `astro-wasm/src/lib.rs` into five explicit Rust modules (`swe_wrappers`, `utils`, `engines/sun`, `localization`, `bridge`) and split the inline WASM boilerplate in `web/index.html` into dedicated JS files (`astro-glue.js`, `data.js`, `components.js`). The only computation is the existing `sun_longitude` operation, now accepting an ISO 8601 UTC datetime and a `lang` identifier, with ISO→JD conversion in pure Rust (`utils`) and localized labels for `en`/`te` returned from Rust (`localization`). The API contract is named `wasm-api-v2`, fully replacing the retired `wasm-api-v1`. Success is measured by `cargo test` passing and Playwright asserting `సూర్యుడు` (te) and `Sun` (en) labels in the result element.

## Technical Context

**Language/Version**: Rust stable (`cdylib`) compiled via Emscripten; vanilla ES6+ JS  
**Primary Dependencies**: `serde_json 1.x` (alloc feature), `cc 1.x` (build), Swiss Ephemeris C library (FFI via `build.rs`), Emscripten toolchain  
**Storage**: N/A — ephemeris data files embedded in Emscripten virtual filesystem at `/ephe`  
**Testing**: `cargo test` (Rust unit/integration); Playwright (E2E browser)  
**Target Platform**: WASM (Emscripten/browser); static file hosting  
**Project Type**: WASM library + static web UI  
**Performance Goals**: Bridge round-trip (submit → result displayed) < 500 ms (starting point)  
**Constraints**: Pure Rust ISO→JD conversion (no `swe_utc_to_jd`); no JS frameworks; no external JS libraries; localization entirely in Rust  
**Scale/Scope**: Single WASM export (`bridge`); 1 engine (`sun`); 2 locales (`en`, `te`)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| # | Principle | Status | Notes |
|---|-----------|--------|-------|
| I | Rust-First Computation | ✅ PASS | All calculation, ISO→JD, and localization logic in Rust; JS only calls bridge and renders |
| II | Lean Web Presentation | ✅ PASS | Only HTML + CSS + vanilla JS; no frameworks or external libraries added |
| III | Contract-Driven WASM API | ✅ PASS | `wasm-api-v2` defined in `contracts/wasm-api-v2.md` before implementation; fully replaces v1 |
| IV | Mobile-First Design | ✅ PASS | Existing responsive viewport preserved; datetime + language inputs are native HTML elements |
| V | Correctness & Accuracy | ✅ PASS | SC-001 requires `sun_longitude` unit test to produce same value within 0.01°; existing test retained |
| VI | Localization in Rust | ✅ PASS | `localization` module in Rust returns pre-rendered strings; JS passes only `lang` identifier |

**Gate result: PASS — no violations. Proceed to Phase 0.**

## Project Structure

### Documentation (this feature)

```text
specs/003-arch-layout/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── wasm-api-v2.md   # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created here)
```

### Source Code (repository root)

```text
astro-wasm/
└── src/
    ├── lib.rs                  # crate root — declares modules, re-exports bridge
    ├── swe_wrappers.rs         # extern "C" FFI declarations only
    ├── utils.rs                # iso_to_jd(), normalize_degrees()
    ├── localization.rs         # get_string(key, lang) — lookup logic only
    ├── bridge.rs               # bridge() export, JSON parse/dispatch/serialize
    └── engines/
        ├── mod.rs              # pub mod sun
        └── sun.rs              # handle_sun_longitude(jd, lang) → SunResult

astro-wasm/src/locales/         # translator-editable data files; no logic
    ├── mod.rs                  # pub mod en; pub mod te;
    ├── en.rs                   # English strings: STRINGS: &[(&str, &str)]
    └── te.rs                   # Telugu strings

web/
├── index.html          # datetime input, lang selector, result element
├── style.css           # unchanged
├── astro-glue.js       # WASM init, buffer management, bridge() JS helper
├── data.js             # getSunLongitude(isoDatetime, lang) → Promise<SunResult>
└── components.js       # stub (placeholder comment only in this feature)

tests/
└── astro.spec.ts       # updated: submits ISO datetime + lang, asserts label + longitude
```

**Structure Decision**: Single project. The WASM crate uses Rust's module system with sub-files under `src/`. The web layer is flat files under `web/`. No new top-level directories.

## Complexity Tracking

No constitution violations. No entries required.

## Post-Phase-1 Constitution Check

*Re-evaluated after data-model.md and contracts/wasm-api-v2.md are complete.*

| # | Principle | Status | Notes |
|---|-----------|--------|-------|
| I | Rust-First Computation | ✅ PASS | ISO→JD in `utils.rs` (pure Rust, Meeus algorithm); localization in `localization.rs`; no JS logic |
| II | Lean Web Presentation | ✅ PASS | `astro-glue.js`, `data.js`, `components.js` are plain JS; no new libraries introduced |
| III | Contract-Driven WASM API | ✅ PASS | `contracts/wasm-api-v2.md` fully specifies request/response shapes, error shape, reference values; contract exists before implementation |
| IV | Mobile-First Design | ✅ PASS | `<input type="datetime-local">` and `<select>` are native HTML — touch-friendly; no JS layout logic |
| V | Correctness & Accuracy | ✅ PASS | `utils::iso_to_jd` unit test uses J2000.0 reference (2451545.0 exactly); `sun_longitude` test within 0.01° |
| VI | Localization in Rust | ✅ PASS | `localization::get_label()` returns `&'static str` compiled into WASM; no runtime fetch; both `en` and `te` provided |

**Post-Phase-1 gate result: PASS — design is constitution-compliant.**

## Key Design Decisions

| Decision | Choice | Reference |
|----------|--------|-----------|
| ISO→JD algorithm | Meeus proleptic Gregorian calendar arithmetic, pure Rust | [research.md](research.md#1-iso-8601-utc--julian-day-number-pure-rust) |
| Module layout | File-per-module; `#[no_mangle]` on `bridge()` definition in `bridge.rs` | [research.md](research.md#2-rust-module-layout-for-a-cdylib-crate) |
| Localization pattern | Separate data files per locale (`locales/en.rs`, `locales/te.rs`) + lookup logic in `localization.rs`; translators edit data files only; new language = one new file + two lines | [research.md](research.md#3-localization-pattern--separate-data-files-per-locale) |
| JS file split | Plain `<script>` with global scope; no ES modules (avoids Emscripten build flag change) | [research.md](research.md#4-js-module-strategy-for-astro-gluejs--datajs--componentsjs) |
| Error response shape | Single `SunResult` shape: `{ label?, longitude?, error? }` | [spec.md Clarification Q1](spec.md) |
| Loading state | Disable submit button during call; Playwright waits for re-enable | [spec.md Clarification Q2](spec.md) |
| Contract versioning | Spec-time only — named `wasm-api-v2`; no runtime version field | [spec.md Clarification Q3](spec.md) |
| Latency target | 500 ms round-trip (starting point) | [spec.md SC-006](spec.md) |
