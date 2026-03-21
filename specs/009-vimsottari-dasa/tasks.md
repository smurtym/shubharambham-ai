# Tasks: Vimsottari Dasa

**Feature**: `009-vimsottari-dasa`
**Input**: Design documents from `specs/009-vimsottari-dasa/`
**Branch**: `009-vimsottari-dasa`
**Generated**: 2026-03-21

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, independent of in-progress tasks)
- **[US1/US2]**: User story this task belongs to
- All paths are relative to the repository root

---

## Phase 1: Setup

**Purpose**: Register the new engine module. No computation logic yet.

- [ ] T001 Register `pub mod vimsottari;` in `astro-wasm/src/engines/mod.rs`
- [ ] T002 Create skeleton `astro-wasm/src/engines/vimsottari.rs` with `pub fn execute(request: &str) -> String` that returns `{"error":"not implemented"}` — file compiles, no logic yet

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add localization keys that both user stories depend on. Must be complete before Phase 3.

**⚠️ No user story work can begin until this phase is complete.**

- [ ] T003 [P] Add 14 English locale keys to `astro-wasm/src/locales/en.rs`: `dasa.maha` ("Mahadasa"), `dasa.antar` ("Antardasa"), `month.1` ("January") through `month.12` ("December") — append after existing 71 entries; update compile-time assertion from 71 to 85
- [ ] T004 [P] Add 14 Telugu locale keys to `astro-wasm/src/locales/te.rs`: `dasa.maha` ("మహాదశ"), `dasa.antar` ("అంతర్దశ"), `month.1` ("జనవరి") through `month.12` ("డిసెంబర్") — append after existing 71 entries; update compile-time assertion from 71 to 85
- [ ] T005 Add `"vimsottari_dasa"` dispatch arm in `astro-wasm/src/bridge.rs` — route to `engines::vimsottari::execute(input)`, write result to output buffer, return byte count; add before the `_ =>` fallback arm

**Checkpoint**: `cargo check` passes. Bridge routes `"vimsottari_dasa"` to the skeleton engine. All 85 locale keys compile in both `en.rs` and `te.rs`.

---

## Phase 3: User Story 1 — Compute Vimsottari Mahadasa and Antardasa periods (Priority: P1) 🎯 MVP

**Goal**: Given a valid `cityId`, `localTime`, and `lang`, return the complete 120-year Vimsottari dasa sequence — 9 Mahadasas, each with 9 Antardasas (first Mahadasa may have fewer), with localized labels and contiguous formatted dates.

**Independent Test**: Supply `cityId` for Hyderabad, `localTime="1997-03-07T20:34:00"`, `lang=en` — verify Mars Mahadasa runs 1997-03-07 to 1999-01-18, its 3 Antardasas match reference dates exactly, and Rahu Mahadasa runs 1999-01-18 to 2017-01-18 with all 9 Antardasas matching.

### Implementation

