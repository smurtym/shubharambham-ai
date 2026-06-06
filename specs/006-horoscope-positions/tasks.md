# Tasks: Horoscope Positions

**Feature**: `006-horoscope-positions`  
**Input**: Design documents from `specs/006-horoscope-positions/`  
**Branch**: `006-horoscope-positions`  
**Generated**: 2026-03-15

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, independent of in-progress tasks)
- **[US1/US2/US3]**: User story this task belongs to
- All paths are relative to the repository root

---

## Phase 1: Setup

**Purpose**: Add new Cargo dependencies; no source changes yet.

- [X] T001 Add `chrono` and `chrono-tz` dependencies to `astro-wasm/Cargo.toml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared utilities and SE wrappers that all user stories depend on. Must be complete before Phase 3.

**⚠️ No user story work can begin until this phase is complete.**

- [X] T002 Add SE constant definitions (`SE_SUN=0`, `SE_MOON=1`, `SE_MERCURY=2`, `SE_VENUS=3`, `SE_MARS=4`, `SE_JUPITER=5`, `SE_SATURN=6`, `SE_TRUE_NODE=11`, `SEFLG_SIDEREAL=65536`, `SEFLG_SPEED=256`, `SE_SIDM_TRUE_CITRA=27`) to `astro-wasm/src/swe_wrappers.rs`
- [X] T003 [P] Add `calc_planet(jd: f64, body: i32) -> Result<f64, String>` wrapper — calls `swe_calc_ut` with `SEFLG_SIDEREAL | SEFLG_SPEED`, returns sidereal longitude — in `astro-wasm/src/swe_wrappers.rs`
- [X] T004 [P] Add `calc_ascendant(jd: f64, lat: f64, lon: f64) -> Result<f64, String>` wrapper — calls `swe_houses_ex` with `SEFLG_SIDEREAL | iflag` and `hsys='W'`, returns `ascmc[0]` — note: caller (`execute()` in T012) MUST call `swe_set_sid_mode(SE_SIDM_TRUE_CITRA, 0.0, 0.0)` before invoking this wrapper — in `astro-wasm/src/swe_wrappers.rs`
- [X] T005 Add `local_to_jd(local_time: &str, iana_tz: &str) -> Result<f64, String>` — parses `%Y-%m-%dT%H:%M:%S`, converts to UTC via `chrono-tz`, then to Julian Day — in `astro-wasm/src/utils.rs`
- [X] T006 [P] Add `decompose_longitude(lon: f64) -> (u8, u8, u8, u8, u8, u8)` returning `(zodiac_num, deg_in_sign, minutes, seconds, nakshatra_num, pada)` using truncation (not rounding) — in `astro-wasm/src/utils.rs`
- [X] T007 [P] Add `navamsa_sign(longitude: f64) -> u8` implementing the Chara/Sthira/Ubhaya D9 algorithm (Chara idx%3==0 → start Aries=0; Sthira idx%3==1 → start Capricorn=9; Ubhaya idx%3==2 → start Cancer=3) returning 1-based sign number — in `astro-wasm/src/utils.rs`
- [X] T008 [P] Add 71 English locale keys to `astro-wasm/src/locales/en.rs`: 10 planet names (`planet.Sun` … `planet.Ketu`, `planet.Ascendant`), 10 planet abbreviations (`planet.abbrev.*`), 12 zodiac sign names (`sign.Aries` … `sign.Pisces`), 12 zodiac abbreviations (`sign.abbrev.*`), 27 nakshatra names (`nakshatra.1` … `nakshatra.27`) — values from `specs/006-horoscope-positions/data-model.md`
- [X] T009 Register `pub mod horoscope;` in `astro-wasm/src/engines/mod.rs`
- [X] T025 [P] Add compile-time locale completeness assertion in `astro-wasm/src/locales/en.rs`: a `const` block or `build.rs` check that verifies all 71 required keys (`planet.*`, `sign.*`, `nakshatra.*`) are present in `en::STRINGS`; build MUST fail if any key is absent (SC-005)

**Checkpoint**: `cargo check` passes, all new symbols resolve, no user story code yet.

---

## Phase 3: User Story 1 — Compute Vedic planetary positions (Priority: P1) 🎯 MVP

**Goal**: Given a valid `cityId`, `localTime`, and `lang`, return all ten planetary positions with complete sidereal zodiac, nakshatra, pada, and navamsa data.

**Independent Test**: Supply `cityId=11705` (Hyderabad), `localTime="1961-10-28T07:30:00"`, `lang="en"` — verify the response contains all ten bodies, each with `longitude`, `zodiacNumber`, `zodiacSign`, `degreesInSign`, `minutes`, `seconds`, `nakshatra`, `nakshatraName`, `pada`, `navamsaZodiacNumber`, `navamsaZodiacSign`.

- [X] T010 [US1] Define `HoroscopeRequest`, `HoroscopeResponse`, and `PlanetaryPosition` structs (with all 15 fields from `data-model.md`) with `#[derive(Serialize, Deserialize)]` in new file `astro-wasm/src/engines/horoscope.rs`
- [X] T011 [US1] Implement `compute_position(longitude: f64, body_key: &str, lang: &str) -> PlanetaryPosition` — calls `decompose_longitude`, `navamsa_sign`, and locale lookups for planet name/abbrev, zodiac name/abbrev, nakshatra name, navamsa name/abbrev — in `astro-wasm/src/engines/horoscope.rs`
- [X] T012 [US1] Implement `execute(request: &str) -> String` — parse JSON into `HoroscopeRequest`, look up city in `CITIES`, resolve lat/lng via `decode_city_id()`, call `local_to_jd()`, call `swe_set_sid_mode(SE_SIDM_TRUE_CITRA, 0.0, 0.0)` + `swe_set_ephe_path`, then loop `calc_planet` for Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn — in `astro-wasm/src/engines/horoscope.rs`
- [X] T013 [US1] Add Rahu and Ketu computation in `execute()`: Rahu = `calc_planet(jd, SE_TRUE_NODE)`, Ketu longitude = `(rahu_lon + 180.0) % 360.0`, both passed to `compute_position` — in `astro-wasm/src/engines/horoscope.rs`
- [X] T014 [US1] Add Ascendant computation in `execute()`: call `calc_ascendant(jd, lat, lon)` and pass result to `compute_position` with key `"Ascendant"` — in `astro-wasm/src/engines/horoscope.rs`
- [X] T015 [P] [US1] Add 71 Telugu locale keys to `astro-wasm/src/locales/te.rs`: same 71 keys as T008 with Telugu values from `specs/006-horoscope-positions/data-model.md` (planet names, zodiac sign names, nakshatra names — zodiac abbreviations use full Telugu names per data-model)
- [X] T016 [US1] Add `"horoscope_positions"` dispatch arm in `astro-wasm/src/bridge.rs` — routes to `engines::horoscope::execute(input)`, writes result to output buffer, returns byte count
- [X] T017 [US1] Add `getHoroscopePositions(cityId, localTime, lang)` function in `web/data.js` following the existing `listCities` pattern — calls bridge with `{"operation":"horoscope_positions","cityId":cityId,"localTime":localTime,"lang":lang}`, parses and returns raw JSON
- [X] T026 [US1] Wire `getHoroscopePositions` into `web/index.html`: add a city/datetime/lang form and a `<pre id="horoscope-result">` block; on submit call `getHoroscopePositions` and display `JSON.stringify(result, null, 2)` in the pre block

