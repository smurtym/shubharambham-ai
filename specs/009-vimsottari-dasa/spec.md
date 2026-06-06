# Feature Specification: Vimsottari Dasa

**Feature Branch**: `009-vimsottari-dasa`  
**Created**: 2026-03-21  
**Status**: Draft  
**Input**: User description: "Rust backend feature — Vimsottari Dasa engine computing Mahadasa and Antardasa periods based on Moon's nakshatra at birth, with localized output through the existing WASM bridge."

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Compute Vimsottari Mahadasa and Antardasa periods for a birth chart (Priority: P1)

An astrologer (or an application on their behalf) provides a city, a local date and time, and a display language. They receive the complete sequence of Vimsottari Mahadasa periods covering 120 years from birth, each subdivided into 9 Antardasa sub-periods. The first Mahadasa is determined by the Moon's birth nakshatra lord and is proportionally shortened based on the Moon's advancement within that nakshatra. All period names are translated to the requested language.

**Why this priority**: This is the only capability of the feature. Without correct dasa period computation, the feature has no value.

**Independent Test**: Supply a known birth date, time, and city; verify that each Mahadasa's start/end dates and all Antardasa start/end dates match a published Vimsottari dasa reference for that birth moment.

**Acceptance Scenarios**:

1. **Given** `cityId` for Hyderabad, `localTime = "1997-03-07T20:34:00"`, and `lang=en`, **When** the `vimsottari_dasa` operation is invoked, **Then** the response contains a sequence of Mahadasas starting with Mars (1997-03-07 to 1999-01-18), followed by Rahu (1999-01-18 to 2017-01-18), Jupiter, Saturn, Mercury, Ketu, Venus, Sun, and Moon — the complete 120-year cycle.
2. **Given** the same inputs, **When** examining the Mars Mahadasa's Antardasas, **Then** the first Antardasa (Mars–Venus) runs 1997-03-07 to 1998-02-11, the second (Mars–Sun) runs 1998-02-11 to 1998-06-19, and the third (Mars–Moon) runs 1998-06-19 to 1999-01-18. All 9 Antardasas within each Mahadasa are present and their dates are contiguous.
3. **Given** `lang=te`, **When** the operation is invoked, **Then** all Mahadasa and Antardasa planet names are in Telugu script (e.g., "కుజ మహాదశ" for Mars Mahadasa, "శుక్ర అంతర్దశ" for Venus Antardasa). Month names in start/end dates are in Telugu.
4. **Given** an unknown `lang` (e.g., `lang=hi`), **When** the operation is invoked, **Then** all translated strings fall back to English — no `[missing]` values appear.
5. **Given** valid inputs, **When** the response is received, **Then** every Mahadasa's end date equals the next Mahadasa's start date, and every Antardasa's end date within a Mahadasa equals the next Antardasa's start date. The first Mahadasa starts on the birth date and the last Mahadasa ends exactly 120 solar years after the adjusted cycle start.

---

### User Story 2 — Reject invalid inputs with clear errors (Priority: P2)

A developer supplies an invalid city ID or a malformed time string and receives a structured error response rather than incorrect dasa periods.

**Why this priority**: Incorrect dasa periods can lead to wrong astrological predictions. Failing loudly on bad input is safer than silent corruption.

**Independent Test**: Supply a non-existent `cityId` and a malformed `localTime` string; assert that the response contains an `error` field with a non-empty descriptive message.

**Acceptance Scenarios**:

1. **Given** a `cityId` that does not exist in the city store, **When** the operation is invoked, **Then** the response is `{"error":"..."}` with a descriptive human-readable message.
2. **Given** a `localTime` string that is not a valid datetime (e.g., `"not-a-date"`), **When** the operation is invoked, **Then** the response is `{"error":"..."}` with a descriptive message.

---

### Edge Cases

