# Quickstart: City Data

**Branch**: `004-city-data`  
**Date**: 2026-03-15

---

## What this feature adds

A `list_cities` operation on the existing WASM bridge. Call it with a language code; get back a sorted array of city objects with localized names. All data is compiled into the WASM binary — no network call needed after the initial binary load.

---

## Files changed

| File | Change |
|------|--------|
| `astro-wasm/src/lib.rs` | Add `pub mod data;` |
| `astro-wasm/src/bridge.rs` | Add `"list_cities"` dispatch arm |
| `astro-wasm/src/data/mod.rs` | **NEW** — `CityRecord`, `TranslationEntry`, `CityResponse`, `CityListResponse` structs; `list_cities()` function |
| `astro-wasm/src/data/cities.rs` | **NEW** — `CITIES: &[CityRecord]` hardcoded 4-city array |
| `web/data.js` | Add `listCities(lang)` function |

---

## Adding a new city

1. Open `astro-wasm/src/data/cities.rs`.
2. Append a new `CityRecord` entry to the `CITIES` slice:

```rust
CityRecord {
    city_id: <unique_u32_quadkey>,
    canonical_name: "CityName",
    timezone: "Region/City",
    translations: &[
        ("en", TranslationEntry {
            city_name: "English Name",
            region1: "State",
            region2: "Country",
            region1_order: <u16>,
            region2_order: <u16>,
        }),
        ("te", TranslationEntry {
            city_name: "తెలుగు పేరు",
            region1: "రాష్ట్రం",
            region2: "దేశం",
            region1_order: <u16>,
            region2_order: <u16>,
        }),
    ],
},
```

3. Run `cargo test` — the data integrity tests will catch duplicate `city_id`, empty `canonical_name`, or unknown timezone.
4. Rebuild the WASM binary: `./build.sh`

> **Note on `city_id`**: Use the 32-bit base-4 zoom-15 quadkey for the city's geographic tile. Obtain the 15-digit base-4 quadkey string at zoom 15 (e.g., from https://labs.mapbox.com/what-the-tile/), then read those digits as a plain decimal integer. See `decode_city_id()` in `astro-wasm/src/data/mod.rs` for the inverse formula.

> **Note on sort keys**: `region2_order` controls country grouping (e.g., all India cities before USA cities). `region1_order` controls state grouping within a country. Lower numbers appear first. Use consistent values across entries (e.g., `region2_order: 1` for all India entries in Telugu).

---

## Adding a new language

1. Add a new `TranslationEntry` inside the `translations` slice of each city that should appear in the new language.
2. If a city has no translation for the new language, it is automatically excluded from responses for that language — no code change needed.
3. Add the new timezone to `VALID_TIMEZONES` in `data/mod.rs` if any new city uses a timezone not yet in the list.
4. Run `cargo test` and rebuild.

---

## Testing

**Unit tests** (run with `cargo test`):

| Test | Location | Validates |
|------|----------|-----------|
| `test_city_ids_unique` | `data/mod.rs` | No duplicate `city_id` values |
| `test_timezones_valid` | `data/mod.rs` | All timezones in `VALID_TIMEZONES` list |
| `test_list_cities_en_returns_all_four` | `data/mod.rs` | All 4 seed cities in English response |
| `test_list_cities_te_returns_only_two` | `data/mod.rs` | Delhi + New York absent from Telugu response |
| `test_list_cities_unknown_lang_empty` | `data/mod.rs` | Unknown language returns empty array |
| `test_list_cities_te_order` | `data/mod.rs` | Hyderabad before Vijayawada in Telugu response |
| `test_sort_keys_absent_from_json` | `data/mod.rs` | `region1Order` / `region2Order` not in serialized JSON |
| `test_bridge_list_cities_en` | `bridge.rs` | End-to-end bridge dispatch for `list_cities` |
| Compile-time | `data/cities.rs` | `canonical_name` non-empty (const fn) |

**E2E test** (Playwright): Update `tests/astro.spec.ts` to call `listCities("te")` via the bridge and assert Hyderabad and Vijayawada are present with Telugu names, and Delhi is absent.

---

## Calling from JavaScript

```javascript
// Returns Promise<Array<CityObject>>
const cities = await listCities('te');

// Each city object:
// {
//   lang: "te",
//   cityId: 12345,
//   timeZone: "Asia/Kolkata",
//   canonicalName: "Hyderabad",   // always English
//   cityName: "హైదరాబాద్",
//   region1: "తెలంగాణ",
//   region2: "భారతదేశం"
// }
```

`listCities` throws an `Error` if the bridge returns `{"error": "..."}`. An empty array is returned normally when the language has no matching cities — this is not an error.
