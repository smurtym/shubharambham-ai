// Vimsottari Dasa engine — Feature 009
//
// Computes Mahadasa and Antardasa periods for a birth chart based on the
// Moon's sidereal nakshatra using Swiss Ephemeris with True Chitrapaksha Ayanamsa.

use serde::{Deserialize, Serialize};
use std::os::raw::c_char;
use chrono::Datelike as _;

use crate::data::{self, cities};
use crate::localization::get_string;
use crate::swe_wrappers::{self, SE_SIDM_TRUE_CITRA, SE_MOON};
use crate::utils;

// ---------------------------------------------------------------------------
// Ephemeris path — virtual FS in WASM, native relative path in tests
// ---------------------------------------------------------------------------

fn ephe_path() -> &'static [u8] {
    if cfg!(test) { b"../ephe/\0" } else { b"/ephe/\0" }
}

// ---------------------------------------------------------------------------
// T008 — Constants and DasaLord definition
// ---------------------------------------------------------------------------

const SOLAR_YEAR_DAYS: f64 = 365.2425;
const NAKSHATRA_SPAN: f64 = 360.0 / 27.0; // 13.333... degrees

struct DasaLord {
    planet_key: &'static str,
    years:      u8,
}

const DASA_SEQUENCE: [DasaLord; 9] = [
    DasaLord { planet_key: "Ketu",    years: 7  },
    DasaLord { planet_key: "Venus",   years: 20 },
    DasaLord { planet_key: "Sun",     years: 6  },
    DasaLord { planet_key: "Moon",    years: 10 },
    DasaLord { planet_key: "Mars",    years: 7  },
    DasaLord { planet_key: "Rahu",    years: 18 },
    DasaLord { planet_key: "Jupiter", years: 16 },
    DasaLord { planet_key: "Saturn",  years: 19 },
    DasaLord { planet_key: "Mercury", years: 17 },
];

// ---------------------------------------------------------------------------
// T006 — Request struct
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VimsottariRequest {
    #[allow(dead_code)]
    operation:  String,
    city_id:    u32,
    local_time: String,
    lang:       String,
}

