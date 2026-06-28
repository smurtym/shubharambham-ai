# Data Model: Runtime City CSV

**Feature**: `012-runtime-city-csv` | **Date**: 2026-06-27

## Overview

The city data model is restructured from compile-time static arrays to runtime-loaded owned data. The wire format (JSON response) is unchanged. The CSV format is unchanged.

---

## Struct: `CityRecord` (modified in `data/mod.rs`)

**Before** (compile-time, `&'static str` fields):
```
city_id:        u32
canonical_name: &'static str
timezone:       &'static str
translations:   &'static [(&'static str, TranslationEntry)]
```

**After** (runtime-loaded, owned fields):

| Field | Type | Source |
|-------|------|--------|
| `city_id` | `u32` | CSV col 0 — decimal u32 quadkey |
| `canonical_name` | `String` | CSV col 1 |
| `timezone` | `String` | CSV col 2 |
| `translations` | `Vec<(String, TranslationEntry)>` | Grouped from CSV rows with same `city_id` |

**Validation during load** (replaces compile-time `const fn`):
- Rows with wrong column count: skipped
- Rows with unparseable `city_id`, `region1_order`, `region2_order`: skipped
- Duplicate `city_id` + `lang` pair: second occurrence wins (last-write per language)
- Empty `canonical_name`: skipped

---

## Struct: `TranslationEntry` (modified in `data/mod.rs`)

**Before**: `&'static str` fields

**After**:

| Field | Type | Source |
|-------|------|--------|
| `city_name` | `String` | CSV col 4 |
| `region1` | `String` | CSV col 5 |
| `region2` | `String` | CSV col 6 |
| `region1_order` | `u16` | CSV col 7 |
| `region2_order` | `u16` | CSV col 8 |

---

## Runtime Cache: `CITY_CACHE` (new in `data/mod.rs`)

| Attribute | Value |
|-----------|-------|
| Type | `std::sync::OnceLock<Vec<CityRecord>>` |
| Scope | Module-level static |
| Initialization | Lazy — on first call to `data::cities()` |
| Lifetime | Same as the process / WASM execution context |
| Thread safety | Guaranteed by `OnceLock` |

**Public accessor**:
```
pub fn cities() -> Result<&'static [CityRecord], String>
```
Returns `Ok(&[CityRecord])` after successful load, `Err(String)` if `cities.csv` cannot be read.

---

## CSV File: `ephe/cities.csv`

Format is **unchanged** from `data/cities.csv`. Reproduced for reference:

| Column | Index | Type | Description |
|--------|-------|------|-------------|
| `city_id` | 0 | u32 | Zoom-15 base-4 quadkey as decimal |
| `canonical_name` | 1 | String | English city name (ASCII) |
| `timezone` | 2 | String | IANA timezone identifier |
| `lang` | 3 | String | BCP-47 language code (`en`, `te`, …) |
| `city_name` | 4 | String | Localised city name |
| `region1` | 5 | String | State/province in requested language |
| `region2` | 6 | String | Country in requested language |
| `region1_order` | 7 | u16 | Sort key within a country |
| `region2_order` | 8 | u16 | Sort key between countries |

- Comment lines start with `#`; header line starts with `city_id` — both skipped
- Multiple rows with the same `city_id` form one `CityRecord` with multiple translations
- Delimiter: comma (`,`); no commas allowed inside field values

---

## Wire Format: `CityResponse` and `CityListResponse` (unchanged)

These structs in `data/mod.rs` are **not changed** — they are already owned-string structs with `serde::Serialize`. The JSON output shape is identical to the current behaviour.

---

## Removed Entities

| Entity | Location | Reason |
|--------|----------|--------|
| `pub static CITIES: &[CityRecord]` | `data/cities.rs` | Replaced by `CITY_CACHE` + `data::cities()` |
| `fn validate_canonical_names()` | `data/mod.rs` | Compile-time const fn; replaced by runtime skip-on-empty |
| `const _: () = validate_canonical_names()` | `data/mod.rs` | Same |
| `pub mod cities` | `data/mod.rs` line 1 | Module deleted |
| `fn generate_cities()` | `build.rs` | Build-time codegen removed |
| `fn escape_str()` | `build.rs` | Helper for codegen; removed |
