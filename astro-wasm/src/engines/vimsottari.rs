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

const SOLAR_YEAR_DAYS: f64 = 365.256363;
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
    is_current: bool,  // true if today falls within this Antardasa's date range; computed by Rust
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MahadasaEntry {
    lord:       String,
    label:      String,
    start_date: String,
    end_date:   String,
    is_current: bool,  // true if any child AntardasaEntry.is_current is true; derived by Rust
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

        let today = chrono::Local::now().date_naive();
        let is_current = start_date <= today && today < end_date;

        let planet_name = get_string(&format!("planet.dasa.{}", antar_lord.planet_key), lang);
        let dasa_antar  = get_string("dasa.antar", lang);
        entries.push(AntardasaEntry {
            lord:       antar_lord.planet_key.to_string(),
            label:      format!("{planet_name} {dasa_antar}"),
            start_date: format_date(start_date, lang),
            end_date:   format_date(end_date, lang),
            is_current,
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

        let is_current = antardasas.iter().any(|a| a.is_current);

        periods.push(MahadasaEntry {
            lord:       lord.planet_key.to_string(),
            label:      format!("{planet_name} {dasa_maha}"),
            start_date: format_date(start_date, lang),
            end_date:   format_date(end_date, lang),
            is_current,
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
// TR003 — Unit tests for is_period_current helper logic
// ---------------------------------------------------------------------------

/// Pure helper extracted for testability: true if today is within [start, end).
fn is_period_current(start: chrono::NaiveDate, end: chrono::NaiveDate, today: chrono::NaiveDate) -> bool {
    start <= today && today < end
}

#[cfg(test)]
mod tests {
    use super::is_period_current;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn today_in_range_is_true() {
        assert!(is_period_current(d(2020, 1, 1), d(2025, 1, 1), d(2022, 6, 15)));
    }

    #[test]
    fn today_equals_start_date_inclusive() {
        assert!(is_period_current(d(2022, 6, 15), d(2025, 1, 1), d(2022, 6, 15)));
    }

    #[test]
    fn today_equals_end_date_exclusive() {
        assert!(!is_period_current(d(2020, 1, 1), d(2022, 6, 15), d(2022, 6, 15)));
    }

    #[test]
    fn today_before_range_is_false() {
        assert!(!is_period_current(d(2025, 1, 1), d(2030, 1, 1), d(2020, 6, 15)));
    }

    #[test]
    fn today_after_range_is_false() {
        assert!(!is_period_current(d(2010, 1, 1), d(2015, 1, 1), d(2020, 6, 15)));
    }
}
