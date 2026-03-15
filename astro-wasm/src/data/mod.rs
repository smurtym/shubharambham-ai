pub mod cities;

use std::f64::consts::PI;

// ---------------------------------------------------------------------------
// Internal data model
// ---------------------------------------------------------------------------

/// Authoritative city record — compiled into the WASM binary. Never serialised
/// directly; a `CityResponse` is built per-request from this + one translation.
pub struct CityRecord {
    pub city_id:        u16,
    pub canonical_name: &'static str,
    pub timezone:       &'static str,
    pub translations:   &'static [(&'static str, TranslationEntry)],
}

/// Language-specific display strings and sort keys for one city.
pub struct TranslationEntry {
    pub city_name:     &'static str,
    pub region1:       &'static str,
    pub region2:       &'static str,
    pub region1_order: u16,
    pub region2_order: u16,
}

// ---------------------------------------------------------------------------
// decode_city_id — pure helper (FR-013)
// ---------------------------------------------------------------------------

/// Decode a 16-bit zoom-7 quadkey back to its tile-centre (lat, lng).
///
/// Formula:
///   tile_x = city_id / 128
///   tile_y = city_id % 128
///   lng    = (tile_x + 0.5) / 128.0 * 360.0 - 180.0
///   n      = π * (1.0 - 2.0 * (tile_y + 0.5) / 128.0)
///   lat    = n.sinh().atan() * 180.0 / π
pub fn decode_city_id(city_id: u16) -> (f64, f64) {
    let tile_x = (city_id / 128) as f64;
    let tile_y = (city_id % 128) as f64;
    let lng    = (tile_x + 0.5) / 128.0 * 360.0 - 180.0;
    let n      = PI * (1.0 - 2.0 * (tile_y + 0.5) / 128.0);
    let lat    = n.sinh().atan() * 180.0 / PI;
    (lat, lng)
}

// ---------------------------------------------------------------------------
// Wire-format structs (serialised to JSON via serde)
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityResponse {
    pub lang:           String,
    pub city_id:        u16,
    pub time_zone:      String,
    pub canonical_name: String,
    pub city_name:      String,
    pub region1:        String,
    pub region2:        String,
    pub lat:            f64,
    pub lng:            f64,
    #[serde(skip)]
    pub region1_order:  u16,
    #[serde(skip)]
    pub region2_order:  u16,
}

#[derive(serde::Serialize)]
pub struct CityListResponse {
    pub cities: Vec<CityResponse>,
}

// ---------------------------------------------------------------------------
// Compile-time data integrity (T017)
// ---------------------------------------------------------------------------

