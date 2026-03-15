# Feature Specification: City Data

**Feature Branch**: `004-city-data`
**Created**: 2026-03-15
**Status**: Draft

## Clarifications

### Session 2026-03-15

- Q: Should the bridge return the city list as a raw JSON array or wrapped in an object? → A: Option A — wrap in object: `{"cities": [{…}]}`. Consistent with existing bridge shape (always a top-level object); error still uses `{"error":"…"}`; JS caller checks `error` field first, then reads `cities`.
- Q: What order should cities appear in the response? → A: Ordered by language-specific sort keys: `region2Order` ascending, then `region1Order` ascending, then localized `cityName` ascending — all from the requested language's translation entry. Telugu users see Telangana-first; other languages see their own regional grouping first. Sort keys are internal to the data store and not present in the response.
- Q: What is the minimum seed dataset for the initial data store? → A: 4 cities — Hyderabad (Telangana/India, te+en), Vijayawada (Andhra Pradesh/India, te+en), Delhi (Delhi/India, en only), New York (New York/USA, en only). Covers two Telugu-speaking states for regional order verification, one domestic en-only city, and one international en-only city.
- Q: How should a malformed `list_cities` request be handled (missing `lang`, invalid JSON, etc.)? → A: Option A — return `{"error": "<descriptive message>"}`, consistent with the existing bridge error pattern. The JS caller already checks the `error` field first, so no new handling is needed on the web side.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Web app fetches a translated city list for language selection (Priority: P1)

A user opens the astrology application. The app needs to show a city picker so the user can select their birth city. The app requests the city list with a language code, and each city's name, state, and country are shown in the user's language. The user can identify their city by its familiar local name.

**Why this priority**: The city picker is a prerequisite for birth-chart calculation — without a correctly named city list, users cannot select their location. This is the core deliverable of the feature.

**Independent Test**: Call `listCities("te")` and verify the response contains at least one entry where `cityName`, `region1`, and `region2` are in Telugu script, `canonicalName` is in English, and `cityId` is a non-zero integer.

**Acceptance Scenarios**:

1. **Given** the library is loaded, **When** `listCities("te")` is called, **Then** the response is a JSON array where every entry has `cityName`, `region1`, and `region2` in Telugu and `canonicalName` in English.
2. **Given** the library is loaded, **When** `listCities("en")` is called, **Then** the response is a JSON array where every entry has `cityName`, `region1`, and `region2` in English.
3. **Given** the library is loaded, **When** `listCities("en")` is called, **Then** each city entry includes a `cityId`, `timeZone`, `canonicalName`, `cityName`, `region1`, and `region2`.

---

### User Story 2 — Cities without a translation for the requested language are hidden (Priority: P2)

A developer adds a remote village that only has Telugu translations. When another app instance requests the city list in English, that village does not appear — keeping the list clean and avoiding untranslated entries.

**Why this priority**: Showing a city with a missing translation would break UI display (empty labels or raw internal identifiers). Language-based filtering is a correctness requirement, not a nice-to-have.

**Independent Test**: Add a test city record with only a `te` translation. Call `listCities("en")` and assert that city is absent. Call `listCities("te")` and assert that city is present.

**Acceptance Scenarios**:

1. **Given** a city exists with only a `te` translation, **When** `listCities("en")` is called, **Then** that city is not present in the response array.
2. **Given** a city exists with only a `te` translation, **When** `listCities("te")` is called, **Then** that city is present in the response array with all fields populated.
3. **Given** no cities have translations for a requested language code, **When** `listCities("<that-language>")` is called, **Then** the response is an empty array (not an error).

---

### User Story 3 — Developer can identify any city by its canonical English name (Priority: P3)

A developer inspecting a bug receives a city object from a Telugu user session. Even though all display names are in Telugu script, the `canonicalName` field is always in English, letting the developer immediately understand which city is involved.

**Why this priority**: Canonical names are a debugging and data-integrity aid. They decouple display from identification and make logs and test assertions language-agnostic.

**Independent Test**: Call `listCities("te")` and assert that every returned city object has a non-empty `canonicalName` containing only Latin-script characters.

**Acceptance Scenarios**:

1. **Given** `listCities("te")` is called, **Then** every entry in the response has a non-empty `canonicalName` in English regardless of the `te` display names.
2. **Given** `listCities("en")` is called, **Then** the `canonicalName` and `cityName` values for a city like Hyderabad are identical (both English).

---

### Edge Cases

- If an unknown or unsupported language code is passed, the response is `{"cities": []}` — not an error.
- If the request JSON is malformed or the `lang` field is missing, the response is `{"error": "<descriptive message>"}` — not an empty cities array.
- If the data array contains a city with no `translatedNames` entries at all, that city must not appear in any language response.
- `canonicalName` must always be present for every city in the data store; a city without a canonical name is a data error caught at build/test time.
- `cityId` values must be unique across all city records; duplicates are a data error caught at build/test time.
- `timeZone` values must be valid IANA timezone identifiers; invalid values are a data error caught at build/test time.
- `region1Order` and `region2Order` are sort keys only — they MUST NOT appear in the JSON response. Their absence from the response must be validated by tests.
- If two cities share the same `region2Order`, `region1Order`, and `cityName` for a given language, their relative order is unspecified (stable sort is preferred but not required).
- International cities (e.g., New York) that have no Telugu translation MUST NOT appear in `listCities("te")` responses.

