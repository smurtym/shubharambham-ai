# Data Model: City Data

**Phase**: 1  
**Branch**: `004-city-data`  
**Date**: 2026-03-15

---

## Entities

### CityRecord *(internal — Rust data store)*

The authoritative city entry in the hardcoded static array. Never serialized directly to JSON.

| Field | Rust Type | Description |
|-------|-----------|-------------|
| `city_id` | `u32` | 32-bit base-4 zoom-15 quadkey stored as decimal. Unique across all records. Decoded to lat/lng by `decode_city_id()`. |
| `canonical_name` | `&'static str` | City name always in English. Non-empty. Never changes with language. |
| `timezone` | `&'static str` | IANA timezone identifier (e.g., `"Asia/Kolkata"`). |
| `translations` | `&'static [(&'static str, TranslationEntry)]` | Ordered slice of (language-code, name-set) pairs. May be empty. |

**Invariants enforced at compile/test time**:
- `city_id` values unique across all records (`#[test]`)
- `canonical_name` non-empty for every record (`const fn` at compile time)
- `timezone` value present in `VALID_TIMEZONES` list (`#[test]`)

---

### TranslationEntry *(internal — nested in CityRecord)*

Language-specific display and ordering data for one city in one language.

| Field | Rust Type | Description |
|-------|-----------|-------------|
| `city_name` | `&'static str` | City name in this language. |
| `region1` | `&'static str` | Administrative subdivision (state/province) in this language. |
| `region2` | `&'static str` | Country name in this language. |
| `region1_order` | `u16` | Sort key for state/province grouping (lower = listed first). Language-specific. |
| `region2_order` | `u16` | Sort key for country grouping (lower = listed first). Language-specific. |

**Note**: `region1_order` and `region2_order` are internal sort keys only. They are **never** serialized to the JSON response.

---

### CityResponse *(external — JSON wire format)*

Ephemeral Rust struct constructed per-request from `CityRecord` + one `TranslationEntry`. Serialized to JSON via `#[derive(serde::Serialize)]`.

| Field | Rust Type | JSON key | Description |
|-------|-----------|----------|-------------|
| `lang` | `String` | `"lang"` | Echoes the requested language code. |
| `city_id` | `u32` | `"cityId"` | The `CityRecord::city_id` promoted to u32 for JSON (JS `number` type). |
| `time_zone` | `String` | `"timeZone"` | IANA timezone from `CityRecord`. |
| `canonical_name` | `String` | `"canonicalName"` | English name from `CityRecord`. |
| `city_name` | `String` | `"cityName"` | Translated name from `TranslationEntry`. |
| `region1` | `String` | `"region1"` | Translated state/province from `TranslationEntry`. |
| `region2` | `String` | `"region2"` | Translated country from `TranslationEntry`. |
| `lat` | `f64` | `"lat"` | Tile-centre latitude (decimal degrees) decoded from `city_id` via `decode_city_id`. Not stored in `CityRecord`. |
| `lng` | `f64` | `"lng"` | Tile-centre longitude (decimal degrees) decoded from `city_id` via `decode_city_id`. Not stored in `CityRecord`. |
| `region1_order` | `u16` | *(skipped)* | `#[serde(skip)]` — used for sorting only. |
| `region2_order` | `u16` | *(skipped)* | `#[serde(skip)]` — used for sorting only. |

---

### decode_city_id *(internal — pure function in `data/mod.rs`)*

Stateless helper that maps a 32-bit zoom-15 quadkey back to its tile-centre coordinates. Called when building each `CityResponse`; the result is placed directly into `lat`/`lng`.

```
pub fn decode_city_id(city_id: u32) -> (f64, f64)
```

| Step | Formula |
|------|--------|
| Decompose | Decode 15 base-4 digits (bit-interleaved): each digit `d = q % 4; q /= 4`; low bit → `tile_x`, high bit → `tile_y` |
| Longitude | `lng = (tile_x as f64 + 0.5) / 32768.0 * 360.0 − 180.0` |
| Latitude | `n = π × (1.0 − 2.0 × (tile_y as f64 + 0.5) / 32768.0)` → `lat = atan(sinh(n)) × 180.0 / π` |

