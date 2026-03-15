# Feature Specification: Horoscope Positions

**Feature Branch**: `006-horoscope-positions`  
**Created**: 2026-03-15  
**Status**: Draft — Clarification complete  
**Input**: User description: "Rust engine providing planetary positions for Vedic horoscope. Input: lang, city_id, local time. Output: sidereal positions for Ascendant, Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu with zodiac sign, nakshatra, pada, and navamsa. Use True Chitrapaksha Ayanamsa."

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Compute Vedic planetary positions for any date and time (Priority: P1)

An astrologer (or an application acting on their behalf) selects a city, enters any date and time — past, present, or future — in the city's local time, and chooses a display language. They receive sidereal planetary positions for all ten Vedic celestial bodies — Ascendant, Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, and Ketu — each with complete zodiac, nakshatra, pada, and navamsa details. The same operation serves natal charts, transit lookups, muhurta timing, and any other time-specific astrological query.

**Why this priority**: This is the core capability. Every downstream feature — chart display, natal analysis, transit overlays, dasha period calculations, compatibility matching, muhurta selection — depends entirely on accurate, complete planetary positions.

**Independent Test**: Supply a known date, time, city, and language; verify that the returned sidereal longitudes match a published Vedic reference for that moment and location.

**Acceptance Scenarios**:

1. **Given** a valid `cityId` (e.g., Hyderabad), a valid local datetime (e.g., `1961-10-28T07:30:00`), and `lang=en`, **When** the horoscope positions operation is invoked, **Then** the response contains all ten bodies, each with absolute sidereal longitude, zodiac sign (number, name, abbreviation), degrees/minutes/seconds within the sign, nakshatra number and name, pada (1–4), navamsa zodiac sign number, name, and abbreviation.
2. **Given** the same inputs with `lang=te`, **When** the operation is invoked, **Then** all planet names, zodiac sign names, nakshatra names, and navamsa names in the response are in Telugu script.
3. **Given** an unknown lang (e.g., `lang=hi`), **When** the operation is invoked, **Then** all translated strings fall back to English — no `[missing]` values appear for any planet, sign, or nakshatra key.
4. **Given** a future datetime (e.g., `2035-06-15T12:00:00`) and a valid `cityId`, **When** the operation is invoked, **Then** the engine returns positions without error — no date-range restriction is enforced.

---

### User Story 2 — Reject invalid inputs with clear errors (Priority: P2)

A developer integrating the engine supplies an invalid city ID or a malformed time string and receives a structured error response rather than silently incorrect data.

**Why this priority**: Incorrect data on a horoscope chart can mislead users into wrong astrological conclusions. Failing loudly on bad input is safer than silent corruption.

**Independent Test**: Supply a non-existent `cityId` and a malformed `localTime` string; assert that the response contains an `error` field with a non-empty descriptive message.

**Acceptance Scenarios**:

1. **Given** a `cityId` that does not exist in the city store, **When** the operation is invoked, **Then** the response is `{"error":"..."}` with a descriptive human-readable message.
2. **Given** a `localTime` string that is not a valid datetime (e.g., `"not-a-date"`), **When** the operation is invoked, **Then** the response is `{"error":"..."}` with a descriptive message.
3. **Given** valid inputs but an `operation` field in the JSON body that does not match the bridge operation name, **When** the bridge routes the request, **Then** the bridge returns -2 (operation mismatch), consistent with the existing bridge contract.

---

### User Story 3 — City context included in response (Priority: P3)

The caller receives the resolved city information (translated name, region, country, coordinates, timezone) alongside the planetary positions so no separate city-lookup round-trip is needed to display the chart header.

**Why this priority**: Convenience improvement — reduces client-side state management without requiring additional computation.

**Independent Test**: Assert that the response contains `cityName`, `region1`, `region2`, `lat`, `lng`, and `timezone` fields matching the stored city data for the given `cityId` and `lang`.

**Acceptance Scenarios**:

1. **Given** `cityId` for Hyderabad and `lang=te`, **When** the operation is invoked, **Then** the response contains the Telugu-translated city name, region, and country alongside coordinates and the IANA timezone string.
2. **Given** `cityId` for New York and `lang=te`, **When** the operation is invoked, **Then** city fields fall back to the English translation (New York has no Telugu translation) — consistent with existing city data behaviour.

---

### Edge Cases