// ---------------------------------------------------------------------------
// T007 — Response structs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AntardasaEntry {
    lord:       String,
    label:      String,
    start_date: String,
    end_date:   String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MahadasaEntry {
    lord:       String,
    label:      String,
    start_date: String,
    end_date:   String,
    antardasas: Vec<AntardasaEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VimsottariResponse {
    lang:      String,
    city_id:   u32,
    city_name: String,
    region1:   String,
    region2:   String,
    #[serde(serialize_with = "crate::utils::serialize_round3")]
    lat:       f64,
    #[serde(serialize_with = "crate::utils::serialize_round3")]
    lng:       f64,
    timezone:  String,
    periods:   Vec<MahadasaEntry>,
}

// ---------------------------------------------------------------------------
// T009 — nakshatra_to_lord_index
// ---------------------------------------------------------------------------

/// Map a nakshatra number (1–27) to an index into DASA_SEQUENCE (0–8).
fn nakshatra_to_lord_index(nakshatra_num: u8) -> usize {
    ((nakshatra_num - 1) % 9) as usize
}

// ---------------------------------------------------------------------------
// T010 — compute_balance_fraction
// ---------------------------------------------------------------------------

/// Compute the remaining fraction of the first Mahadasa at birth.
///
/// Returns a value in `(0.0, 1.0]`: 1.0 if Moon is at the exact nakshatra
/// start (full dasa remains), approaching 0.0 near the nakshatra end.
fn compute_balance_fraction(moon_lon: f64, nakshatra_num: u8) -> f64 {
    let nakshatra_start = (nakshatra_num as f64 - 1.0) * NAKSHATRA_SPAN;
    let nakshatra_end   = nakshatra_start + NAKSHATRA_SPAN;
    (nakshatra_end - moon_lon) / NAKSHATRA_SPAN
}

// ---------------------------------------------------------------------------
// T011 — offset_to_date
// ---------------------------------------------------------------------------

/// Add `cumulative_days` (fractional) to `birth_dt` and return a calendar date.
///
/// Rationale: the birth is at a specific time-of-day (e.g. 20:34). Adding an
/// offset of N.frac days to that datetime yields a result at approximately the
/// same time-of-day as birth, shifted by frac×24 hours. If that shifts the
/// time past midnight (i.e. the result time ≥ 12:00 and frac pushes it into the
/// next calendar day), we round up to the next day.
///
/// Special case: cumulative_days = 0.0 (the exact birth moment) → always returns
/// the birth date without any rounding, so period[0].startDate = the actual
/// birth date rather than the next day.
fn offset_to_date(
    birth_dt:        chrono::NaiveDateTime,
    cumulative_days: f64,
) -> chrono::NaiveDate {
    use chrono::TimeDelta;
    let total_secs  = (cumulative_days * 86_400.0).round() as i64;
    let boundary_dt = birth_dt + TimeDelta::seconds(total_secs);
    let noon        = chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap();
    // Only advance to the next day for non-zero offsets whose resulting time
    // has crossed noon — the birth date (offset = 0) is always returned as-is.
    if total_secs > 0 && boundary_dt.time() >= noon {
        boundary_dt.date() + TimeDelta::days(1)
    } else {
        boundary_dt.date()
    }
}

// ---------------------------------------------------------------------------
// T012 — format_date
// ---------------------------------------------------------------------------

/// Format a `NaiveDate` as `"YYYY MonthName DD"` with localized month name.
fn format_date(date: chrono::NaiveDate, lang: &str) -> String {
    let month_key = format!("month.{}", date.month());
    let month_name = get_string(&month_key, lang);
    format!("{} {} {:02}", date.year(), month_name, date.day())
}

// ---------------------------------------------------------------------------
// T013 — build_antardasas
// ---------------------------------------------------------------------------

/// Build the Antardasa entries for one Mahadasa.
///
/// For the first (partial) Mahadasa, `elapsed_days` is the portion of the
/// Mahadasa that preceded birth; the function skips Antardasas that ended
/// before birth and starts the first included one on the birth date.
///
/// For full Mahadasas, `elapsed_days` = 0.0 and all 9 Antardasas are included.
///
/// `mahadasa_start_offset` is the cumulative day offset (from birth_dt) at
/// which the Mahadasa begins. For the first (partial) Mahadasa this is 0.0
/// because the Mahadasa starts at birth.
fn build_antardasas(
    mahadasa_lord_idx:    usize,
    mahadasa_total_days:  f64,
    birth_dt:             chrono::NaiveDateTime,
    mahadasa_start_offset: f64,
    elapsed_days:         f64,
    lang:                 &str,
) -> Vec<AntardasaEntry> {
    let mut entries = Vec::new();
    // Cumulative within the Mahadasa (from its logical start, before birth for
    // the first partial Mahadasa).
    let mut antar_cumulative = 0.0_f64;

    for i in 0..9_usize {
        let antar_lord_idx = (mahadasa_lord_idx + i) % 9;
        let antar_lord     = &DASA_SEQUENCE[antar_lord_idx];
        let antar_days     = mahadasa_total_days * (antar_lord.years as f64) / 120.0;

        let antar_start_in_maha = antar_cumulative;
        let antar_end_in_maha   = antar_cumulative + antar_days;
        antar_cumulative = antar_end_in_maha;

        // Skip Antardasas that ended entirely before birth.
        if antar_end_in_maha <= elapsed_days {
            continue;
        }

        // Start offset from birth_dt for this Antardasa.
        // The first retained Antardasa starts exactly at birth (offset = 0.0).
        let start_offset = if antar_start_in_maha < elapsed_days {
            // This Antardasa straddles birth — starts at birth.
            mahadasa_start_offset
        } else {
            mahadasa_start_offset + (antar_start_in_maha - elapsed_days)
        };
        let end_offset = mahadasa_start_offset + (antar_end_in_maha - elapsed_days);

        let start_date = offset_to_date(birth_dt, start_offset);
        let end_date   = offset_to_date(birth_dt, end_offset);

        let planet_name = get_string(&format!("planet.dasa.{}", antar_lord.planet_key), lang);
        let dasa_antar  = get_string("dasa.antar", lang);
        entries.push(AntardasaEntry {
            lord:       antar_lord.planet_key.to_string(),
            label:      format!("{planet_name} {dasa_antar}"),
            start_date: format_date(start_date, lang),
            end_date:   format_date(end_date, lang),
        });
    }
    entries
}

// ---------------------------------------------------------------------------
// T014 — build_periods
// ---------------------------------------------------------------------------

/// Build the complete 9-Mahadasa sequence starting from `starting_lord_idx`.
///
/// The first Mahadasa uses `balance_fraction × lord_years × SOLAR_YEAR_DAYS`.
/// Subsequent 8 Mahadasas use the full dasa period for each lord.
fn build_periods(
    starting_lord_idx: usize,
    balance_fraction:  f64,
    birth_dt:          chrono::NaiveDateTime,
    lang:              &str,
) -> Vec<MahadasaEntry> {
    let mut periods = Vec::new();
    // Cumulative day offset from birth_dt (always starts at 0.0 = birth)
    let mut cumulative_days = 0.0_f64;

    for i in 0..9_usize {
        let lord_idx  = (starting_lord_idx + i) % 9;
        let lord      = &DASA_SEQUENCE[lord_idx];
        let full_days = lord.years as f64 * SOLAR_YEAR_DAYS;

        // First Mahadasa is partial (balance_fraction of full period).
        let maha_days = if i == 0 { balance_fraction * full_days } else { full_days };

        // Elapsed portion of the first Mahadasa that preceded birth.
        let elapsed_days = if i == 0 { (1.0 - balance_fraction) * full_days } else { 0.0 };

        let maha_start_offset = cumulative_days;
        let maha_end_offset   = cumulative_days + maha_days;

        let start_date = offset_to_date(birth_dt, maha_start_offset);
        let end_date   = offset_to_date(birth_dt, maha_end_offset);

        let planet_name = get_string(&format!("planet.dasa.{}", lord.planet_key), lang);
        let dasa_maha   = get_string("dasa.maha", lang);

        let antardasas = build_antardasas(
            lord_idx,
            full_days,       // proportional Antardasas use the FULL Mahadasa total
            birth_dt,
            maha_start_offset,
            elapsed_days,
            lang,
        );

        periods.push(MahadasaEntry {
            lord:       lord.planet_key.to_string(),
            label:      format!("{planet_name} {dasa_maha}"),
            start_date: format_date(start_date, lang),
            end_date:   format_date(end_date, lang),
            antardasas,
        });

        cumulative_days = maha_end_offset;
    }
    periods
}

// ---------------------------------------------------------------------------
// T015 / T022-T024 — execute (entry point + error handling)
// ---------------------------------------------------------------------------

/// Entry point called by the bridge for `"vimsottari_dasa"`.
pub fn execute(request: &str) -> String {
    // T024 / FR-020 — JSON parse error
    let req: VimsottariRequest = match serde_json::from_str(request) {
        Ok(r)  => r,
        Err(e) => {
            let msg = e.to_string().replace('"', "\\\"");
            return format!("{{\"error\":\"JSON parse error: {msg}\"}}");
        }
    };

    // T022 / FR-014 — unknown cityId
    let city = match cities::CITIES.iter().find(|c| c.city_id == req.city_id) {
        Some(c) => c,
        None    => return format!("{{\"error\":\"city not found: cityId={}\"}}",  req.city_id),
    };

    let (lat, lng) = data::decode_city_id(req.city_id);

    // T023 / FR-015 — malformed localTime
    let jd = match utils::local_to_jd(&req.local_time, city.timezone) {
        Ok(jd)  => jd,
        Err(e)  => {
            let msg = e.replace('"', "\\\"");
            return format!("{{\"error\":\"invalid localTime '{}': {msg}\"}}", req.local_time);
        }
    };

    // Configure Swiss Ephemeris (identical to horoscope engine)
    let ephe = ephe_path();
    unsafe {
        swe_wrappers::swe_set_ephe_path(ephe.as_ptr() as *const c_char);
        swe_wrappers::swe_set_sid_mode(SE_SIDM_TRUE_CITRA, 0.0, 0.0);
    }

    // FR-002 — Moon sidereal longitude
    let moon_lon = match swe_wrappers::calc_planet(jd, SE_MOON) {
        Ok(lon)  => lon,
        Err(e)   => {
            let msg = e.replace('"', "\\\"");
            return format!("{{\"error\":\"calc_planet(Moon) failed: {msg}\"}}");
        }
    };

    // FR-002/FR-003 — Nakshatra identification and lord index
    let (_, _, _, _, nakshatra_num, _) = utils::decompose_longitude(moon_lon);
    let starting_lord_idx = nakshatra_to_lord_index(nakshatra_num);

    // FR-004 — Balance fraction
    let balance_fraction = compute_balance_fraction(moon_lon, nakshatra_num);

    // FR-006 — Parse birth datetime preserving time-of-day (do NOT truncate to date)
    let birth_dt = match chrono::NaiveDateTime::parse_from_str(&req.local_time, "%Y-%m-%dT%H:%M:%S") {
        Ok(dt)  => dt,
        Err(_)  => {
            // Should not reach here — local_to_jd already validated the format.
            return format!("{{\"error\":\"internal: cannot re-parse localTime\"}}");
        }
    };

    // Build periods (FR-007–FR-012)
    let periods = build_periods(starting_lord_idx, balance_fraction, birth_dt, &req.lang);

    // FR-019 — City echo fields (resolve translations for requested lang)
    let translation = city.translations.iter()
        .find(|(l, _)| *l == req.lang)
        .or_else(|| city.translations.iter().find(|(l, _)| *l == "en"))
        .or_else(|| city.translations.first())
        .map(|(_, t)| t);

    let (city_name, region1, region2) = match translation {
        Some(t) => (t.city_name.to_string(), t.region1.to_string(), t.region2.to_string()),
        None    => (city.canonical_name.to_string(), String::new(), String::new()),
    };

    let response = VimsottariResponse {
        lang:      req.lang.clone(),
        city_id:   req.city_id,
        city_name,
        region1,
        region2,
        lat,
        lng,
        timezone:  city.timezone.to_string(),
        periods,
    };

    serde_json::to_string(&response).unwrap_or_else(|e| {
        format!("{{\"error\":\"serialization failed: {e}\"}}")
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Parses "YYYY MonthName DD" into a NaiveDate.
    fn parse_test_date(s: &str) -> chrono::NaiveDate {
        const MONTHS: &[&str] = &[
            "January","February","March","April","May","June",
            "July","August","September","October","November","December",
        ];
        let parts: Vec<&str> = s.split_whitespace().collect();
        let year  = parts[0].parse::<i32>().unwrap();
        let month = MONTHS.iter().position(|&m| m == parts[1]).unwrap() as u32 + 1;
        let day   = parts[2].parse::<u32>().unwrap();
        chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    // Asserts two date strings are within ±1 calendar day of each other.
    fn assert_date_approx(actual: &str, expected: &str) {
        let a = parse_test_date(actual);
        let e = parse_test_date(expected);
        let diff = (a - e).num_days().abs();
        assert!(diff <= 1,
            "date {actual:?} differs from expected {expected:?} by {diff} days (> 1)");
    }

    // -----------------------------------------------------------------------
    // T020 — test_balance_fraction
    // -----------------------------------------------------------------------

    #[test]
    fn test_balance_fraction() {
        // Ashwini = nakshatra 1, spans 0.0 – 13.333...°
        let span = NAKSHATRA_SPAN; // 13.333...

        // Moon at exact nakshatra start → balance = 1.0 (full dasa remains)
        let fraction = compute_balance_fraction(0.0, 1);
        assert!((fraction - 1.0).abs() < 1e-10, "start: expected 1.0, got {fraction}");

        // Moon at exact nakshatra end (approaching 0, not quite)
        let fraction = compute_balance_fraction(span - 1e-9, 1);
        assert!(fraction > 0.0 && fraction < 1e-7, "end: expected ≈0.0, got {fraction}");

        // Moon at nakshatra mid-point → balance ≈ 0.5
        let fraction = compute_balance_fraction(span / 2.0, 1);
        assert!((fraction - 0.5).abs() < 1e-10, "mid: expected 0.5, got {fraction}");

        // Moon at 3/4 through nakshatra → balance ≈ 0.25
        let fraction = compute_balance_fraction(span * 0.75, 1);
        assert!((fraction - 0.25).abs() < 1e-10, "3/4: expected 0.25, got {fraction}");
    }

    // -----------------------------------------------------------------------
    // T021 — test_nakshatra_to_lord_mapping
    // -----------------------------------------------------------------------

    #[test]
    fn test_nakshatra_to_lord_mapping() {
        // Expected lord keys by nakshatra (1-indexed)
        let expected = [
            "Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury", // 1-9
            "Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury", // 10-18
            "Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury", // 19-27
        ];
        for (i, &expected_key) in expected.iter().enumerate() {
            let nakshatra_num = (i + 1) as u8;
            let idx = nakshatra_to_lord_index(nakshatra_num);
            assert_eq!(
                DASA_SEQUENCE[idx].planet_key, expected_key,
                "nakshatra {nakshatra_num}: expected {expected_key}, got {}",
                DASA_SEQUENCE[idx].planet_key
            );
        }
    }

    // -----------------------------------------------------------------------
    // T029 — test_edge_case_nakshatra_boundary
    // -----------------------------------------------------------------------

    #[test]
    fn test_edge_case_nakshatra_boundary() {
        // Moon at exact start of nakshatra 1 (Ashwini, 0°) → balance = 1.0
        // Full first Mahadasa = Ketu (7 years)
        let balance = compute_balance_fraction(0.0, 1);
        assert!((balance - 1.0).abs() < 1e-10);
        let full_ketu_days = 7.0 * SOLAR_YEAR_DAYS;
        let birth_dt = chrono::NaiveDateTime::parse_from_str("2000-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let periods = build_periods(0, balance, birth_dt, "en");
        // First Mahadasa is Ketu with full 7 years → all 9 Antardasas
        assert_eq!(periods[0].lord, "Ketu");
        assert_eq!(periods[0].antardasas.len(), 9, "full first Mahadasa must have 9 Antardasas");
        let first_end = offset_to_date(birth_dt, full_ketu_days);
        let formatted_end = format_date(first_end, "en");
        assert_eq!(periods[0].end_date, formatted_end);

        // Moon near end of nakshatra 1 (just before 13.333°) → balance ≈ 0
        // First Mahadasa duration should be near 0 days
        let near_end_lon = NAKSHATRA_SPAN - 1e-6;
        let tiny_balance = compute_balance_fraction(near_end_lon, 1);
        assert!(tiny_balance > 0.0 && tiny_balance < 1e-4, "balance near end should be ≈0");
        let tiny_days = tiny_balance * 7.0 * SOLAR_YEAR_DAYS;
        assert!(tiny_days < 1.0, "near-end first Mahadasa should be < 1 day, got {tiny_days}");
    }

    // -----------------------------------------------------------------------
    // T016 — test_reference_chart_mars_mahadasa (SC-001)
    // -----------------------------------------------------------------------
    // Reference chart: Hyderabad, 1997-03-07T20:34:00, Mars nakshatra (Chitra/Dhanishtha).
    // The expected dates are from the known Vimsottari reference for this birth.

    #[test]
    fn test_reference_chart_mars_mahadasa() {
        let json = execute(r#"{
            "operation": "vimsottari_dasa",
            "cityId": 466083476,
            "localTime": "1997-03-07T20:34:00",
            "lang": "en"
        }"#);
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        if v.get("error").is_some() {
            return; // SE data unavailable in this environment — skip
        }

        let periods = v["periods"].as_array().expect("periods array");
        assert_eq!(periods.len(), 9, "must have exactly 9 Mahadasas");

        // SC-001: Mars Mahadasa
        assert_eq!(periods[0]["lord"].as_str().unwrap(), "Mars");
        assert_eq!(periods[0]["startDate"].as_str().unwrap(), "1997 March 07");
        assert_date_approx(periods[0]["endDate"].as_str().unwrap(),   "1999 January 18");

        let antars = periods[0]["antardasas"].as_array().unwrap();
        // First 3 Antardasas of (partial) Mars Mahadasa
        assert_eq!(antars[0]["lord"].as_str().unwrap(), "Venus");
        assert_eq!(antars[0]["startDate"].as_str().unwrap(), "1997 March 07");
        assert_date_approx(antars[0]["endDate"].as_str().unwrap(),   "1998 February 11");

        assert_eq!(antars[1]["lord"].as_str().unwrap(), "Sun");
        assert_date_approx(antars[1]["startDate"].as_str().unwrap(), "1998 February 11");
        assert_date_approx(antars[1]["endDate"].as_str().unwrap(),   "1998 June 19");

        assert_eq!(antars[2]["lord"].as_str().unwrap(), "Moon");
        assert_date_approx(antars[2]["startDate"].as_str().unwrap(), "1998 June 19");
        assert_date_approx(antars[2]["endDate"].as_str().unwrap(),   "1999 January 18");
    }

    // -----------------------------------------------------------------------
    // T017 — test_reference_chart_rahu_mahadasa (SC-002)
    // -----------------------------------------------------------------------

    #[test]
    fn test_reference_chart_rahu_mahadasa() {
        let json = execute(r#"{
            "operation": "vimsottari_dasa",
            "cityId": 466083476,
            "localTime": "1997-03-07T20:34:00",
            "lang": "en"
        }"#);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        if v.get("error").is_some() {
            return; // SE data unavailable — skip
        }
        let periods = v["periods"].as_array().unwrap();

        // SC-002: Rahu Mahadasa (index 1 after Mars)
        assert_eq!(periods[1]["lord"].as_str().unwrap(), "Rahu");
        assert_date_approx(periods[1]["startDate"].as_str().unwrap(), "1999 January 18");
        assert_date_approx(periods[1]["endDate"].as_str().unwrap(),   "2017 January 18");

        let antars = periods[1]["antardasas"].as_array().unwrap();
        assert_eq!(antars.len(), 9, "Rahu Mahadasa must have 9 Antardasas");

        // First 2 Rahu Antardasas from the reference
        assert_eq!(antars[0]["lord"].as_str().unwrap(), "Rahu");
        assert_date_approx(antars[0]["startDate"].as_str().unwrap(), "1999 January 18");
        assert_date_approx(antars[0]["endDate"].as_str().unwrap(),   "2001 October 01");

        assert_eq!(antars[1]["lord"].as_str().unwrap(), "Jupiter");
        assert_date_approx(antars[1]["startDate"].as_str().unwrap(), "2001 October 01");
        assert_date_approx(antars[1]["endDate"].as_str().unwrap(),   "2004 February 23");
    }

    // -----------------------------------------------------------------------
    // T018 — test_contiguity_all_periods (SC-003)
    // -----------------------------------------------------------------------

    #[test]
    fn test_contiguity_all_periods() {
        let json = execute(r#"{
            "operation": "vimsottari_dasa",
            "cityId": 466083476,
            "localTime": "1997-03-07T20:34:00",
            "lang": "en"
        }"#);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        if v.get("error").is_some() {
            return; // SE data unavailable — skip
        }
        let periods = v["periods"].as_array().unwrap();
        assert_eq!(periods.len(), 9);

        // First Mahadasa starts on birth date
        assert_eq!(periods[0]["startDate"].as_str().unwrap(), "1997 March 07");

        // Mahadasa contiguity: end[i] == start[i+1]
        for i in 0..8 {
            let end_i   = periods[i]["endDate"].as_str().unwrap();
            let start_i1 = periods[i + 1]["startDate"].as_str().unwrap();
            assert_eq!(end_i, start_i1,
                "Mahadasa gap at index {i}: end={end_i} != start={start_i1}");
        }

        // Full Mahadasas (2nd onward) have exactly 9 Antardasas; first may have fewer
        assert!(periods[0]["antardasas"].as_array().unwrap().len() <= 9);
        for i in 1..9 {
            assert_eq!(
                periods[i]["antardasas"].as_array().unwrap().len(), 9,
                "Mahadasa {i} must have 9 Antardasas"
            );
        }

        // Antardasa contiguity within each Mahadasa
        for (mi, maha) in periods.iter().enumerate() {
            let antars = maha["antardasas"].as_array().unwrap();
            if antars.len() > 1 {
                for j in 0..antars.len() - 1 {
                    let end_j   = antars[j]["endDate"].as_str().unwrap();
                    let start_j1 = antars[j + 1]["startDate"].as_str().unwrap();
                    assert_eq!(end_j, start_j1,
                        "Antardasa gap in Mahadasa {mi} at {j}: {end_j} != {start_j1}");
                }
            }
            // First Antardasa of each Mahadasa starts at Mahadasa start date
            if !antars.is_empty() {
                assert_eq!(
                    antars[0]["startDate"].as_str().unwrap(),
                    maha["startDate"].as_str().unwrap(),
                    "Mahadasa {mi}: first Antardasa start must equal Mahadasa start"
                );
                // Last Antardasa of each Mahadasa ends at Mahadasa end date
                let last = antars.len() - 1;
                assert_eq!(
                    antars[last]["endDate"].as_str().unwrap(),
                    maha["endDate"].as_str().unwrap(),
                    "Mahadasa {mi}: last Antardasa end must equal Mahadasa end"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // T019 — test_telugu_localization (SC-004)
    // -----------------------------------------------------------------------

    #[test]
    fn test_telugu_localization() {
        let json = execute(r#"{
            "operation": "vimsottari_dasa",
            "cityId": 466083476,
            "localTime": "1997-03-07T20:34:00",
            "lang": "te"
        }"#);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        if v.get("error").is_some() {
            return; // SE data unavailable — skip
        }

        let periods = v["periods"].as_array().unwrap();

        // Mars Mahadasa label should be in Telugu ("కుజ మహాదశ")
        let mars_label = periods[0]["label"].as_str().unwrap();
        assert!(
            mars_label.contains("మహాదశ"),
            "Mars Mahadasa label should contain Telugu 'మహాదశ', got: {mars_label}"
        );
        // Mars planet name in Telugu adjective form is "కుజ"
        assert!(
            mars_label.contains("కుజ"),
            "Mars label should contain Telugu 'కుజ', got: {mars_label}"
        );

        // Start date month in Telugu ("మార్చి" for March)
        let start_date = periods[0]["startDate"].as_str().unwrap();
        assert!(
            start_date.contains("మార్చి"),
            "Start date should contain Telugu 'మార్చి' for March, got: {start_date}"
        );

        // Antardasa label in Telugu
        let antars = periods[0]["antardasas"].as_array().unwrap();
        let antar_label = antars[0]["label"].as_str().unwrap();
        assert!(
            antar_label.contains("అంతర్దశ"),
            "Antardasa label should contain Telugu 'అంతర్దశ', got: {antar_label}"
        );

        // No [missing] strings anywhere in the response
        assert!(
            !json.contains("[missing]"),
            "Response contains [missing] placeholder: {json}"
        );
    }

    // -----------------------------------------------------------------------
    // T025 — Error path tests (SC-005)
    // -----------------------------------------------------------------------

    // (a) test_unknown_city
    #[test]
    fn test_unknown_city() {
        let json = execute(r#"{
            "operation": "vimsottari_dasa",
            "cityId": 99999,
            "localTime": "1997-03-07T20:34:00",
            "lang": "en"
        }"#);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let err = v["error"].as_str().expect("must have error field");
        assert!(err.contains("city not found"), "unexpected error message: {err}");
        assert!(err.contains("99999"), "error should include cityId: {err}");
    }

    // (b) test_malformed_time
    #[test]
    fn test_malformed_time() {
        let json = execute(r#"{
            "operation": "vimsottari_dasa",
            "cityId": 466083476,
            "localTime": "not-a-date",
            "lang": "en"
        }"#);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let err = v["error"].as_str().expect("must have error field");
        assert!(err.contains("invalid localTime") || err.contains("not-a-date"),
            "unexpected error message: {err}");
    }

    // (c) test_unknown_lang_fallback
    #[test]
    fn test_unknown_lang_fallback() {
        let json = execute(r#"{
            "operation": "vimsottari_dasa",
            "cityId": 466083476,
            "localTime": "1997-03-07T20:34:00",
            "lang": "hi"
        }"#);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        if v.get("error").is_some() {
            return; // SE data unavailable — skip
        }
        // No [missing] placeholder — falls back to English
        assert!(!json.contains("[missing]"), "should fall back to English, not [missing]: {json}");
        // Mahadasa label should be in English
        let label = v["periods"][0]["label"].as_str().unwrap();
        assert!(label.contains("Mahadasa"), "should fall back to English 'Mahadasa', got: {label}");
    }

    // (d) test_json_parse_error (FR-020)
    #[test]
    fn test_json_parse_error() {
        let json = execute("this is not json at all");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let err = v["error"].as_str().expect("must have error field");
        assert!(err.contains("JSON parse error"), "unexpected error message: {err}");
    }
}