Returns `(lat, lng)` in decimal degrees. Produces tile-centre values — not exact city coordinates.

**Precomputed values for the seed dataset**:

| canonical_name | city_id | lat | lng |
|----------------|---------|-----|-----|
| Hyderabad | 11705 | 17.97 | 77.34 |
| Vijayawada | 11834 | 15.31 | 80.16 |
| Eluru | 11833 | 18.01 | 80.16 |
| Delhi | 11701 | 28.17 | 77.34 |
| New York | 4784 | 39.96 | −74.53 |

---

### CityListResponse *(external — top-level JSON wrapper)*

Top-level success response. Consistent with the existing bridge object-at-root shape.

| Field | Rust Type | JSON key | Description |
|-------|-----------|----------|-------------|
| `cities` | `Vec<CityResponse>` | `"cities"` | Sorted, filtered array of city objects. May be empty. |

Success shape: `{"cities": [{...}, ...]}`  
Error shape (reuses existing bridge pattern): `{"error": "<message>"}`

---

## Seed Dataset

Initial hardcoded `CITIES` array (4 cities):

| city_id | canonical_name | timezone | Languages |
|---------|---------------|----------|-----------|
| 11705 | Hyderabad | Asia/Kolkata | te, en |
| 11834 | Vijayawada | Asia/Kolkata | te, en |
| 11833 | Eluru | Asia/Kolkata | te, en |
| 11701 | Delhi | Asia/Kolkata | en only |
| 4784 | New York | America/New_York | en only |

> **Note**: `city_id` values are zoom-15 base-4 quadkeys stored as decimal u32. See `data/cities.csv` for the authoritative current values and `decode_city_id()` in `astro-wasm/src/data/mod.rs` for the encoding/decoding algorithm. The seed cityId values in the table above reflect the original spec estimates and have since been superseded by the correct zoom-15 values in `cities.csv`.

**Sort order example for `lang="te"`** (Telangana-first, then city-name tie-break within Andhra Pradesh):

| city_name (te) | region2_order | region1_order | tie-break |
|----------------|--------------|--------------|----------|
| హైదరాబాద్ (Hyderabad) | 1 (India first) | 1 (Telangana first) | — |
| ఏలూరు (Eluru) | 1 (India first) | 2 (Andhra Pradesh second) | "ఏ" < "వి" |
| విజయవాడ (Vijayawada) | 1 (India first) | 2 (Andhra Pradesh second) | "వి" > "ఏ" |

**Sort order example for `lang="en"`** (alphabetical by region, then city-name tie-break within Andhra Pradesh):

| city_name | region2_order | region1_order | tie-break |
|-----------|--------------|--------------|----------|
| Delhi | 1 (India) | 1 (Delhi state) | — |
| Hyderabad | 1 (India) | 2 (Telangana) | — |
| Eluru | 1 (India) | 3 (Andhra Pradesh) | "E" < "V" |
| Vijayawada | 1 (India) | 3 (Andhra Pradesh) | "V" > "E" |
| New York | 2 (USA) | 1 (New York state) | — |

---

## Relationships

```
CityRecord (1) ──── (*) TranslationEntry
    ↑ indexed by (lang_code: &'static str)

CityRecord → [filtered by lang] → CityResponse
CityResponse → [sorted by (region2_order, region1_order, city_name)] → CityListResponse

CityListResponse → serialized by serde_json → JSON string → bridge output buffer
```

---

## Validation Rules

| Rule | Entity | Enforcement |
|------|--------|-------------|
| `city_id` unique | CityRecord | `#[test] fn test_city_ids_unique()` |
| `canonical_name` non-empty | CityRecord | `const fn validate_canonical_names()` at compile time |
| `timezone` valid IANA string | CityRecord | `#[test] fn test_timezones_valid()` against `VALID_TIMEZONES` list |
| `region1_order` / `region2_order` absent from JSON | CityResponse | `#[serde(skip)]` — compiler-enforced |
| Response array always present (may be empty) | CityListResponse | Structural guarantee from `Vec<CityResponse>` |