## Requirements *(mandatory)*

### Web ↔ Library Contract

The library exposes a single city-list operation. This establishes the `city-data-v1` contract.

*Request* (JS → WASM):
```json
{
  "operation": "list_cities",
  "lang": "en"
}
```

`lang` is an IETF-style language tag string (e.g., `"en"`, `"te"`).

*Response* (WASM → JS) — success:
```json
{
  "cities": [
    {
      "lang": "en",
      "cityId": 12345,
      "timeZone": "Asia/Kolkata",
      "canonicalName": "Hyderabad",
      "cityName": "Hyderabad",
      "region1": "Telangana",
      "region2": "India"
    }
  ]
}
```

*Response* (WASM → JS) — error:
```json
{
  "error": "<human-readable error message string>"
}
```

The top-level value is always a JSON object, consistent with the existing bridge contract. On success, `cities` contains the (possibly empty) array of city objects — an empty array `[]` is a valid success response when no cities match the requested language. On error (malformed request, missing `lang` field, invalid JSON), `cities` is absent and `error` is present with a descriptive message. The JS caller MUST check for the `error` field before reading `cities`.

*Field definitions*:

| Field | Type | Description |
|-------|------|-------------|
| `lang` | string | Echoes the language code from the request |
| `cityId` | integer | 16-bit quadkey encoding the city's tile at zoom level 7 (formula: `tile_x × 128 + tile_y`, Web Mercator) |
| `timeZone` | string | IANA timezone identifier (e.g., `"Asia/Kolkata"`) |
| `canonicalName` | string | City name always in English; never changes with `lang` |
| `cityName` | string | City name in the requested language |
| `region1` | string | Administrative subdivision (state/province) in the requested language |
| `region2` | string | Country name in the requested language |
| `lat` | number | Latitude of the city's tile centre (decimal degrees), decoded from `cityId` in Rust — tile-centre approximation, not the exact geographic centre of the city |
| `lng` | number | Longitude of the city's tile centre (decimal degrees), decoded from `cityId` in Rust — tile-centre approximation, not the exact geographic centre of the city |

### Functional Requirements

- **FR-001**: The library MUST expose a `list_cities` operation via the existing bridge contract, accepting a `lang` parameter and returning a JSON object `{"cities": […]}` on success or `{"error": "…"}` on failure, as defined above. The top-level response is always an object — never a bare array.
- **FR-002**: The library MUST store all city data as a static array compiled into the WASM binary. No external database, file, or network call is involved at runtime. The source of truth for city data is `data/cities.csv` at the repository root; `astro-wasm/src/data/cities.rs` is auto-generated from this CSV at build time by `build.rs` and MUST NOT be edited directly.
- **FR-003**: Each city record in the data store MUST contain: a unique 16-bit quadkey as its identifier, an IANA timezone string, a canonical English name, and a map of language-keyed translation entries.
- **FR-004**: Each translation entry in the data store MUST contain: `cityName`, `region1`, `region2`, `region1Order`, and `region2Order` for that language. `region1Order` and `region2Order` are integer sort keys used solely for ordering; they MUST NOT appear in the JSON response.
- **FR-005**: The library MUST exclude any city from the response whose data store record contains no translation entry for the requested language.
- **FR-006**: The `canonicalName` field in every response entry MUST always be the English name of the city, regardless of the `lang` parameter.
- **FR-007**: The `lang` field in every response entry MUST echo the `lang` value from the request.
- **FR-008**: The `cityId` field MUST be the city's 16-bit quadkey integer computed at zoom level 7 (`tile_x × 128 + tile_y`, Web Mercator). The `lat` and `lng` response fields MUST be decoded from `cityId` inside Rust using the inverse tile-centre formula; no coordinates are stored in the data array.
- **FR-013**: The Rust `data` module MUST expose a `pub fn decode_city_id(city_id: u16) -> (f64, f64)` function returning `(lat, lng)` in decimal degrees using the Web Mercator tile-centre formula. The JS layer MUST NOT implement any coordinate decoding; it receives ready-decoded `lat`/`lng` values in the response payload.
- **FR-009**: Adding a new city or a new language translation MUST require only a change to `data/cities.csv` and a rebuild of the WASM binary. No Rust source edits, schema migrations, or configuration file changes are required. A non-technical editor can manage `data/cities.csv` directly via a text editor or the GitHub web UI.
- **FR-014**: The `build.rs` script MUST read `data/cities.csv` at compile time, parse it into `CityRecord` / `TranslationEntry` struct literals, and write the result into a generated Rust source file (`OUT_DIR/cities_generated.rs`). The generator MUST emit clear build errors with line numbers for malformed rows, unknown columns, or duplicate `city_id` values. No new `[build-dependencies]` crates are required — the generator uses only `std`.
- **FR-010**: The city data array MUST be validated at build/test time for: unique `cityId` values, non-empty `canonicalName` for every city, and valid IANA timezone strings.
- **FR-011**: The `list_cities` response MUST be sorted by the requested language's `region2Order` (ascending), then `region1Order` (ascending), then localized `cityName` (ascending, lexicographic). This produces language-aware regional grouping (e.g., Telugu users see Telangana cities before other states). Sort keys MUST NOT appear in the response payload.
- **FR-012**: If the bridge receives a malformed JSON request or a `list_cities` request with a missing `lang` field, it MUST return `{"error": "<descriptive message>"}`. An unknown but syntactically valid language code is NOT an error; it returns `{"cities": []}`.