- [ ] T006 [US1] Define `VimsottariRequest` struct (`operation`, `city_id`, `local_time`, `lang`) with `#[derive(Deserialize)]` and `#[serde(rename_all = "camelCase")]` in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T007 [US1] Define `VimsottariResponse`, `MahadasaEntry`, `AntardasaEntry` structs with `#[derive(Serialize)]` and `#[serde(rename_all = "camelCase")]` per data-model.md — include `lord`, `label`, `start_date`, `end_date`, `antardasas` fields — in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T008 [US1] Define `DasaLord` enum with variants (Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury) — each carries `years: u8` and `planet_key: &'static str`; define `DASA_SEQUENCE: [DasaLord; 9]` constant and `SOLAR_YEAR_DAYS: f64 = 365.2425` constant in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T009 [US1] Implement `nakshatra_to_lord_index(nakshatra_num: u8) -> usize` — returns `((nakshatra_num - 1) % 9) as usize` mapping nakshatra to DASA_SEQUENCE index per FR-003 — in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T010 [US1] Implement `compute_balance_fraction(moon_lon: f64, nakshatra_num: u8) -> f64` — computes `(nakshatra_end - moon_lon) / NAKSHATRA_SPAN` where `NAKSHATRA_SPAN = 13.0 + 1.0/3.0` per FR-004; returns value in `(0.0, 1.0]` — in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T011 [US1] Implement `offset_to_date(birth_dt: NaiveDateTime, cumulative_days: f64) -> NaiveDate` — adds fractional days as seconds via `TimeDelta::seconds((cumulative_days * 86_400.0).round() as i64)` to birth datetime, then rounds to nearest calendar day (time ≥ 12:00 → next day) per FR-006/R-004 — in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T012 [US1] Implement `format_date(date: NaiveDate, lang: &str) -> String` — formats as `"YYYY MonthName DD"` using `get_string(&format!("month.{}", month), lang)` for translated month name and zero-padded day per FR-012 — in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T013 [US1] Implement `build_antardasas(mahadasa_lord_idx: usize, mahadasa_total_days: f64, birth_dt: NaiveDateTime, mahadasa_start_offset: f64, elapsed_days: f64, lang: &str) -> Vec<AntardasaEntry>` — computes 9 Antardasas per FR-008/FR-009; for the partial first Mahadasa, skips Antardasas whose cumulative end ≤ elapsed portion and starts the first included one at birth date; uses cumulative fractional offsets from birth_dt for contiguous date formatting — in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T014 [US1] Implement `build_periods(starting_lord_idx: usize, balance_fraction: f64, birth_dt: NaiveDateTime, lang: &str) -> Vec<MahadasaEntry>` — builds 9 Mahadasas: first uses `balance_fraction × lord_years × SOLAR_YEAR_DAYS`, remaining 8 use full durations per FR-005/FR-007; each Mahadasa delegates to `build_antardasas`; all boundaries computed via cumulative offsets from birth_dt — in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T015 [US1] Implement `execute(request: &str) -> String` — parse JSON → `VimsottariRequest`; look up city in `CITIES`; resolve lat/lng via `decode_city_id()`; call `local_to_jd()`; call `swe_set_sid_mode(SE_SIDM_TRUE_CITRA, 0.0, 0.0)` + `swe_set_ephe_path`; call `calc_planet(jd, SE_MOON)` → moon_lon; call `decompose_longitude(moon_lon)` → nakshatra_num; compute balance; build periods; assemble `VimsottariResponse` with city echo fields; serialize to JSON — in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T016 [US1] Write unit test `test_reference_chart_mars_mahadasa` — verify SC-001: Hyderabad 1997-03-07T20:34:00, Mars Mahadasa 1997-03-07 to 1999-01-18, first 3 Antardasas (Mars–Venus 1997-03-07→1998-02-11, Mars–Sun 1998-02-11→1998-06-19, Mars–Moon 1998-06-19→1999-01-18) match exactly — in `astro-wasm/src/engines/vimsottari.rs` `#[cfg(test)]` module
- [ ] T017 [US1] Write unit test `test_reference_chart_rahu_mahadasa` — verify SC-002: Rahu Mahadasa 1999-01-18 to 2017-01-18, all 9 Antardasas match reference dates exactly — in `astro-wasm/src/engines/vimsottari.rs` `#[cfg(test)]` module
- [ ] T018 [US1] Write unit test `test_contiguity_all_periods` — verify SC-003: every Mahadasa end = next Mahadasa start; every Antardasa end = next Antardasa start within each Mahadasa; first Mahadasa starts on birth date; full Mahadasas have exactly 9 Antardasas — in `astro-wasm/src/engines/vimsottari.rs` `#[cfg(test)]` module
- [ ] T019 [P] [US1] Write unit test `test_telugu_localization` — verify SC-004: `lang=te` response contains Telugu labels (e.g., "కుజ మహాదశ") and Telugu month names (e.g., "మార్చి"); no `[missing]` strings — in `astro-wasm/src/engines/vimsottari.rs` `#[cfg(test)]` module
- [ ] T020 [P] [US1] Write unit test `test_balance_fraction` — verify `compute_balance_fraction` for known Moon longitudes: exact nakshatra start (balance ≈ 1.0), exact nakshatra end (balance ≈ 0.0), mid-nakshatra (balance ≈ 0.5) — in `astro-wasm/src/engines/vimsottari.rs` `#[cfg(test)]` module
- [ ] T021 [P] [US1] Write unit test `test_nakshatra_to_lord_mapping` — verify all 27 nakshatras map to correct lords: Ashwini(1)→Ketu, Bharani(2)→Venus, ..., Revati(27)→Mercury — in `astro-wasm/src/engines/vimsottari.rs` `#[cfg(test)]` module
- [ ] T029 [P] [US1] Write unit test `test_edge_case_nakshatra_boundary` — verify extreme balance values: Moon at exact nakshatra start (0° into nakshatra, e.g., 0.000° of Ashwini) → balance fraction = 1.0 → first Mahadasa equals full lord duration with all 9 Antardasas; Moon near nakshatra end → balance fraction ≈ 0.0 → first Mahadasa duration rounds to 0 or 1 day — in `astro-wasm/src/engines/vimsottari.rs` `#[cfg(test)]` module

