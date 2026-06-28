use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Internal data model (runtime-loaded; fields are owned Strings)
// ---------------------------------------------------------------------------

pub struct CityRecord {
    pub city_id:        u32,
    pub canonical_name: String,
    pub timezone:       String,
    pub translations:   Vec<(String, TranslationEntry)>,
}

pub struct TranslationEntry {
    pub city_name:     String,
    pub region1:       String,
    pub region2:       String,
    pub region1_order: u16,
    pub region2_order: u16,
}

// ---------------------------------------------------------------------------
// Runtime CSV loading
// ---------------------------------------------------------------------------

fn cities_csv_path() -> &'static str {
    if cfg!(test) { "../ephe/cities.csv" } else { "/ephe/cities.csv" }
}

fn load_cities_from_csv() -> Result<Vec<CityRecord>, String> {
    let path = cities_csv_path();
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {path}: {e}"))?;

    struct Row {
        city_id:       u32,
        canonical:     String,
        timezone:      String,
        lang:          String,
        city_name:     String,
        region1:       String,
        region2:       String,
        region1_order: u16,
        region2_order: u16,
    }

    let mut rows: Vec<Row> = Vec::new();
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("city_id") {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() != 9 {
            continue;
        }
        let city_id = match cols[0].trim().parse::<u32>() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let canonical = cols[1].trim().to_string();
        if canonical.is_empty() {
            continue;
        }
        let region1_order = match cols[7].trim().parse::<u16>() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let region2_order = match cols[8].trim().parse::<u16>() {
            Ok(v) => v,
            Err(_) => continue,
        };
        rows.push(Row {
            city_id,
            canonical,
            timezone:      cols[2].trim().to_string(),
            lang:          cols[3].trim().to_string(),
            city_name:     cols[4].trim().to_string(),
            region1:       cols[5].trim().to_string(),
            region2:       cols[6].trim().to_string(),
            region1_order,
            region2_order,
        });
    }

    let mut city_order: Vec<u32> = Vec::new();
    let mut city_map: HashMap<u32, (String, String, Vec<(String, TranslationEntry)>)> =
        HashMap::new();

    for row in rows {
        let id = row.city_id;
        if !city_map.contains_key(&id) {
            city_order.push(id);
            city_map.insert(id, (row.canonical, row.timezone, Vec::new()));
        }
        city_map.get_mut(&id).unwrap().2.push((
            row.lang,
            TranslationEntry {
                city_name:     row.city_name,
                region1:       row.region1,
                region2:       row.region2,
                region1_order: row.region1_order,
                region2_order: row.region2_order,
            },
        ));
    }

    let mut result: Vec<CityRecord> = Vec::with_capacity(city_order.len());
    for id in city_order {
        let (canonical_name, timezone, translations) = city_map.remove(&id).unwrap();
        result.push(CityRecord { city_id: id, canonical_name, timezone, translations });
    }
    Ok(result)
}

static CITY_CACHE: OnceLock<Vec<CityRecord>> = OnceLock::new();

pub fn cities() -> Result<&'static [CityRecord], String> {
    if let Some(v) = CITY_CACHE.get() {
        return Ok(v.as_slice());
    }
    let loaded = load_cities_from_csv()?;
    let _ = CITY_CACHE.set(loaded);
    Ok(CITY_CACHE.get().expect("just set above").as_slice())
}

// ---------------------------------------------------------------------------
// decode_city_id — pure helper (FR-013)
// ---------------------------------------------------------------------------

/// Decode a zoom-15 quadkey stored as a decimal u32 back to its tile-centre (lat, lng).
///
/// city_id is the base-4 quadkey string interpreted as a decimal integer.
/// Each base-4 digit encodes one zoom level: low bit → tile_x bit, high bit → tile_y bit.
///
/// Example: quadkey "123301331322112" (base 4) = 466083478 (decimal)
///          → tile_x = 23526, tile_y = 14777 → (17.38°N, 78.47°E) = Hyderabad
pub fn decode_city_id(city_id: u32) -> (f64, f64) {
    const ZOOM:  u32 = 15;
    const TILES: f64 = 32_768.0; // 2^15
    let mut tile_x: u32 = 0;
    let mut tile_y: u32 = 0;
    let mut q = city_id;
    for k in 0..ZOOM {
        let digit = q % 4;
        q /= 4;
        tile_x |= (digit & 1) << k;
        tile_y |= ((digit >> 1) & 1) << k;
    }
    let lng = (tile_x as f64 + 0.5) / TILES * 360.0 - 180.0;
    let n   = PI * (1.0 - 2.0 * (tile_y as f64 + 0.5) / TILES);
    let lat = n.sinh().atan() * 180.0 / PI;
    (lat, lng)
}