**Checkpoint**: `bash build.sh` succeeds. Calling `getHoroscopePositions(11705, "1961-10-28T07:30:00", "en")` from browser console returns a fully-populated response with all ten bodies.

---

## Phase 4: User Story 2 — Reject invalid inputs (Priority: P2)

**Goal**: Unknown `cityId` or malformed `localTime` returns `{"error":"..."}` with a descriptive message; no silent corruption.

**Independent Test**: Call with `cityId=99999` → response contains `"error"` field; call with `localTime="not-a-date"` → response contains `"error"` field.

- [X] T018 [US2] Return `{"error":"city not found: cityId=<value>"}` when `cityId` is absent from `CITIES` in `astro-wasm/src/engines/horoscope.rs` (early-return from `execute()` before any SE calls)
- [X] T019 [US2] Return `{"error":"invalid localTime: <value>"}` when `local_to_jd()` returns `Err(_)` in `astro-wasm/src/engines/horoscope.rs`
- [X] T020 [P] [US2] Write unit tests in `astro-wasm/src/engines/horoscope.rs` (`#[cfg(test)]` module): (a) unknown cityId → error JSON, (b) malformed localTime → error JSON, (c) unknown lang → all fields in English, no `[missing]` strings

**Checkpoint**: `cargo test` passes. Bridge returns -2 or error JSON for both invalid-input cases.

