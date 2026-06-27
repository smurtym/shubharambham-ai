# Tasks: Runtime City CSV

**Branch**: `012-runtime-city-csv` | **Date**: 2026-06-27

## Phase 1 — Move data file

- [X] T001: Copy `data/cities.csv` → `ephe/cities.csv`
- [X] T002: Delete `data/cities.csv`

## Phase 2 — Remove build-time codegen

- [X] T003: Remove `generate_cities()` function from `astro-wasm/build.rs`
- [X] T004: Remove `escape_str()` helper from `astro-wasm/build.rs`
- [X] T005: Remove `generate_cities()` call from `main()` in `build.rs`

## Phase 3 — Runtime loading in data/mod.rs

- [X] T006: Change `CityRecord` fields from `&'static str` to `String`
- [X] T007: Change `TranslationEntry` fields from `&'static str` to `String`
- [X] T008: Remove `pub mod cities;` from `data/mod.rs`
- [X] T009: Remove `validate_canonical_names()` and `const _: ()` invocation
- [X] T010: Add `fn cities_csv_path() -> &'static str` (cfg-gated path)
- [X] T011: Add `fn load_cities_from_csv() -> Result<Vec<CityRecord>, String>`
- [X] T012: Add `static CITY_CACHE: OnceLock<Vec<CityRecord>>`
- [X] T013: Add `pub fn cities() -> Result<&'static [CityRecord], String>`
- [X] T014: Update `list_cities()` to call `cities()` instead of `cities::CITIES`

## Phase 4 — Delete generated file

- [X] T015: Delete `astro-wasm/src/data/cities.rs`

## Phase 5 — Update callers

- [X] T016: Update `engines/horoscope.rs`: replace `cities::CITIES` with `data::cities()`
- [X] T017: Update `engines/vimsottari.rs`: replace `cities::CITIES` with `data::cities()`
- [X] T018: Update `src/bridge.rs` test code: replace `data::cities::CITIES` with `data::cities()`

## Phase 6 — Update tests

- [X] T019: Update `data/mod.rs` tests: `cities::CITIES` → `cities().expect(...)`
- [X] T020: Fix `test_timezones_valid`: `VALID_TIMEZONES.contains(&rec.timezone)` → `.contains(&rec.timezone.as_str())`
- [X] T021: Fix `test_list_cities_te_excludes_en_only`: `canonical_names.contains(&rec.canonical_name)` → `.contains(&rec.canonical_name.as_str())`

## Phase 7 — Documentation

- [X] T022: Update `CLAUDE.md` architecture description

## Phase 8 — Validation

- [X] T023: Run `cargo test` — all 45 tests pass
