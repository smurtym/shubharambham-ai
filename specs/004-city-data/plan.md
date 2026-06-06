# Implementation Plan: City Data

**Branch**: `004-city-data` | **Date**: 2026-03-15 | **Spec**: [spec.md](spec.md)

## Summary

Expose a `list_cities` operation on the existing wasm-api-v2 bridge. The operation accepts a language code and returns a `{"cities": [...]}` JSON object containing language-filtered, sort-ordered city objects. All city data (names, translations, sort keys, timezones) is hardcoded in a Rust static array compiled into the WASM binary — no backend, no network. The bridge function signature is unchanged; only a new dispatch arm and a new `data` module are added.

## Technical Context

**Language/Version**: Rust 2021 (stable toolchain), compiled to WASM via Emscripten (`wasm32-unknown-emscripten`)  
**Primary Dependencies**: `serde 1` + `serde_json 1` (alloc features — already in `Cargo.toml`); no new crates required  
**Storage**: Static `&'static [CityRecord]` array compiled into the WASM binary, generated at build time from `data/cities.csv`  
**City Data Source**: `data/cities.csv` — human-editable, one row per city+language, no Rust knowledge needed  
**Code Generation**: `build.rs` reads `data/cities.csv` → writes `OUT_DIR/cities_generated.rs` → included by `cities.rs` via `include!`  
**Testing**: `cargo test` for Rust unit + data-integrity tests; Playwright for JS E2E  
**Target Platform**: WASM (`wasm32-unknown-emscripten`) + static HTML/vanilla JS  
**Project Type**: Library (WASM module) — additive extension of existing bridge  
**Performance Goals**: Imperceptible — pure in-memory filter + sort over ≤ hundreds of structs; no latency target needed  
**Constraints**: Binary size — no new crates; all translations compiled into binary  
**Scale/Scope**: 5-city seed; architecture supports hundreds of cities

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Rust-First Computation | ✅ PASS | All filtering, sorting, and serialization in Rust; JS layer only calls bridge and renders strings |
| II. Lean Web Presentation | ✅ PASS | One new function added to `web/data.js`; no new libraries or frameworks |
| III. Contract-Driven WASM API | ✅ PASS | `contracts/city-data-v1.md` created in Phase 1 before any implementation task starts |
| IV. Mobile-First Design | ✅ N/A | Feature adds no new UI components |
| V. Correctness & Accuracy | ✅ N/A | No astronomical calculation involved |
| VI. Localization in Rust | ✅ PASS | All city name translations compiled into WASM binary; `canonicalName` always English; JS receives ready-to-render strings |

**Gate result: PASS — no violations. Proceeding to Phase 0.**

## Project Structure

### Documentation (this feature)

```text
specs/004-city-data/
├── plan.md              ← this file
├── research.md          ← Phase 0 output
├── data-model.md        ← Phase 1 output
├── quickstart.md        ← Phase 1 output
├── contracts/
│   └── city-data-v1.md  ← Phase 1 output
└── tasks.md             ← Phase 2 output (/speckit.tasks — NOT created by /speckit.plan)
```

### Source Code

```text
data/
└── cities.csv           NEW — single source of truth for all city data (human-editable)

astro-wasm/
├── build.rs             MODIFY — add generate_cities() that reads cities.csv → OUT_DIR/cities_generated.rs
└── src/
    ├── lib.rs            MODIFY — add `pub mod data;`
    ├── bridge.rs         MODIFY — add `list_cities` dispatch arm
    └── data/             NEW MODULE
        ├── mod.rs        — CityRecord + TranslationEntry structs; list_cities() fn; decode_city_id()
        └── cities.rs     — two lines: use super structs + include!(cities_generated.rs)

web/
└── data.js              MODIFY — add listCities(lang) function (mirrors getSunLongitude pattern)
```

**Structure Decision**: Single-project extension. City data lives in `data/cities.csv` — editable by anyone without Rust knowledge. `build.rs` generates `cities.rs` content at compile time using only `std` (no new build-dependencies). The existing `astro-wasm` crate gains one new module (`data/`) with two files. The bridge gains one new match arm. The web data layer gains one new function following the established `getSunLongitude` pattern. No new projects, no new crates.