---

## Phase 5: User Story 3 — City context in response (Priority: P3)

**Goal**: Every successful response includes `cityName`, `region1`, `region2`, `lat`, `lng`, `timezone` matching the stored city record for the requested language.

**Independent Test**: Call with `cityId=11705`, `lang=te` — assert `cityName` is in Telugu script, `timezone` is `"Asia/Kolkata"`, `lat` and `lng` are non-zero.

- [X] T021 [US3] Populate `cityName`, `region1`, `region2`, `lat`, `lng`, and `timezone` in `HoroscopeResponse` from the resolved `CityRecord` inside `execute()` in `astro-wasm/src/engines/horoscope.rs` — use `get_string(city_name_key, lang)` for translated fields
- [X] T022 [P] [US3] Write unit test in `astro-wasm/src/engines/horoscope.rs`: assert that `cityId=11705` + `lang=te` response contains Telugu `cityName`, correct `timezone="Asia/Kolkata"`, and matching `lat`/`lng` values

**Checkpoint**: Response for any valid request always contains all six city-context fields.

---

## Phase 6: Polish & Correctness

**Purpose**: Validate accuracy (SC-001) and confirm end-to-end WASM build.

- [X] T023 Write reference chart accuracy test in `astro-wasm/src/engines/horoscope.rs` (`#[cfg(test)]`): use a published Vedic reference birth chart (date, time, city with known sidereal longitudes), assert all ten computed longitudes within ±0.02° (SC-001)
- [X] T024 [P] Run full WASM build (`bash build.sh`) and manually verify `getHoroscopePositions` in `web/index.html` renders raw JSON with all ten planets for a test input (pre-condition: T026 must be complete — the horoscope form must exist in index.html)

---

## Dependencies

```
US1 (Phase 3) requires Phase 2 (all foundational tasks)
US2 (Phase 4) requires T010-T014 (execute() must exist to add error branches)
US3 (Phase 5) requires T010 (HoroscopeResponse struct must exist to add city fields)
T023 (Phase 6) requires Phase 3 complete
```

Story completion order: **Phase 2 → US1 → US2 → US3 → Polish**

US2 and US3 can partially overlap — T020/T022 (tests) can be written while US3 implementation (T021) is in progress, since they touch different parts of the same file.

---

## Parallel Execution Examples

**Within Phase 2** (fully parallel after T002):
```
T002  →  T003, T004 (both in swe_wrappers.rs — write sequentially)
       →  T005, T006, T007 (utils.rs — write sequentially)
       →  T008 (locales/en.rs)
       →  T009 (engines/mod.rs)
```

**Within Phase 3**:
```
T010 (structs) → T011 (compute_position) → T012 (execute body loop) → T013, T014
T015 (te.rs) can run in parallel with any of T010-T014
T016 (bridge) requires T010
T017 (data.js) requires T016
```

---

## Implementation Strategy

**MVP** = Phase 1 + Phase 2 + Phase 3 (US1 only)  
This delivers the complete happy path: valid input → all ten planetary positions in English or Telugu.

**Increment 2** = Add Phase 4 (US2) — safe error handling before exposing to any UI.

**Increment 3** = Add Phase 5 (US3) — city context convenience; trivially backward-compatible.

**Increment 4** = Phase 6 — accuracy verification and final build check.