**Checkpoint**: `cargo test` passes. All reference chart dates match exactly. `bash build.sh` succeeds. Calling bridge with `"vimsottari_dasa"` returns complete dasa periods.

---

## Phase 4: User Story 2 — Reject invalid inputs with clear errors (Priority: P2)

**Goal**: Unknown `cityId` or malformed `localTime` returns `{"error":"..."}` with a descriptive message; no silent corruption.

**Independent Test**: Call with `cityId=99999` → response contains `"error"` field; call with `localTime="not-a-date"` → response contains `"error"` field.

- [ ] T022 [US2] Return `{"error":"city not found: cityId=<value>"}` when `cityId` is absent from `CITIES` in `astro-wasm/src/engines/vimsottari.rs` (early-return from `execute()` before any SE calls)
- [ ] T023 [US2] Return `{"error":"invalid localTime '<value>': <detail>"}` when `local_to_jd()` returns `Err(_)` in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T024 [US2] Return `{"error":"JSON parse error: <detail>"}` when request JSON cannot be deserialized into `VimsottariRequest` in `astro-wasm/src/engines/vimsottari.rs`
- [ ] T025 [P] [US2] Write unit tests in `astro-wasm/src/engines/vimsottari.rs` `#[cfg(test)]` module: (a) `test_unknown_city` — unknown cityId → error JSON with "city not found", (b) `test_malformed_time` — malformed localTime → error JSON, (c) `test_unknown_lang_fallback` — unknown lang → all strings in English, no `[missing]`, (d) `test_json_parse_error` — malformed JSON body → error JSON with "JSON parse error" per FR-020

**Checkpoint**: `cargo test` passes. Bridge returns error JSON for both invalid-input cases. SC-005 verified.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, WASM build, and integration verification.

- [ ] T026 Verify `bash build.sh` succeeds with zero warnings; WASM module includes new `"vimsottari_dasa"` operation; record WASM binary size before and after this feature and document the delta (constitution Workflow Step 4 — payload audit)
- [ ] T027 Verify existing `cargo test` suite still passes (all horoscope, stub, city, and bridge tests unchanged)
- [ ] T028 Verify locale completeness: both `en.rs` (85 keys) and `te.rs` (85 keys) compile with updated assertions; no `[missing]` for any key used by vimsottari

---

## Dependencies

```text
T001, T002 ──→ T003, T004, T005 (Setup before Foundational)

T003, T004, T005 ──→ T006–T021, T029 (Foundational before US1)

T006, T007, T008 ──→ T009, T010 (structs/constants before helpers)
T009, T010 ──→ T011, T012 (helpers before date formatting)
T011, T012 ──→ T013 (date formatting before Antardasa builder)
T013 ──→ T014 (Antardasa before Mahadasa builder)
T014 ──→ T015 (period builder before execute)
T015 ──→ T016, T017, T018, T019, T029 (execute before integration tests)
T009, T010 ──→ T020, T021 (helper unit tests — parallelizable alongside T015 and its dependents)

T015 ──→ T022, T023, T024 (execute before error handling)
T022, T023, T024 ──→ T025 (error handling before error tests)

T025, T021, T029 ──→ T026, T027, T028 (all tests before polish)
```

## Parallel execution opportunities

| Phase | Parallel tasks | Why parallelizable |
|-------|---------------|-------------------|
| Phase 2 | T003, T004 | Different files (`en.rs`, `te.rs`) — no overlap |
| Phase 3a | T016, T017, T018, T019, T029 | All depend only on T015 (execute); independent test functions |
| Phase 3b | T020, T021 | Depend only on T009/T010 (pure math helpers); parallelizable independently of T015 |
| Phase 4 | T025 (a), (b), (c), (d) | Independent test functions within same file |

## Implementation strategy

1. **MVP (Phase 1–3)**: Delivers the core Vimsottari computation — everything needed to compute and return correct dasa periods for any birth chart. After Phase 3, the feature is fully functional for valid inputs.
2. **Error handling (Phase 4)**: Adds defensive validation for invalid inputs. Independent of Phase 3 correctness.
3. **Polish (Phase 5)**: Final build verification and regression check.

## Summary

| Metric | Value |
|--------|-------|
| Total tasks | 29 |
| Phase 1 (Setup) | 2 |
| Phase 2 (Foundational) | 3 |
| Phase 3 (US1 — P1) | 17 |
| Phase 4 (US2 — P2) | 4 |
| Phase 5 (Polish) | 3 |
| Parallel opportunities | 4 groups |
| Files created | 1 (`vimsottari.rs`) |
| Files modified | 4 (`engines/mod.rs`, `bridge.rs`, `en.rs`, `te.rs`) |
| Files unchanged | All existing engine/utility files |
