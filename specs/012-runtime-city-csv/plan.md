# Implementation Plan: Runtime City CSV

**Branch**: `012-runtime-city-csv` | **Date**: 2026-06-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/012-runtime-city-csv/spec.md`

## Summary

Replace build-time city code generation with runtime CSV parsing. Move `data/cities.csv` to `ephe/cities.csv` (where it is automatically included in the Emscripten virtual FS preload), remove `generate_cities()` from `build.rs`, delete `data/cities.rs`, and load city data on first use via a lazy-initialized cache. The Rust data structures change from `&'static str` fields to owned `String` fields. The WASM API contract and response shape are unchanged.

## Technical Context

**Language/Version**: Rust (stable toolchain), dual-target: native `cargo test` + `wasm32-unknown-emscripten`

**Primary Dependencies**: `serde`, `serde_json`, `std::sync::OnceLock` (std — no new crates)

**Storage**: `ephe/cities.csv` — co-located with ephemeris data, preloaded into WASM virtual FS by the existing Emscripten `--preload-file ephe/@/ephe/` argument in `build.rs`

**Testing**: `cargo test` in `astro-wasm/` — native build reads `../ephe/cities.csv`

**Target Platform**: WASM (Emscripten) + native

**Performance Goals**: CSV parse happens once on first request; subsequent requests use the in-memory cache — imperceptible latency for a ~160-row file

**Constraints**:
- No new Cargo dependencies
- WASM API contract `city-data-v1` unchanged (same request/response shapes, same error codes)
- `ephe/cities.csv` must be present at both `ephe/cities.csv` (WASM) and `../ephe/cities.csv` (native test path)

**Scale/Scope**: ~160 CSV rows; 7 files changed/deleted, 1 file moved

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Rust-First Computation | PASS | All CSV parsing remains in Rust; zero JS changes |
| II. React + TypeScript | N/A | No frontend changes |
| III. Contract-Driven WASM API | PASS | `city-data-v1` v1.0.0 unchanged — same operation, request, response, error codes |
| IV. Mobile-First Design | N/A | No UI changes |
| V. Correctness & Accuracy | PASS | Runtime-loaded data produces identical output to compiled-in data; verified by existing tests |
| VI. Localization in Rust | PASS | No localization changes |

**No violations. No Complexity Tracking entry needed.**

## Key Architectural Decisions

### 1. Cache strategy: `std::sync::OnceLock<Vec<CityRecord>>`

A module-level `OnceLock<Vec<CityRecord>>` in `data/mod.rs` is initialized on the first call to any city-needing operation and reused for all subsequent calls within the same WASM execution context (or native process).

```
static CITY_CACHE: OnceLock<Vec<CityRecord>> = OnceLock::new();

pub fn cities() -> Result<&'static [CityRecord], String>
```

All three callers (`dispatch_list_cities` / `list_cities`, horoscope engine, vimsottari engine) call `data::cities()` instead of `cities::CITIES`. On CSV read failure the `Result::Err` propagates to the caller as a structured error response.

**Alternatives rejected:**
- Re-read CSV on every call: wastes I/O; rejected
- `lazy_static` / `once_cell` crate: `std::sync::OnceLock` is stable since Rust 1.70 with no new dependency; preferred
- `static mut`: unsafe; rejected

### 2. Data type change: `&'static str` → `String`

`CityRecord` and `TranslationEntry` currently use `&'static str` because the data is compiled in. With runtime loading, fields become `String`. Callers that already call `.to_owned()` are unaffected.

### 3. Compile-time `const fn` validation removal

`const _: () = validate_canonical_names();` in `data/mod.rs` evaluates at compile time using the static `CITIES` array. After removal of the static array, this is deleted. The equivalent check moves into `load_cities_from_csv()` — any city with an empty `canonical_name` is skipped (or causes a load error — implementation choice). The existing `test_city_ids_unique` and `test_canonical_name_always_english` tests continue to provide runtime coverage.

### 4. Tests that reference `cities::CITIES` directly

Tests in `data/mod.rs` and `bridge::tests` that iterate `cities::CITIES` are updated to call `data::cities().unwrap()` (or `super::cities().unwrap()` inside the data module). Native tests read from `../ephe/cities.csv`.

### 5. Malformed row handling

Malformed CSV rows (wrong column count, unparseable integers) are **skipped silently**. An unreadable CSV file (not found, I/O error) returns `Err(String)` from `data::cities()`, which propagates to the bridge as a `-3` error response (per FR-008).

## Project Structure

### Documentation (this feature)

```text
specs/012-runtime-city-csv/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── checklists/
│   └── requirements.md
└── tasks.md             # /speckit-tasks output
```

### Source Code Changes

```text
ephe/
├── semo_18.se1          ← unchanged
├── sepl_18.se1          ← unchanged
└── cities.csv           ← NEW (moved from data/cities.csv)

data/
└── cities.csv           ← DELETED (moved to ephe/)

astro-wasm/
├── build.rs             ← MODIFY: remove generate_cities() + escape_str(); remove cities.csv rerun-if-changed
└── src/
    └── data/
        ├── mod.rs       ← MODIFY: String fields; OnceLock cache; load_cities_from_csv(); update list_cities(); remove pub mod cities; remove const validate
        └── cities.rs    ← DELETE

    └── engines/
        ├── horoscope.rs ← MODIFY: cities::CITIES.iter() → data::cities()?.iter()
        └── vimsottari.rs ← MODIFY: same
```

## Complexity Tracking

No Constitution Check violations.