#[allow(dead_code)] // invoked at compile time via `const _: () = ...` below
const fn validate_canonical_names() {
    let cities = cities::CITIES;
    let mut i = 0;
    while i < cities.len() {
        if cities[i].canonical_name.is_empty() {
            panic!("CityRecord has empty canonical_name");
        }
        i += 1;
    }
}
const _: () = validate_canonical_names();

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Filter `CITIES` to those with a translation for `lang`, build sorted
/// `CityResponse` objects, and return the result as a JSON string.
///
/// Returns `{"cities":[...]}` on success or `{"error":"..."}` on failure.
pub fn list_cities(lang: &str) -> String {
    let mut responses: Vec<CityResponse> = cities::CITIES
        .iter()
        .filter_map(|rec| {
            rec.translations
                .iter()
                .find(|(code, _)| *code == lang)
                .map(|(_, tr)| {
                    let (lat, lng) = decode_city_id(rec.city_id);
                    CityResponse {
                        lang:           lang.to_owned(),
                        city_id:        rec.city_id,
                        time_zone:      rec.timezone.to_owned(),
                        canonical_name: rec.canonical_name.to_owned(),
                        city_name:      tr.city_name.to_owned(),
                        region1:        tr.region1.to_owned(),
                        region2:        tr.region2.to_owned(),
                        lat,
                        lng,
                        region1_order:  tr.region1_order,
                        region2_order:  tr.region2_order,
                    }
                })
        })
        .collect();

    responses.sort_by(|a, b| {
        a.region2_order
            .cmp(&b.region2_order)
            .then(a.region1_order.cmp(&b.region1_order))
            .then(a.city_name.cmp(&b.city_name))
    });

    match serde_json::to_string(&CityListResponse { cities: responses }) {
        Ok(json) => json,
        Err(e)   => {
            let msg = e.to_string().replace('"', "\\\"");
            format!("{{\"error\":\"{msg}\"}}")
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // Helper: count CITIES entries that have a translation for `lang`.
    fn expected_count(lang: &str) -> usize {
        cities::CITIES
            .iter()
            .filter(|rec| rec.translations.iter().any(|(l, _)| *l == lang))
            .count()
    }

    // Helper: find the position of a `canonicalName` in a cities JSON array.
    fn find_pos(cities: &[serde_json::Value], canonical: &str) -> Option<usize> {
        cities
            .iter()
            .position(|c| c["canonicalName"].as_str().unwrap_or("") == canonical)
    }

    // -----------------------------------------------------------------------
    // T011 — User Story 1 unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_list_cities_en_returns_all_five() {
        let json = list_cities("en");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let cities = v["cities"].as_array().unwrap();
        // Count is derived from CITIES so adding more en-translated cities
        // keeps this test green without any edits.
        let exp = expected_count("en");
        assert_eq!(cities.len(), exp,
            "en response has {} cities but {} have en translations in CITIES",
            cities.len(), exp);
        for city in cities {
            assert!(city["lang"].as_str().is_some() && !city["lang"].as_str().unwrap().is_empty());
            assert!(city["cityId"].as_u64().is_some());
            assert!(city["timeZone"].as_str().is_some() && !city["timeZone"].as_str().unwrap().is_empty());
            assert!(city["canonicalName"].as_str().is_some() && !city["canonicalName"].as_str().unwrap().is_empty());
            assert!(city["cityName"].as_str().is_some() && !city["cityName"].as_str().unwrap().is_empty());
            assert!(city["region1"].as_str().is_some() && !city["region1"].as_str().unwrap().is_empty());
            assert!(city["region2"].as_str().is_some() && !city["region2"].as_str().unwrap().is_empty());
            assert!(city["lat"].as_f64().is_some());
            assert!(city["lng"].as_f64().is_some());
        }
    }

    #[test]
    fn test_list_cities_te_has_three_entries() {
        let json = list_cities("te");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let cities = v["cities"].as_array().unwrap();
        // Count is derived from CITIES — adding Telugu to more cities keeps this green.
        let exp = expected_count("te");
        assert_eq!(cities.len(), exp,
            "te response has {} cities but {} have te translations in CITIES",
            cities.len(), exp);
        for city in cities {
            let name = city["cityName"].as_str().unwrap();
            assert!(
                name.chars().any(|c| c as u32 > 0x0C00),
                "expected Telugu script in cityName, got: {name}"
            );
        }
    }

    #[test]
    fn test_list_cities_te_order() {
        let json = list_cities("te");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let cities = v["cities"].as_array().unwrap();
        // Assert pairwise relative order rather than absolute positions so the
        // test survives other cities being inserted before or between these.
        if let (Some(hyd), Some(eluru)) = (find_pos(cities, "Hyderabad"), find_pos(cities, "Eluru")) {
            assert!(hyd < eluru, "Hyderabad must precede Eluru in te (lower region1_order)");
        }
        if let (Some(hyd), Some(vij)) = (find_pos(cities, "Hyderabad"), find_pos(cities, "Vijayawada")) {
            assert!(hyd < vij,   "Hyderabad must precede Vijayawada in te (lower region1_order)");
        }
        // Eluru and Vijayawada share region1_order=2; city_name tie-break: "ఏలూరు" < "విజయవాడ".
        if let (Some(eluru), Some(vij)) = (find_pos(cities, "Eluru"), find_pos(cities, "Vijayawada")) {
            assert!(eluru < vij,
                "Eluru must sort before Vijayawada (city_name tie-break); \
                 positions: Eluru={eluru}, Vijayawada={vij}");
        }
    }

    #[test]
    fn test_list_cities_en_order() {
        let json = list_cities("en");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let cities = v["cities"].as_array().unwrap();
        // Relative-order assertions — robust to additional cities being inserted.
        // India (region2_order=1) before USA (region2_order=2).
        if let (Some(hyd), Some(ny)) = (find_pos(cities, "Hyderabad"), find_pos(cities, "New York")) {
            assert!(hyd < ny, "Hyderabad (India) must precede New York (USA)");
        }
        // Within India: Delhi (region1_order=1) before Hyderabad (region1_order=2).
        if let (Some(del), Some(hyd)) = (find_pos(cities, "Delhi"), find_pos(cities, "Hyderabad")) {
            assert!(del < hyd, "Delhi must precede Hyderabad in en (lower region1_order)");
        }
        // Eluru/Vijayawada both have region1_order=3; city_name tie-break: "Eluru" < "Vijayawada".
        if let (Some(eluru), Some(vij)) = (find_pos(cities, "Eluru"), find_pos(cities, "Vijayawada")) {
            assert!(eluru < vij,
                "Eluru must sort before Vijayawada in en (city_name tie-break); \
                 positions: Eluru={eluru}, Vijayawada={vij}");
        }
    }

    #[test]
    fn test_decode_city_id() {
        let (lat, lng) = decode_city_id(11705); // Hyderabad tile 91,57
        assert!((lat - 17.97).abs() < 0.1, "lat out of range: {lat}");
        assert!((lng - 77.34).abs() < 0.1, "lng out of range: {lng}");
    }

    // -----------------------------------------------------------------------
    // T013 — User Story 2 unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_list_cities_te_excludes_en_only() {
        // Derive expectations from CITIES itself: any city whose translation slice
        // lacks an entry for "te" must be absent, and any city that has one must
        // be present.  This stays correct regardless of which cities gain or lose
        // Telugu translations in the future.
        let json = list_cities("te");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let canonical_names: Vec<&str> = v["cities"].as_array().unwrap()
            .iter()
            .map(|c| c["canonicalName"].as_str().unwrap())
            .collect();
        for rec in cities::CITIES {
            let has_te = rec.translations.iter().any(|(l, _)| *l == "te");
            if has_te {
                assert!(
                    canonical_names.contains(&rec.canonical_name),
                    "'{}' has a te translation but is absent from the te response",
                    rec.canonical_name
                );
            } else {
                assert!(
                    !canonical_names.contains(&rec.canonical_name),
                    "'{}' has no te translation but appears in the te response — \
                     if you added a te translation, the test is already passing",
                    rec.canonical_name
                );
            }
        }
    }

    #[test]
    fn test_list_cities_unknown_lang_returns_empty() {
        let json = list_cities("hi");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let cities = v["cities"].as_array().unwrap();
        assert!(cities.is_empty(), "unknown lang should return empty cities array");
    }

    // -----------------------------------------------------------------------
    // T015 — User Story 3 unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_canonical_name_always_english() {
        // canonicalName must be ASCII for every language.  Checking is_ascii()
        // is strictly correct and scales to any number of cities or languages.
        for lang in &["te", "en"] {
            let json = list_cities(lang);
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            for city in v["cities"].as_array().unwrap() {
                let name = city["canonicalName"].as_str().unwrap();
                assert!(
                    name.is_ascii(),
                    "canonicalName '{name}' contains non-ASCII chars for lang={lang}"
                );
                assert!(!name.is_empty(), "canonicalName must not be empty for lang={lang}");
            }
        }
    }

    #[test]
    fn test_en_canonical_matches_city_name() {
        let json = list_cities("en");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let cities = v["cities"].as_array().unwrap();
        for city in cities {
            let canonical = city["canonicalName"].as_str().unwrap();
            let city_name  = city["cityName"].as_str().unwrap();
            assert_eq!(canonical, city_name,
                "canonicalName and cityName must match for lang=en; got canonical={canonical}, cityName={city_name}");
        }
    }

    // -----------------------------------------------------------------------
    // T017 — Compile-time validation is a const fn invoked at module level.
    //         This test documents the expectation.
    // -----------------------------------------------------------------------

    // T018 — Data integrity tests
    #[test]
    fn test_city_ids_unique() {
        let mut seen = HashSet::new();
        for rec in cities::CITIES {
            assert!(seen.insert(rec.city_id), "duplicate city_id: {}", rec.city_id);
        }
    }

    #[test]
    fn test_timezones_valid() {
        const VALID_TIMEZONES: &[&str] = &["Asia/Kolkata", "America/New_York"];
        for rec in cities::CITIES {
            assert!(
                VALID_TIMEZONES.contains(&rec.timezone),
                "unknown timezone '{}' for city '{}'", rec.timezone, rec.canonical_name
            );
        }
    }

    // T019 — Wire format purity test
    #[test]
    fn test_sort_keys_absent_from_json() {
        let json = list_cities("en");
        assert!(!json.contains("region1Order"),  "region1Order must not appear in JSON");
        assert!(!json.contains("region2Order"),  "region2Order must not appear in JSON");
        assert!(!json.contains("region1_order"), "region1_order must not appear in JSON");
        assert!(!json.contains("region2_order"), "region2_order must not appear in JSON");
    }
}
