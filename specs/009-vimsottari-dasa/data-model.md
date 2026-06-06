# Data Model: Vimsottari Dasa

**Branch**: `009-vimsottari-dasa` | **Date**: 2026-03-21

---

## Entities

### VimsottariRequest

Input to the `"vimsottari_dasa"` bridge operation.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `operation` | string | must equal `"vimsottari_dasa"` | Bridge operation name |
| `cityId` | u32 | must exist in CITIES store | 32-bit zoom-15 base-4 quadkey identifying the city |
| `localTime` | string | `YYYY-MM-DDTHH:MM:SS`, no tz suffix | Date and time in the city's local timezone |
| `lang` | string | BCP-47; unknown → falls back to `"en"` | Display language for all translated strings |

---

### VimsottariResponse

Top-level JSON output object.

| Field | Type | Description |
|-------|------|-------------|
| `lang` | string | Echoed input lang |
| `cityId` | u32 | Echoed input city ID |
| `cityName` | string | Translated city name |
| `region1` | string | Translated state/province |
| `region2` | string | Translated country |
| `lat` | f64 | City latitude (decimal degrees, 3 decimal places) |
| `lng` | f64 | City longitude (decimal degrees, 3 decimal places) |
| `timezone` | string | IANA timezone string (e.g. `"Asia/Kolkata"`) |
| `periods` | array | Ordered array of `MahadasaEntry` objects (9 entries, covering 120 solar years) |

---

### MahadasaEntry

One element in the `periods` array. Represents a Mahadasa period.

| Field | JSON key | Type | Description |
|-------|----------|------|-------------|
| Mahadasa label | `label` | string | Translated label: `"{planet_name} {dasa.maha}"` (e.g. "Venus Mahadasa" / "శుక్ర మహాదశ") |
| Start date | `startDate` | string | `"YYYY MonthName DD"` — translated month, zero-padded day |
| End date | `endDate` | string | Same format. Equals the next Mahadasa's `startDate`. |
| Lord key | `lord` | string | Canonical planet key (e.g. `"Venus"`, `"Mars"`) — language-independent |
| Antardasas | `antardasas` | array | Ordered array of `AntardasaEntry` objects. Full Mahadasas have 9; the first (partial) Mahadasa may have fewer. |

---

### AntardasaEntry

One element in the `antardasas` array within a Mahadasa. Represents an Antardasa sub-period.

| Field | JSON key | Type | Description |
|-------|----------|------|-------------|
| Antardasa label | `label` | string | Translated label: `"{planet_name} {dasa.antar}"` (e.g. "Venus Antardasa" / "శుక్ర అంతర్దశ") |
| Start date | `startDate` | string | `"YYYY MonthName DD"` — same format as Mahadasa dates |
| End date | `endDate` | string | Same format. Equals the next Antardasa's `startDate` within the Mahadasa. |
| Lord key | `lord` | string | Canonical planet key — language-independent |

---

### DasaLord (Rust enum — internal)

Maps Vimsottari lords to their dasa periods and planet keys.

| Variant | Planet key | Dasa years | Dasa days (×365.2425) |
|---------|-----------|------------|----------------------|
| Ketu | `"Ketu"` | 7 | 2556.6975 |
| Venus | `"Venus"` | 20 | 7304.85 |
| Sun | `"Sun"` | 6 | 2191.455 |
| Moon | `"Moon"` | 10 | 3652.425 |
| Mars | `"Mars"` | 7 | 2556.6975 |
| Rahu | `"Rahu"` | 18 | 6573.165 |
| Jupiter | `"Jupiter"` | 16 | 5843.88 |
| Saturn | `"Saturn"` | 19 | 6939.0075 |
| Mercury | `"Mercury"` | 17 | 6191.4225 |

**Sequence order** (FR-007): Ketu → Venus → Sun → Moon → Mars → Rahu → Jupiter → Saturn → Mercury → (wraps to Ketu)

---

## Constants

| Name | Value | Source |
|------|-------|--------|
| `SOLAR_YEAR_DAYS` | `365.2425` | FR-006 |
| `NAKSHATRA_SPAN` | `13.333...` (13 + 1/3) | 360° / 27 nakshatras |
| `TOTAL_CYCLE_YEARS` | `120` | Sum of all 9 dasa periods |
| `TOTAL_CYCLE_DAYS` | `43829.1` | 120 × 365.2425 |

---

## State Transitions / Processing Pipeline

```
VimsottariRequest (JSON)
  │
  ├─ Parse JSON → VimsottariRequest (error if malformed)
  ├─ Lookup cityId in CITIES → CityRecord (error if not found)
  ├─ Resolve lat/lng via decode_city_id()
  ├─ local_to_jd(localTime, city.timezone) → jd (error if malformed)
  │
  ├─ swe_set_sid_mode(SE_SIDM_TRUE_CITRA=27, 0.0, 0.0)
  ├─ swe_set_ephe_path("/ephe")
  ├─ calc_planet(jd, SE_MOON) → moon_lon (sidereal, f64)
  │
  ├─ decompose_longitude(moon_lon) → (..., nakshatra_num, ...)
  ├─ nakshatra_to_lord_index(nakshatra_num) → starting lord index
  │
  ├─ Compute balance fraction:
  │     nakshatra_start = (nakshatra_num - 1) × NAKSHATRA_SPAN
  │     balance = (nakshatra_start + NAKSHATRA_SPAN - moon_lon) / NAKSHATRA_SPAN
  │
  ├─ Parse birth datetime → NaiveDateTime (preserve time-of-day, do NOT truncate to date)
  │
  ├─ Build 9 Mahadasas (starting lord, then sequence wrapping):
  │     For each Mahadasa:
  │       duration_days = lord_years × SOLAR_YEAR_DAYS × (balance if first, else 1.0)
  │       Subdivide into Antardasas:
  │         antardasa_days = mahadasa_days × (antar_lord_years / 120)
  │       For first Mahadasa: skip elapsed Antardasas before birth
  │       Accumulate cumulative_days from birth_datetime (NaiveDateTime)
  │       Convert cumulative datetime → calendar date only at formatting (round to nearest day)
  │
  ├─ Format dates as "YYYY MonthName DD" with localized month names
  ├─ Assemble localized labels using get_string()
  │
  └─ Serialize VimsottariResponse → JSON string
```

---

## Validation Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `operation` must equal `"vimsottari_dasa"` | FR-001 | JSON parse + field check |
| `cityId` must exist in city store | FR-014 | Lookup or error |
| `localTime` must parse as `YYYY-MM-DDTHH:MM:SS` | FR-015 | `local_to_jd` or error |
| `lang` unknown → fallback to `"en"` | FR-016 | `get_string` fallback chain |
| Balance fraction ∈ (0.0, 1.0] | FR-004 | Mathematical guarantee from nakshatra position |
| All period boundaries contiguous | FR-010 | Cumulative offset + single rounding pass |
| 9 Antardasas per full Mahadasa | FR-008 | Loop invariant |
| First Mahadasa: ≤9 Antardasas | FR-008 | Skip elapsed, start from birth |