- **Moon at exact nakshatra boundary (0° of a nakshatra)**: The remaining fraction is 1.0 (entire dasa period remains). The first Mahadasa equals the full dasa length for that lord.
- **Moon at end of a nakshatra (13°19'59")**: The remaining fraction is near 0. The first Mahadasa is nearly exhausted — only a fraction of a day remains. This rounds to 0 or 1 day.
- **Rounding of fractional days**: Dates are computed to whole days only. Fractional days are rounded (not truncated) to the nearest whole day. Sub-day precision (hours/minutes) is not reported.
- **Leap years and long periods**: A solar year is defined as exactly 365.2425 days. Over 120 years, cumulative day fractions must be handled correctly so that contiguous periods have no gaps or overlaps.
- **All 9 nakshatras as birth star**: Each of the 9 dasa lords (Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury) can be the starting lord. The cycle wraps around after Mercury back to Ketu.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The operation MUST accept four inputs: an operation name (`"vimsottari_dasa"`), a language code (`lang`, BCP-47), a 32-bit city identifier (`cityId`), and a local datetime string (`localTime`) in the format `YYYY-MM-DDTHH:MM:SS` without a timezone suffix. The timezone is derived from the city record.
- **FR-002**: The engine MUST compute the Moon's sidereal longitude for the given birth moment using the existing horoscope engine's infrastructure (True Chitrapaksha Ayanamsa, Swiss Ephemeris). The Moon's nakshatra (1–27) and its exact degree within that nakshatra MUST be determined.
- **FR-003**: The engine MUST identify the Vimsottari dasa lord of the Moon's birth nakshatra using this fixed mapping:
  - Ketu: Ashwini (1), Magha (10), Mula (19)
  - Venus: Bharani (2), Purva Phalguni (11), Purva Ashadha (20)
  - Sun: Krittika (3), Uttara Phalguni (12), Uttara Ashadha (21)
  - Moon: Rohini (4), Hasta (13), Shravana (22)
  - Mars: Mrigashira (5), Chitra (14), Dhanishtha (23)
  - Rahu: Ardra (6), Swati (15), Shatabhisha (24)
  - Jupiter: Punarvasu (7), Vishakha (16), Purva Bhadrapada (25)
  - Saturn: Pushya (8), Anuradha (17), Uttara Bhadrapada (26)
  - Mercury: Ashlesha (9), Jyeshtha (18), Revati (27)
- **FR-004**: The engine MUST compute the balance of the first Mahadasa at birth. The balance fraction is: `(nakshatra_end_degree − moon_longitude) / nakshatra_span`, where each nakshatra spans exactly 13°20' (13.333... degrees). The first Mahadasa's duration is this fraction multiplied by the lord's full dasa period in years.
- **FR-005**: The fixed dasa periods in solar years are: Sun 6, Moon 10, Mars 7, Rahu 18, Jupiter 16, Saturn 19, Mercury 17, Ketu 7, Venus 20. Total cycle: 120 solar years.
- **FR-006**: A solar year is defined as exactly 365.2425 days. Durations in years MUST be converted to days using this constant. All intermediate calculations (Mahadasa durations, Antardasa durations, cumulative boundary offsets) MUST retain full fractional-day precision. The full birth datetime (date **and** time-of-day) MUST be used as the base point for cumulative day addition — truncating the time component would shift all boundaries by up to ±1 day. Rounding to the nearest whole day MUST occur only at the final step when converting each cumulative datetime to an output date.
- **FR-007**: After the first (partial) Mahadasa, the remaining 8 Mahadasas follow the fixed Vimsottari sequence: Ketu → Venus → Sun → Moon → Mars → Rahu → Jupiter → Saturn → Mercury, wrapping around as needed. Each subsequent Mahadasa is the full dasa period for that lord.
- **FR-008**: Each full Mahadasa MUST be subdivided into exactly 9 Antardasas. The first Antardasa within a Mahadasa belongs to the Mahadasa lord itself. Subsequent Antardasas follow the same Vimsottari sequence (wrapping from the Mahadasa lord's position). For the first (partial) Mahadasa at birth, only the remaining Antardasas that fall after the birth date are included — preceding Antardasas that elapsed before birth are omitted. The first remaining Antardasa starts on the birth date.
- **FR-009**: The duration of each Antardasa within a Mahadasa is proportional: `antardasa_days = mahadasa_total_days × (antardasa_lord_years / 120)`. This value MUST be kept as a fractional day count throughout computation. Boundary datetimes are computed by cumulative addition of fractional durations to the birth datetime (date + time-of-day); the resulting datetime is rounded to the nearest whole day only when formatting the output date.
- **FR-010**: All Mahadasa and Antardasa start/end dates MUST be contiguous — each period's end date equals the next period's start date. No gaps or overlaps are permitted.
- **FR-011**: The response MUST include translated Mahadasa labels (e.g., "Venus Mahadasa" in English, "శుక్ర మహాదశ" in Telugu) and translated Antardasa labels (e.g., "Venus Antardasa" / "శుక్ర అంతర్దశ"). The planet name portion comes from the existing localization table.
- **FR-012**: Start and end dates MUST be formatted as `"YYYY MonthName DD"` where `MonthName` is the full translated month name (e.g., "March" in English, "మార్చి" in Telugu) and `DD` is zero-padded.
- **FR-013**: The operation MUST be accessible through the existing WASM bridge under the operation name `"vimsottari_dasa"`.
- **FR-014**: An unknown or absent `cityId` MUST produce an error response (`{"error":"..."}`).
- **FR-015**: A malformed `localTime` string MUST produce an error response.
- **FR-016**: An unknown `lang` MUST silently fall back to English for all translated strings. No `[missing]` placeholder values are permitted.
- **FR-017**: Localization keys for Mahadasa and Antardasa labels (`"dasa.maha"`, `"dasa.antar"`) and month names (`"month.1"` through `"month.12"`) MUST be added to the existing localization tables for both English and Telugu.
- **FR-018**: Only the standard birth-star-based (Janma Nakshatra Adhipati) Vimsottari calculation is implemented. Variants based on kshema, utpanna, and adhana stars are explicitly excluded.
- **FR-019**: The response MUST echo the input `lang` and `cityId`, and MUST include the resolved city fields: translated `cityName`, `region1`, `region2`, decimal `lat`, decimal `lng`, and `timezone` (IANA string) — identical to the horoscope positions response format.
- **FR-020**: A request body that cannot be deserialized into a `VimsottariRequest` (e.g., missing required fields, wrong field types, or malformed JSON syntax) MUST produce an error response (`{"error":"JSON parse error: <detail>"}`). This error is distinct from the field-level validation errors in FR-014 and FR-015.

### Key Entities

- **VimsottariRequest**: Inputs — `operation` (must equal `"vimsottari_dasa"`), `lang` (BCP-47 string), `cityId` (u32), `localTime` (local datetime string, `YYYY-MM-DDTHH:MM:SS`)
- **VimsottariResponse**: A JSON object containing echoed city context fields (`lang`, `cityId`, `cityName`, `region1`, `region2`, `lat`, `lng`, `timezone`) and a `periods` field that is an array of Mahadasa objects, each containing the Mahadasa label, start date, end date, and an array of Antardasa objects
- **MahadasaEntry**: Canonical planet key (`lord`, language-independent), translated Mahadasa label (e.g., "Venus Mahadasa"), formatted start date, formatted end date, array of Antardasa entries
- **AntardasaEntry**: Canonical planet key (`lord`, language-independent), translated Antardasa label (e.g., "Venus Antardasa"), formatted start date, formatted end date

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Given the reference birth chart (Hyderabad, 1997-03-07T20:34:00), the computed Mars Mahadasa runs 1997-03-07 to 1999-01-18 and its first three Antardasas (Mars–Venus, Mars–Sun, Mars–Moon) match the reference dates exactly (±0 days).
- **SC-002**: Given the same reference, the Rahu Mahadasa runs 1999-01-18 to 2017-01-18 and all 9 of its Antardasas match the reference dates exactly (±0 days).
- **SC-003**: Every full Mahadasa (2nd through 9th) contains exactly 9 Antardasa entries. The first (partial) Mahadasa contains only the remaining post-birth Antardasas. Every Mahadasa's end date equals the next Mahadasa's start date. No gaps or overlaps exist across the entire 120-year cycle.
- **SC-004**: All strings in the response for `lang=te` are in Telugu script. All strings for `lang=en` are in English. No `[missing]` placeholder appears.
- **SC-005**: An invalid `cityId`, a malformed `localTime`, or a malformed request JSON body returns `{"error":"..."}` with a non-empty descriptive message rather than incorrect dasa periods.

## Assumptions

- The Moon's sidereal longitude is obtained by reusing the existing horoscope engine's Swiss Ephemeris infrastructure (ayanamsa mode, Julian Day conversion, timezone handling). No separate Swiss Ephemeris initialization or ayanamsa configuration is needed — the Vimsottari engine can call into the same utilities.
- The nakshatra number (1–27) and Moon's exact degree within the nakshatra are derivable from `utils::decompose_longitude()` already implemented in feature 006.
- Planet name translations (English and Telugu) already exist in the localization tables from feature 006. Only Mahadasa/Antardasa label strings and month names need to be added.
- The response is a JSON object with city echo fields (matching horoscope positions format) and a `periods` array of Mahadasa entries. The caller receives city context alongside dasa data in a single response.
- Day arithmetic uses the Gregorian proleptic calendar with full datetime precision (date + time-of-day). The `chrono` crate's `NaiveDateTime` (already a dependency) provides correct datetime addition across leap years and century boundaries. The time-of-day component from `localTime` is preserved as the base point; only the final output dates are rounded to whole days.
- The total cycle is always 120 solar years from the adjusted start point (birth date minus the elapsed portion of the first Mahadasa). The last Mahadasa's end date falls on the 120-year anniversary of this adjusted start.

## Clarifications

### Session 2026-03-21

- Q: Should the first (partial) Mahadasa include all 9 Antardasas (with pre-birth dates) or only the remaining post-birth ones? → A: Only remaining post-birth Antardasas; first one starts on birth date.
- Q: How should fractional days be handled to guarantee contiguous dates without gaps or overlaps? → A: Keep all durations as fractional days throughout computation; use the full birth datetime (date + time-of-day) as the base for cumulative addition; round only at final date formatting.
- Q: Should the time-of-day component from `localTime` be preserved or truncated to date-only? → A: Preserved. The birth datetime (e.g., `1997-03-07T20:34:00`) is the base for all cumulative offsets. Truncating to midnight would shift every boundary by up to ±1 day.
- Q: Should the response include metadata (moon nakshatra, dasa lord, etc.) or remain a flat JSON array? → A: The `periods` array contains no metadata — only Mahadasa entries with their Antardasas. The response object additionally includes city echo fields (`lang`, `cityId`, `cityName`, etc.) per FR-019; partial first Mahadasa is self-evident from fewer Antardasas.