- **DST ambiguity (fall-back)**: A `localTime` value that falls in a DST overlap (clocks fall back, time exists twice) is resolved using the earlier UTC offset. **DST gap (spring-forward)**: A `localTime` in a gap (time doesn't exist) is resolved by advancing to the post-gap time. Cities with no DST (e.g., `Asia/Kolkata`) are unaffected.
- **Rahu / Ketu pair**: Ketu's longitude is always exactly Rahu's longitude + 180°, normalised to [0°, 360°). No separate body lookup is performed for Ketu.
- **Ascendant and a planet in the same nakshatra/pada**: Both entries are correct and both appear independently in the response — no deduplication.
- **Longitude precision**: Fractional seconds are truncated, not rounded, to integers in the `minutes` and `seconds` fields.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The operation MUST accept three inputs: a language code (`lang`, BCP-47), a 16-bit city identifier (`cityId`), and a local datetime string (`localTime`) in the format `YYYY-MM-DDTHH:MM:SS` without a timezone suffix. The timezone is derived from the city record. The engine MUST accept any valid calendar date — past, present, or future — within the range supported by the Swiss Ephemeris.
- **FR-002**: The city's IANA timezone string MUST be resolved from the city store using `cityId`. The engine MUST convert `localTime` from that local timezone to UTC using a full IANA timezone database (provided via the `chrono-tz` crate or equivalent). The resulting UTC datetime is then converted to the Julian Day used for all Swiss Ephemeris calculations. For DST-observing timezones, ambiguous local times (clocks fall back) are resolved using the earlier UTC offset (pre-transition); gap times (clocks spring forward) are resolved by advancing to the post-gap UTC time.
- **FR-003**: The city's geographic coordinates (latitude and longitude) MUST be resolved from the city store using the same zoom-7 tile-centre formula used by the city-data feature, and MUST be used for Ascendant calculation.
- **FR-004**: All sidereal longitudes — including the Ascendant — MUST be computed using the **True Chitrapaksha Ayanamsa**, implemented via Swiss Ephemeris sidereal mode `SE_SIDM_TRUE_CITRA` (mode 27), which fixes the star Spica/Chitra at exactly 0° Libra using its true stellar position. This mode MUST be applied consistently to all ten bodies. Lahiri (mode 1) and all other ayanamsa modes are explicitly excluded.
- **FR-005**: The Ascendant (Lagna) degree MUST be calculated using the Swiss Ephemeris `swe_houses_ex` function with the **Whole Sign** house system (hsys `'W'`), the resolved city latitude and longitude, and the computed Julian Day. The Ascendant is treated as a planetary body in its own right: its exact sidereal degree (after ayanamsa subtraction) MUST be returned as a `PlanetaryPosition` entry with its own nakshatra, pada, and navamsa values. House cusp degrees are not part of this feature's output.
- **FR-006**: The engine MUST compute sidereal positions for exactly ten bodies: Ascendant, Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu (True Node), and Ketu (True Node + 180°).
- **FR-007**: For each of the ten bodies, the response MUST include:
  - Absolute sidereal longitude in decimal degrees (0–360)
  - Zodiac sign number (1–12, Aries = 1)
  - Zodiac sign name and two-letter abbreviation, both translated
  - Integer degrees within the sign (0–29)
  - Minutes within the degree (0–59), truncated integer
  - Seconds within the minute (0–59), truncated integer
  - Nakshatra number (1–27, Ashwini = 1)
  - Nakshatra name, translated
  - Pada within the nakshatra (1–4)
  - Navamsa (D9) zodiac sign number (1–12)
  - Navamsa zodiac sign name and two-letter abbreviation, both translated
- **FR-008**: For each of the ten bodies, the response MUST include the planet's translated full name and translated two-letter abbreviation.
- **FR-009**: The response MUST echo the input `lang` and `cityId`, and MUST include the resolved city fields: translated `cityName`, `region1`, `region2`, decimal `lat`, decimal `lng`, and `timezone` (IANA string).
- **FR-010**: An unknown or absent `cityId` MUST produce an error response (`{"error":"..."}`).
- **FR-011**: A malformed `localTime` string MUST produce an error response.
- **FR-012**: An unknown `lang` MUST silently fall back to English for all translated strings. No `[missing]` placeholder values are permitted for any standard planet name, zodiac sign, or nakshatra key.
- **FR-013**: The operation MUST be accessible through the existing WASM bridge under the operation name `"horoscope_positions"`.
- **FR-014**: All locale strings required by this feature — planet names and abbreviations (10 bodies), zodiac sign names and abbreviations (12 signs), and nakshatra names (27 stars) — MUST be added to the existing localization tables for both English and Telugu. This amounts to **71 keys** per language.
- **FR-015**: The Navamsa (D9) zodiac sign for each body MUST be computed using the standard Vedic D9 method: each zodiac sign is divided into 9 equal parts of 3°20'; the starting navamsa sign follows the Movable / Fixed / Dual (Chara / Sthira / Ubhaya) classification of the sign containing the body.

### Key Entities

- **HoroscopeRequest**: Inputs — `lang` (BCP-47 string), `cityId` (u16), `localTime` (local datetime string, no timezone suffix, format `YYYY-MM-DDTHH:MM:SS`)
- **HoroscopeResponse**: Echoed `lang` and `cityId`; resolved city context (`cityName`, `region1`, `region2`, `lat`, `lng`, `timezone`); a `planets` field that is a **keyed JSON object** mapping each canonical body name (English, e.g. `"Sun"`, `"Ascendant"`) to its **PlanetaryPosition** record. The ten keys are fixed: `"Ascendant"`, `"Sun"`, `"Moon"`, `"Mars"`, `"Mercury"`, `"Jupiter"`, `"Venus"`, `"Saturn"`, `"Rahu"`, `"Ketu"`.
- **PlanetaryPosition**: Complete sidereal position for one celestial body — translated name and abbreviation, absolute longitude, zodiac sign, sign-degrees, minutes, seconds, nakshatra, pada, navamsa sign — all translated per the request `lang`
- **CelestialBody**: One of the ten bodies — Ascendant, Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Given a known reference birth chart (published date, time, and city with known planetary positions), all ten computed sidereal longitudes match the reference values within ±0.02 degrees.
- **SC-002**: Every valid request returns all ten planet entries; each entry contains every required field — no field is absent, null, or empty in a valid response.
- **SC-003**: All translated strings for `lang=te` are in Telugu script; all strings for `lang=en` are in English. No `[missing]` placeholder appears for any standard planet name, zodiac sign, or nakshatra key.
- **SC-004**: An invalid `cityId` or a malformed `localTime` returns `{"error":"..."}` with a non-empty human-readable message rather than incorrect chart data.
- **SC-005**: The build fails if any required locale key (planet name, zodiac sign name, nakshatra name) is absent from the English locale table — preventing untranslated placeholders from reaching production.

## Assumptions

- City geographic coordinates are the zoom-7 tile-centre of the city's Web Mercator tile (same formula as the city-data feature). This precision (±22 km) is adequate for Ascendant calculation at birth-town granularity.
- `localTime` seconds are optional for callers — supplying `HH:MM:00` is valid for minute-precision birth times.
- A full IANA timezone database crate (`chrono-tz` or equivalent) is an accepted dependency for this feature. Unlike the city-data feature, the zero-new-crates constraint does not apply here — correct timezone conversion is a fundamental correctness requirement for astrology.
- DST ambiguity (clocks fall back) is resolved using the earlier UTC offset; DST gap (clocks spring forward) is resolved by advancing to the post-gap time. The engine handles this internally — the caller supplies only the naive local time.
- Rahu and Ketu are always exactly 180° apart (True Node pair); Ketu is derived from Rahu, not looked up independently.
- Only the Ascendant calculation uses the city's geographic coordinates; all other bodies are geocentric and depend only on the Julian Day.
- The Navamsa (D9) calculation uses the standard Vedic classification: Movable signs (Aries, Cancer, Libra, Capricorn) begin their navamsa sequence from Aries; Fixed signs (Taurus, Leo, Scorpio, Aquarius) from Capricorn; Dual signs (Gemini, Virgo, Sagittarius, Pisces) from Cancer.

## Clarifications

### Session 2026-03-15

- Q: How should `localTime` (city-local, no timezone suffix) be converted to UTC — caller supplies offset, hardcoded table, or a timezone library crate? → A: Use a full IANA timezone database crate (`chrono-tz` or equivalent); no new-crates restriction applies to this feature.
- Q: Should the ten planetary positions in the response be a JSON array or a keyed object? → A: Keyed object — `"planets": {"Sun": {...}, "Moon": {...}, ...}` with fixed English canonical keys.
- Q: Should the engine accept only birth dates, or any point in time (past/present/future)? → A: Any valid calendar date — no restriction; the same operation serves natal charts, transits, muhurta, and any time-specific query.
- Q: Which Swiss Ephemeris sidereal mode implements True Chitrapaksha Ayanamsa? → A: `SE_SIDM_TRUE_CITRA` (mode 27) — Spica/Chitra fixed at 0° Libra using true stellar position.
- Q: Is the Whole Sign house system (FR-005) confirmed, or should Placidus cusp degrees be computed? → A: Whole Sign confirmed — classical Vedic (Parashari) system; Ascendant degree returned as a position entry; house cusp degrees are out of scope.