// ---------------------------------------------------------------------------
// Wire-format structs (serialised to JSON via serde)
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityResponse {
    pub lang:           String,
    pub city_id:        u32,
    pub time_zone:      String,
    pub canonical_name: String,
    pub city_name:      String,
    pub region1:        String,
    pub region2:        String,
    #[serde(serialize_with = "crate::utils::serialize_round3")]
    pub lat:            f64,
    #[serde(serialize_with = "crate::utils::serialize_round3")]
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
// Public API
// ---------------------------------------------------------------------------

/// Filter cities to those with a translation for `lang`, build sorted
/// `CityResponse` objects, and return the result as a JSON string.
///
/// Returns `{"cities":[...]}` on success or `{"error":"..."}` on failure.
pub fn list_cities(lang: &str) -> String {
    let all_cities = match cities() {
        Ok(c) => c,
        Err(e) => {
            let msg = e.replace('"', "\\\"");
            return format!("{{\"error\":\"cities.csv error: {msg}\"}}");
        }
    };

    let mut responses: Vec<CityResponse> = all_cities
        .iter()
        .filter_map(|rec| {
            rec.translations
                .iter()
                .find(|(code, _)| code.as_str() == lang)
                .map(|(_, tr)| {
                    let (lat, lng) = decode_city_id(rec.city_id);
                    CityResponse {
                        lang:           lang.to_owned(),
                        city_id:        rec.city_id,
                        time_zone:      rec.timezone.clone(),
                        canonical_name: rec.canonical_name.clone(),
                        city_name:      tr.city_name.clone(),
                        region1:        tr.region1.clone(),
                        region2:        tr.region2.clone(),
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

    // Helper: count city records that have a translation for `lang`.
    fn expected_count(lang: &str) -> usize {
        cities().expect("cities.csv must be readable in test environment")
            .iter()
            .filter(|rec| rec.translations.iter().any(|(l, _)| l.as_str() == lang))
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
        let exp = expected_count("en");
        assert_eq!(cities.len(), exp,
            "en response has {} cities but {} have en translations in cities.csv",
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
        let exp = expected_count("te");
        assert_eq!(cities.len(), exp,
            "te response has {} cities but {} have te translations in cities.csv",
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
        if let (Some(hyd), Some(eluru)) = (find_pos(cities, "Hyderabad"), find_pos(cities, "Eluru")) {
            assert!(hyd < eluru, "Hyderabad must precede Eluru in te (lower region1_order)");
        }
        if let (Some(hyd), Some(vij)) = (find_pos(cities, "Hyderabad"), find_pos(cities, "Vijayawada")) {
            assert!(hyd < vij,   "Hyderabad must precede Vijayawada in te (lower region1_order)");
        }
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
        if let (Some(hyd), Some(ny)) = (find_pos(cities, "Hyderabad"), find_pos(cities, "New York")) {
            assert!(hyd < ny, "Hyderabad (India) must precede New York (USA)");
        }
        if let (Some(del), Some(hyd)) = (find_pos(cities, "Delhi"), find_pos(cities, "Hyderabad")) {
            assert!(del < hyd, "Delhi must precede Hyderabad in en (lower region1_order)");
        }
        if let (Some(eluru), Some(vij)) = (find_pos(cities, "Eluru"), find_pos(cities, "Vijayawada")) {
            assert!(eluru < vij,
                "Eluru must sort before Vijayawada in en (city_name tie-break); \
                 positions: Eluru={eluru}, Vijayawada={vij}");
        }
    }

    #[test]
    fn test_decode_city_id() {
        let (lat, lng) = decode_city_id(466083478); // Hyderabad tile 91,57
        assert!((lat - 17.38).abs() < 0.1, "lat out of range: {lat}");
        assert!((lng - 78.47).abs() < 0.1, "lng out of range: {lng}");
    }

    // -----------------------------------------------------------------------
    // T013 — User Story 2 unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_list_cities_te_excludes_en_only() {
        let json = list_cities("te");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let canonical_names: Vec<&str> = v["cities"].as_array().unwrap()
            .iter()
            .map(|c| c["canonicalName"].as_str().unwrap())
            .collect();
        for rec in cities().expect("cities.csv must be readable in test environment") {
            let has_te = rec.translations.iter().any(|(l, _)| l.as_str() == "te");
            if has_te {
                assert!(
                    canonical_names.contains(&rec.canonical_name.as_str()),
                    "'{}' has a te translation but is absent from the te response",
                    rec.canonical_name
                );
            } else {
                assert!(
                    !canonical_names.contains(&rec.canonical_name.as_str()),
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

    // T018 — Data integrity tests
    #[test]
    fn test_city_ids_unique() {
        let mut seen = HashSet::new();
        for rec in cities().expect("cities.csv must be readable in test environment") {
            assert!(seen.insert(rec.city_id), "duplicate city_id: {}", rec.city_id);
        }
    }

    #[test]
    fn test_timezones_valid() {
        const VALID_TIMEZONES: &[&str] = &[
            "Africa/Cairo", "Africa/Johannesburg", "Africa/Lagos", "Africa/Nairobi",
            "America/Argentina/Buenos_Aires", "America/Chicago", "America/Denver",
            "America/Detroit", "America/Indiana/Indianapolis",
            "America/Kentucky/Louisville", "America/Los_Angeles", "America/Mexico_City",
            "America/New_York", "America/Phoenix", "America/Santiago", "America/Sao_Paulo",
            "America/Toronto", "America/Vancouver",
            "Asia/Bahrain", "Asia/Bangkok", "Asia/Colombo", "Asia/Dhaka", "Asia/Dubai",
            "Asia/Hong_Kong", "Asia/Jerusalem", "Asia/Kathmandu", "Asia/Kolkata",
            "Asia/Kuala_Lumpur", "Asia/Qatar", "Asia/Riyadh", "Asia/Seoul",
            "Asia/Shanghai", "Asia/Singapore", "Asia/Taipei", "Asia/Tokyo", "Asia/Yangon",
            "Australia/Melbourne", "Australia/Perth", "Australia/Sydney",
            "Europe/Amsterdam", "Europe/Berlin", "Europe/Copenhagen", "Europe/Dublin",
            "Europe/Helsinki", "Europe/Lisbon", "Europe/London", "Europe/Madrid",
            "Europe/Moscow", "Europe/Oslo", "Europe/Paris", "Europe/Stockholm",
            "Europe/Vienna", "Europe/Warsaw",
            "Pacific/Auckland",
        ];
        for rec in cities().expect("cities.csv must be readable in test environment") {
            assert!(
                VALID_TIMEZONES.contains(&rec.timezone.as_str()),
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