### Seed Dataset

The initial hardcoded data array MUST contain exactly these five cities:

| canonicalName | region1 (en) | region2 (en) | Languages |
|---|---|---|---|
| Hyderabad | Telangana | India | te, en |
| Vijayawada | Andhra Pradesh | India | te, en |
| Eluru | Andhra Pradesh | India | te, en |
| Delhi | Delhi | India | en only |
| New York | New York | USA | en only |

This set validates: translated output (Hyderabad, Eluru, Vijayawada in Telugu), regional ordering (Telangana before Andhra Pradesh for `te`), **city-name tie-breaking** (Eluru and Vijayawada share `region1_order`; `cityName` lexicographic sort places Eluru first in both `te` and `en`), domestic en-only city (Delhi), and international en-only city (New York).

### Key Entities

- **City Record**: The authoritative description of a city held in the Rust data store. Contains a unique 16-bit quadkey identifier, an IANA timezone, a canonical English name, and a map of translated name sets keyed by language code. Not directly exposed to JS — it is the source from which City Response Objects are derived.
- **Translation Entry**: A language-specific name set within a City Record. Contains `cityName`, `region1`, `region2`, `region1Order`, and `region2Order` for one language. `region1Order` and `region2Order` are integer sort keys that control the ordering of the response for this language (e.g., lower values surface preferred regions first). A City Record may have zero or more Translation Entries.
- **City Response Object**: The JSON object returned to the web app. Derived from a City Record + one Translation Entry for the requested language. Contains `lang`, `cityId`, `timeZone`, `canonicalName`, `cityName`, `region1`, `region2`, `lat`, and `lng`. `lat` and `lng` are decoded from `cityId` in Rust — the JS layer receives them as plain numbers. Only emitted when a matching Translation Entry exists.
- **QuadKey**: A 16-bit integer that encodes a geographic tile at zoom level 7 (128×128 tile grid, Web Mercator). Computed as `tile_x × 128 + tile_y`. Serves as the unique, stable identifier for a city. The tile-centre latitude and longitude are decoded from the quadkey in Rust (`decode_city_id`) and included as `lat`/`lng` in every response entry — the JS layer never performs this decoding.

## Assumptions

- The initial dataset focuses on cities in India (primarily Telangana/Andhra Pradesh), with Telugu (`te`) and English (`en`) as the two supported languages at launch.
- The city list is expected to be small enough (hundreds, not millions) that returning the full list in a single call is always acceptable; no pagination is needed.
- Redeployment is an acceptable mechanism for adding cities, since city additions are infrequent (estimated less than a dozen per year).
- `region1` corresponds to the state/province level and `region2` corresponds to the country level. Finer-grained administrative divisions (district, taluk) are out of scope for this feature.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Calling `list_cities` with `lang="en"` returns a `cities` array containing all five seed cities (Hyderabad, Vijayawada, Eluru, Delhi, New York), each with all nine required fields populated (including `lat` and `lng`).
- **SC-002**: Calling `list_cities` with `lang="te"` returns exactly three cities (Hyderabad, Eluru, and Vijayawada, in that order); Delhi and New York, which have no Telugu translation, are absent. Eluru and Vijayawada share the same `region1_order`; Eluru sorts first because `"ఏలూరు"` precedes `"విజయవాడ"` lexicographically.
- **SC-003**: Every entry in any `list_cities` response has a `canonicalName` in English, verifiable by asserting no non-Latin script characters appear in that field.
- **SC-004**: The city list is available to the web app without any network request beyond the initial WASM binary load — verified by checking that no HTTP requests are made after WASM initialisation when `list_cities` is called.
- **SC-005**: The build fails (compile error or failing test) if any city record in the data store has a duplicate `cityId`, an empty `canonicalName`, or an invalid timezone string — preventing bad data from reaching production. A malformed row in `data/cities.csv` (wrong column count, non-integer `city_id`) must also cause the build to fail with a diagnostic message including the offending line number.
