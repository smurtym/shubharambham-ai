---
description: "Task list for feature 004-city-data"
---

# Tasks: City Data

**Feature**: `004-city-data`  
**Input**: Design documents from `/specs/004-city-data/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/city-data-v1.md ✅, quickstart.md ✅

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no blocking dependencies on incomplete tasks)
- **[US1]**: Web app fetches a translated city list for language selection (P1)
- **[US2]**: Cities without a translation for the requested language are hidden (P2)
- **[US3]**: Developer can identify any city by its canonical English name (P3)
- Exact file paths are included in every task description

---

## Phase 1: Setup (Baseline Verification)

**Purpose**: Confirm the crate builds and all existing tests pass before any files are touched. Establishes a clean reference point.

- [X] T001 Run `cargo test` in `astro-wasm/` and confirm all existing tests pass before any new files are created; record pass/fail count as the pre-change baseline

**Checkpoint**: Clean baseline — all existing tests green.

---

## Phase 2: Foundational — data/ Module Scaffold

**Purpose**: Create the two new Rust source files as minimal compilable stubs and wire them into `lib.rs`. No functional logic yet. **All user story work is blocked until `cargo check` passes at the end of this phase.**

**⚠️ CRITICAL**: Complete T002 → T003 → T004 in order before starting any Phase 3 task.

- [X] T002 Create `astro-wasm/src/data/mod.rs` with stub struct definitions — `pub struct CityRecord`, `pub struct TranslationEntry` (both with empty bodies using `todo!()`-free placeholder fields), and a stub `pub fn list_cities(_lang: &str) -> String { String::new() }`; add `pub mod cities;` declaration at top of file
- [X] T003 Create `astro-wasm/src/data/cities.rs` with `use super::{CityRecord, TranslationEntry};` and `pub static CITIES: &[CityRecord] = &[];` (empty array — populated in T007)
- [X] T004 Add `pub mod data;` to `astro-wasm/src/lib.rs` immediately after the existing module declarations; run `cargo check` in `astro-wasm/` and confirm zero errors

**Checkpoint**: `cargo check` passes — `data` module is declared, both files compile, existing functionality unchanged.

---

## Phase 3: User Story 1 — Web App Fetches a Translated City List (Priority: P1) 🎯 MVP

**Goal**: A fully working `list_cities` bridge operation that accepts a language code and returns `{"cities": [...]}` with all seven fields per city, using the 4-city seed dataset.

**Independent Test**: Call `list_cities("en")` via the bridge, parse the result, assert the `cities` array has 4 entries each with `lang`, `cityId`, `timeZone`, `canonicalName`, `cityName`, `region1`, `region2` all populated. `cargo test` must pass. Separately, verify `listCities("te")` from JS returns an array with at least one Telugu-named city.

### Implementation for User Story 1

- [X] T005 [US1] Replace stub structs in `astro-wasm/src/data/mod.rs` with complete definitions — `CityRecord { city_id: u16, canonical_name: &'static str, timezone: &'static str, translations: &'static [(&'static str, TranslationEntry)] }` and `TranslationEntry { city_name: &'static str, region1: &'static str, region2: &'static str, region1_order: u16, region2_order: u16 }`
- [X] T006 [US1] Add `CityResponse`, `CityListResponse`, and `decode_city_id` to `astro-wasm/src/data/mod.rs`:
  - `pub fn decode_city_id(city_id: u16) -> (f64, f64)` — decompose quadkey (`tile_x = city_id / 128`, `tile_y = city_id % 128`), compute `lng = (tile_x as f64 + 0.5) / 128.0 * 360.0 - 180.0`, compute `lat` via `n = PI * (1.0 - 2.0 * (tile_y as f64 + 0.5) / 128.0)` then `n.sinh().atan() * 180.0 / PI`; returns `(lat, lng)` in decimal degrees
  - `CityResponse` derives `serde::Serialize` with `#[serde(rename_all = "camelCase")]`; fields: `lang: String`, `city_id: u16`, `time_zone: String`, `canonical_name: String`, `city_name: String`, `region1: String`, `region2: String`, `lat: f64`, `lng: f64`; mark `region1_order: u16` and `region2_order: u16` with `#[serde(skip)]`
  - `CityListResponse { cities: Vec<CityResponse> }` also derives `serde::Serialize`
- [X] T007 [P] [US1] Hardcode the 5-city seed array in `astro-wasm/src/data/cities.rs` — use precomputed zoom-level-7 `city_id` values (see data-model.md): Hyderabad=11705, Vijayawada=11834, Eluru=11833, Delhi=11701, New York=4784; add `te` + `en` TranslationEntry for Hyderabad (te: `region2_order=1, region1_order=1`; en: `region2_order=1, region1_order=2`), Vijayawada (te: `region2_order=1, region1_order=2`; en: `region2_order=1, region1_order=3`), and Eluru (te: `city_name="ఏలూరు", region1="ఆంధ్రప్రదేశ్", region2="భారతదేశం", region2_order=1, region1_order=2`; en: `city_name="Eluru", region1="Andhra Pradesh", region2="India", region2_order=1, region1_order=3`); add `en`-only entry for Delhi (`region2_order=1, region1_order=1`) and New York (`region2_order=2, region1_order=1`)
- [X] T008 [US1] Replace the stub `list_cities` body in `astro-wasm/src/data/mod.rs` — iterate `cities::CITIES`, filter to records that have a translation entry for `lang`, map each matched record + translation into a `CityResponse` (call `decode_city_id(rec.city_id)` to populate `lat`/`lng`), collect into `Vec`, sort by `(region2_order, region1_order, city_name)` ascending, wrap in `CityListResponse`, serialise with `serde_json::to_string` and return the JSON string (or an error string `{"error":"..."}` on serialisation failure)
- [X] T009 [P] [US1] Add `listCities(lang)` function to `web/data.js` — follow the `getSunLongitude` pattern: call `bridge('list_cities', JSON.stringify({ operation: 'list_cities', lang }))` (synchronous, no `async`/`await`; `bridge` is defined in `astro-glue.js`), throw `new Error(response.error)` if the `error` field is present, otherwise return `response.cities`
- [X] T010 [US1] Add `"list_cities"` dispatch arm to `astro-wasm/src/bridge.rs` — define `CitiesRequest { operation: String, lang: String }`, deserialise `input` into `CitiesRequest` (-2 on parse error), verify `req.operation == op` (-2 on mismatch), call `data::list_cities(&req.lang)`, call `write_json` with the result string; place the new arm before the `_ =>` catch-all

### Tests for User Story 1

- [X] T011 [P] [US1] Add unit tests in `astro-wasm/src/data/mod.rs`:
  - `test_list_cities_en_returns_all_five` — call `list_cities("en")`, parse JSON, assert `cities` length = 5, assert every entry has all 9 required fields non-empty (including `lat` and `lng`)
  - `test_list_cities_te_has_three_entries` — call `list_cities("te")`, assert length = 3, assert every entry's `cityName` contains Telugu script (code point > U+0C00)
  - `test_list_cities_te_order` — call `list_cities("te")`, assert `cities[0]["canonicalName"] == "Hyderabad"` (region1_order=1), `cities[1]["canonicalName"] == "Eluru"` (region1_order=2, city_name "\u0C0F...") and `cities[2]["canonicalName"] == "Vijayawada"` (region1_order=2, city_name "\u0C35...") — verifies both region-order and city-name tie-break in one assertion
  - `test_list_cities_en_order` — call `list_cities("en")`, assert `cities[0]["canonicalName"] == "Delhi"`, `cities[2]["canonicalName"] == "Eluru"`, `cities[3]["canonicalName"] == "Vijayawada"`, `cities[4]["canonicalName"] == "New York"` — verifies name-sort tie-break for Andhra Pradesh pair
  - `test_decode_city_id` — assert `decode_city_id(11705)` returns `lat` within 0.1° of 17.97 and `lng` within 0.1° of 77.34 (Hyderabad tile centre)
- [X] T012 [P] [US1] Add bridge integration test in `astro-wasm/src/bridge.rs`: `test_bridge_list_cities_en` — call `bridge` with `op_ptr = "list_cities"` and `input = {"operation":"list_cities","lang":"en"}`, assert return value > 0, parse output as JSON, assert top-level key is `"cities"` and array length = 5

**Checkpoint**: `cargo test` passes. `list_cities("en")` returns all 5 cities in order (Delhi, Hyderabad, Eluru, Vijayawada, New York). `list_cities("te")` returns Hyderabad, Eluru, Vijayawada in that order with Telugu names. Bridge dispatch works end-to-end. `web/data.js` exposes `listCities(lang)`. User Story 1 is independently shippable.

---

## Phase 4: User Story 2 — Language Filtering (Priority: P2)

**Goal**: Confirm that cities without a translation for the requested language are silently excluded, and that an unknown language code returns an empty array rather than an error.

**Independent Test**: Call `list_cities("te")` — assert Delhi and New York are absent by `canonicalName`; assert length = 3. Call `list_cities("hi")` — assert `cities` is an empty array and no `error` field is present.

### Tests for User Story 2

- [X] T013 [US2] Add unit tests in `astro-wasm/src/data/mod.rs`: `test_list_cities_te_excludes_en_only` — call `list_cities("te")`, parse, assert no entry has `canonicalName` equal to `"Delhi"` or `"New York"`, assert length = 3 and all three entries are `"Hyderabad"`, `"Eluru"`, `"Vijayawada"`; `test_list_cities_unknown_lang_returns_empty` — call `list_cities("hi")`, parse, assert `cities` is an empty array `[]`
- [X] T014 [P] [US2] Add bridge integration test in `astro-wasm/src/bridge.rs`: `test_bridge_list_cities_te_filters` — bridge call with `lang="te"`, parse response, assert `cities` length = 3, collect `canonicalName` values into a set, assert set contains exactly `{"Hyderabad", "Eluru", "Vijayawada"}` and does not contain `"Delhi"` or `"New York"`

**Checkpoint**: `cargo test` passes. Language filtering is verified — en-only cities absent from te response, unknown languages return empty array.

---

## Phase 5: User Story 3 — Canonical English Name (Priority: P3)

**Goal**: Confirm that `canonicalName` is always in English regardless of the requested language, and matches `cityName` when the request language is English.

**Independent Test**: Call `list_cities("te")`, assert every `canonicalName` value is non-empty and consists of only Latin-script characters. Call `list_cities("en")`, assert `canonicalName == cityName` for all entries.

### Tests for User Story 3

- [X] T015 [P] [US3] Add unit tests in `astro-wasm/src/data/mod.rs`: `test_canonical_name_always_english` — call `list_cities("te")`, assert each entry's `canonical_name` is one of `["Hyderabad", "Eluru", "Vijayawada"]` (exact English names); `test_en_canonical_matches_city_name` — call `list_cities("en")`, assert `canonical_name == city_name` for every entry
- [X] T016 [P] [US3] Add bridge integration test in `astro-wasm/src/bridge.rs`: `test_bridge_canonical_name_invariant` — bridge call with `lang="te"`, parse response, assert `cities[0].canonicalName` is `"Hyderabad"`, `cities[1].canonicalName` is `"Eluru"`, and `cities[2].canonicalName` is `"Vijayawada"` — all English regardless of `te` locale

**Checkpoint**: `cargo test` passes. `canonicalName` invariant verified at both unit and bridge integration level.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Build-time data integrity validation, wire-format purity test, and E2E browser smoke test.

- [X] T017 [P] Add compile-time `canonicalName` validation in `astro-wasm/src/data/mod.rs` — implement `const fn validate_canonical_names()` that loops over `cities::CITIES` and panics (compile error) if any `canonical_name.is_empty()`; add `const _: () = validate_canonical_names();` at module level so the check runs at compile time on every `cargo build`
- [X] T018 [P] Add data integrity tests in `astro-wasm/src/data/mod.rs`: `test_city_ids_unique` — insert all `city_id` values into a `std::collections::HashSet`, assert `set.len() == CITIES.len()`; `test_timezones_valid` — define `const VALID_TIMEZONES: &[&str] = &["Asia/Kolkata", "America/New_York"];` and assert every city's `timezone` is in it
- [X] T019 [P] Add wire format purity test in `astro-wasm/src/data/mod.rs`: `test_sort_keys_absent_from_json` — call `list_cities("en")`, assert the raw JSON string does NOT contain the substrings `"region1Order"`, `"region2Order"`, `"region1_order"`, or `"region2_order"`
- [X] T020 Update `tests/astro.spec.ts` — add a Playwright test case that calls `listCities("te")` via the bridge in the browser, asserts the returned array contains an entry with `cityName` equal to `"హైదరాబాద్"`, asserts no entry has `cityName` equal to `"Delhi"`, and asserts every entry's `canonicalName` is ASCII-only (all char codes ≤ 127)
- [X] T021 [P] Add bridge integration test for malformed `list_cities` request in `astro-wasm/src/bridge.rs`: `test_bridge_list_cities_missing_lang` — call `bridge` with `op_ptr = "list_cities"` and `input = {"operation":"list_cities"}` (no `lang` field), assert return value = `-2`, parse output as JSON, assert response contains an `"error"` key and no `"cities"` key (verifies FR-012 missing-`lang` path)

**Checkpoint**: `cargo test` passes including all data integrity and wire-format tests. Playwright smoke test passes. Build fails correctly if a city with an empty `canonicalName` is added to the data array. Bridge returns `-2` for requests missing `lang` field.

---

## Dependencies

```
T001 (baseline)
  └─► T002 (data/mod.rs stubs)
        └─► T003 (data/cities.rs stub)
              └─► T004 (lib.rs + cargo check)
                    └─► T005 (US1: full struct definitions)
                          ├─► T006 (US1: list_cities() implementation)
                          │     └─► T008 (US1: bridge dispatch arm)
                          │           ├─► T010 (US1: bridge test)
                          │           ├─► T013 (US2: bridge filter test)
                          │           └─► T016 (US3: bridge canonical test)
                          └─► T007 [P] (US1: CITIES array — parallel with T006)
                                └─► T011 (US1: unit tests — needs T006+T007)
                                      └─► T012 (US1: filtering tests)
                                            └─► T015 (US3: canonical unit tests)

T009 [P] (web/data.js) — no Rust dependency; can start after T004
T014 [P], T017 [P], T018 [P], T019 [P] — can start after their phase prerequisites
T020 — after all Rust tests pass and WASM build succeeds
T021 [P] — can run in parallel with T020 (Rust-only test, no WASM build needed)
```

## Parallel Execution Examples

**After T004 (cargo check clean), these tasks can proceed in parallel**:
- T005 (struct definitions in `data/mod.rs`) + T009 (web/data.js `listCities`)

**After T005 completes, these can proceed in parallel**:
- T006 (implement `list_cities()` in `data/mod.rs`) + T007 (populate `CITIES` in `data/cities.rs`)

**After T006 + T007 complete, these can proceed in parallel**:
- T008 (bridge dispatch) + T011 (unit tests)

**After T010 (bridge dispatch arm) completes, these test tasks can proceed in parallel**:
- T012 (US1 bridge test) + T013 (US2 unit filter tests) + T016 (US3 bridge canonical test)

**After Phase 5 complete, these polish tasks can proceed in parallel**:
- T017 (const validation) + T018 (data integrity tests) + T019 (wire format purity test)

## Implementation Strategy

**MVP = Phase 3 only** (T005–T012). Completing Phase 3 gives a working `list_cities("en")` bridge call with all 4 seed cities and a verifiable `listCities(lang)` in JS. Phases 4 and 5 add targeted test coverage for filtering and canonical name properties that are already implemented in Phase 3 — they verify correctness rather than add new behaviour.

**Recommended execution order**:
1. T001 — verify baseline
2. T002 → T003 → T004 — scaffold (sequential, ~15 min)
3. T005 → (T006 ‖ T007) → T008 → T009 — core implementation (T009 anytime after T004)
4. T010 ‖ T011 — US1 tests
5. T012 ‖ T013 ‖ T014 — US2 tests (can mostly reuse bridge test infrastructure from T010)
6. T015 ‖ T016 — US3 tests
7. T017 ‖ T018 ‖ T019 — polish (all parallel)
8. T020 — E2E Playwright

## Task Count Summary

| Phase | Tasks | User Story |
|-------|-------|------------|
| Phase 1: Setup | 1 (T001) | — |
| Phase 2: Foundational | 3 (T002–T004) | — |
| Phase 3: US1 impl + tests | 8 (T005–T012) | US1 (P1) |
| Phase 4: US2 tests | 2 (T013–T014) | US2 (P2) |
| Phase 5: US3 tests | 2 (T015–T016) | US3 (P3) |
| Phase 6: Polish | 5 (T017–T021) | — |
| **Total** | **21** | |

| User Story | Implementation tasks | Test tasks | Parallel opportunities |
|---|---|---|---|
| US1 | T005, T006, T007, T008, T009, T010 | T011, T012 | T007‖T006, T009 anytime, T011‖T012 |
| US2 | *(implemented in US1 `list_cities()`)* | T013, T014 | T013‖T014 |
| US3 | *(implemented in US1 struct layout)* | T015, T016 | T015‖T016 |
