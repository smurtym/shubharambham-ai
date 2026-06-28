// Vimsottari Dasa engine — Feature 009
//
// Computes Mahadasa and Antardasa periods for a birth chart based on the
// Moon's sidereal nakshatra using Swiss Ephemeris with True Chitrapaksha Ayanamsa.
//
// Period durations are measured by actual Sun travel in tropical degrees rather
// than fixed calendar years. One Vimsottari year = Sun traveling 360° (one
// tropical revolution). This means a "6-year Sun dasa" lasts exactly as long as
// it takes the Sun to travel 6 × 360 = 2160° from the dasa's start, capturing
// the Sun's true orbital speed variation (faster near perihelion in January,
// slower near aphelion in July).

use serde::{Deserialize, Serialize};
use std::os::raw::c_char;
use chrono::Datelike as _;

use crate::city_data;
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
// Constants and DasaLord definition
// ---------------------------------------------------------------------------

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
// Request struct
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
// Response structs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AntardasaEntry {
    lord:       String,
    label:      String,
    start_date: String,
    end_date:   String,
    is_current: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MahadasaEntry {
    lord:       String,
    label:      String,
    start_date: String,
    end_date:   String,
    is_current: bool,
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
// nakshatra_to_lord_index
// ---------------------------------------------------------------------------

fn nakshatra_to_lord_index(nakshatra_num: u8) -> usize {
    ((nakshatra_num - 1) % 9) as usize
}

// ---------------------------------------------------------------------------
// compute_balance_fraction
// ---------------------------------------------------------------------------

/// Returns the fraction (0, 1] of the first Mahadasa remaining at birth.
fn compute_balance_fraction(moon_lon: f64, nakshatra_num: u8) -> f64 {
    let nakshatra_start = (nakshatra_num as f64 - 1.0) * NAKSHATRA_SPAN;
    let nakshatra_end   = nakshatra_start + NAKSHATRA_SPAN;
    (nakshatra_end - moon_lon) / NAKSHATRA_SPAN
}

// ---------------------------------------------------------------------------
// advance_sun_degrees
// ---------------------------------------------------------------------------

/// Find the JD at which the Sun (tropical longitude) has traveled exactly
/// `degrees` from `start_jd`.
///
/// Uses Newton's method against `calc_sun_longitude`. Convergence is typically
/// 3–5 iterations; the loop cap of 30 is a safety net.
/// Precision: converges to within 0.0001 JD (~8.6 seconds).
fn advance_sun_degrees(start_jd: f64, degrees: f64) -> Result<f64, String> {
    // Mean sidereal motion: ~0.9856 deg/day (sidereal year = 365.256363 days)
    const SIDEREAL_YEAR: f64 = 365.256363;
    const MEAN_MOTION: f64 = 360.0 / SIDEREAL_YEAR;

    let start_lon = swe_wrappers::calc_planet(start_jd, swe_wrappers::SE_SUN)?;
    let target_lon = (start_lon + degrees % 360.0).rem_euclid(360.0);

    // Initial estimate via mean motion
    let mut jd = start_jd + degrees / MEAN_MOTION;

    for _ in 0..30 {
        let lon = swe_wrappers::calc_planet(jd, swe_wrappers::SE_SUN)?;
        // Signed angular difference, normalised to (−180, 180]
        let mut delta = target_lon - lon;
        if delta >  180.0 { delta -= 360.0; }
        if delta < -180.0 { delta += 360.0; }
        let delta_jd = delta / MEAN_MOTION;
        jd += delta_jd;
        if delta_jd.abs() < 0.0001 { break; }
    }

    Ok(jd)
}

// ---------------------------------------------------------------------------
// jd_to_local_date
// ---------------------------------------------------------------------------

/// Convert a Julian Day (UT) to the calendar date in the given IANA timezone.
fn jd_to_local_date(jd: f64, tz: &chrono_tz::Tz) -> chrono::NaiveDate {
    use chrono::{DateTime, Utc};
    let unix_secs = ((jd - 2440587.5) * 86400.0).round() as i64;
    let utc: DateTime<Utc> = DateTime::from_timestamp(unix_secs, 0)
        .unwrap_or(DateTime::UNIX_EPOCH);
    utc.with_timezone(tz).date_naive()
}


// ---------------------------------------------------------------------------
// format_date
// ---------------------------------------------------------------------------

fn format_date(date: chrono::NaiveDate, lang: &str) -> String {
    let month_key = format!("month.{}", date.month());
    let month_name = get_string(&month_key, lang);
    format!("{} {} {:02}", date.year(), month_name, date.day())
}

// ---------------------------------------------------------------------------
// build_antardasas
// ---------------------------------------------------------------------------

/// Build Antardasa entries for one Mahadasa using sequential Sun-travel boundaries.
///
/// `mahadasa_full_degrees` — the lord's full years × 360° (used to derive each
///   antardasa's proportional degree slice).
/// `first_antar_start_jd` — the JD at which this Mahadasa begins (= birth JD for
///   the first partial Mahadasa).
/// `elapsed_degrees` — degrees of the Mahadasa already elapsed before birth; 0.0
///   for full Mahadasas.
///
/// Antardasa boundaries are computed sequentially: each antardasa's end JD is
/// found by advancing the Sun exactly `antar_degrees` from that antardasa's start.
fn build_antardasas(
    mahadasa_lord_idx:     usize,
    mahadasa_full_degrees: f64,
    first_antar_start_jd:  f64,
    elapsed_degrees:       f64,
    tz:                    &chrono_tz::Tz,
    lang:                  &str,
) -> Result<Vec<AntardasaEntry>, String> {
    let mut entries = Vec::new();
    let mut antar_cumulative = 0.0_f64;
    // Tracks the JD at the start of the next antardasa (updated each iteration).
    let mut current_jd = first_antar_start_jd;

    for i in 0..9_usize {
        let antar_lord_idx = (mahadasa_lord_idx + i) % 9;
        let antar_lord     = &DASA_SEQUENCE[antar_lord_idx];
        let antar_degrees  = mahadasa_full_degrees * (antar_lord.years as f64) / 120.0;

        let antar_start_in_maha = antar_cumulative;
        let antar_end_in_maha   = antar_cumulative + antar_degrees;
        antar_cumulative = antar_end_in_maha;

        // Skip Antardasas that ended entirely before birth.
        if antar_end_in_maha <= elapsed_degrees {
            continue;
        }

        let (start_date, end_jd) = if antar_start_in_maha < elapsed_degrees {
            // This Antardasa straddles birth: display start is birth; advance only
            // the remaining degrees (from birth to the antardasa's logical end).
            let remaining = antar_end_in_maha - elapsed_degrees;
            let end = advance_sun_degrees(first_antar_start_jd, remaining)?;
            (jd_to_local_date(first_antar_start_jd, tz), end)
        } else {
            // Full antardasa: advance from the previous antardasa's end JD.
            let end = advance_sun_degrees(current_jd, antar_degrees)?;
            (jd_to_local_date(current_jd, tz), end)
        };

        let end_date = jd_to_local_date(end_jd, tz);
        current_jd = end_jd;

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

    Ok(entries)
}

// ---------------------------------------------------------------------------
// build_periods
// ---------------------------------------------------------------------------

/// Build the complete 9-Mahadasa sequence starting from `starting_lord_idx`.
///
/// Period boundaries are found by `advance_sun_degrees`: each Mahadasa ends when
/// the Sun has traveled `lord.years × 360°` (tropical) from that Mahadasa's start.
fn build_periods(
    starting_lord_idx: usize,
    balance_fraction:  f64,
    birth_jd:          f64,
    tz:                &chrono_tz::Tz,
    lang:              &str,
) -> Result<Vec<MahadasaEntry>, String> {
    let mut periods = Vec::new();
    let mut current_jd = birth_jd;

    for i in 0..9_usize {
        let lord_idx     = (starting_lord_idx + i) % 9;
        let lord         = &DASA_SEQUENCE[lord_idx];
        let full_degrees = lord.years as f64 * 360.0;

        // First Mahadasa is partial (balance_fraction of full period).
        let maha_degrees    = if i == 0 { balance_fraction * full_degrees } else { full_degrees };
        // Degrees already elapsed in the first Mahadasa before birth; 0 for the rest.
        let elapsed_degrees = if i == 0 { (1.0 - balance_fraction) * full_degrees } else { 0.0 };

        let maha_start_jd = current_jd;
        let maha_end_jd   = advance_sun_degrees(maha_start_jd, maha_degrees)?;

        let start_date = jd_to_local_date(maha_start_jd, tz);
        let end_date   = jd_to_local_date(maha_end_jd, tz);

        let planet_name = get_string(&format!("planet.dasa.{}", lord.planet_key), lang);
        let dasa_maha   = get_string("dasa.maha", lang);

        let antardasas = build_antardasas(
            lord_idx,
            full_degrees,   // proportional Antardasas use the FULL Mahadasa degrees
            maha_start_jd,
            elapsed_degrees,
            tz,
            lang,
        )?;

        let is_current = antardasas.iter().any(|a| a.is_current);

        periods.push(MahadasaEntry {
            lord:       lord.planet_key.to_string(),
            label:      format!("{planet_name} {dasa_maha}"),
            start_date: format_date(start_date, lang),
            end_date:   format_date(end_date, lang),
            is_current,
            antardasas,
        });

        current_jd = maha_end_jd;
    }

    Ok(periods)
}

// ---------------------------------------------------------------------------
// execute (entry point + error handling)
// ---------------------------------------------------------------------------

pub fn execute(request: &str) -> String {
    let req: VimsottariRequest = match serde_json::from_str(request) {
        Ok(r)  => r,
        Err(e) => {
            let msg = e.to_string().replace('"', "\\\"");
            return format!("{{\"error\":\"JSON parse error: {msg}\"}}");
        }
    };

    let all_cities = match city_data::cities() {
        Ok(c)  => c,
        Err(e) => {
            let msg = e.replace('"', "\\\"");
            return format!("{{\"error\":\"cities.csv error: {msg}\"}}");
        }
    };
    let city = match all_cities.iter().find(|c| c.city_id == req.city_id) {
        Some(c) => c,
        None    => return format!("{{\"error\":\"city not found: cityId={}\"}}",  req.city_id),
    };

    let (lat, lng) = city_data::decode_city_id(req.city_id);

    let jd = match utils::local_to_jd(&req.local_time, &city.timezone) {
        Ok(jd)  => jd,
        Err(e)  => {
            let msg = e.replace('"', "\\\"");
            return format!("{{\"error\":\"invalid localTime '{}': {msg}\"}}", req.local_time);
        }
    };

    let tz: chrono_tz::Tz = match city.timezone.parse() {
        Ok(tz)  => tz,
        Err(_)  => return format!("{{\"error\":\"invalid timezone: '{}'\"}}", city.timezone),
    };

    let ephe = ephe_path();
    unsafe {
        swe_wrappers::swe_set_ephe_path(ephe.as_ptr() as *const c_char);
        swe_wrappers::swe_set_sid_mode(SE_SIDM_TRUE_CITRA, 0.0, 0.0);
    }

    let moon_lon = match swe_wrappers::calc_planet(jd, SE_MOON) {
        Ok(lon)  => lon,
        Err(e)   => {
            let msg = e.replace('"', "\\\"");
            return format!("{{\"error\":\"calc_planet(Moon) failed: {msg}\"}}");
        }
    };

    let (_, _, _, _, nakshatra_num, _) = utils::decompose_longitude(moon_lon);
    let starting_lord_idx = nakshatra_to_lord_index(nakshatra_num);
    let balance_fraction  = compute_balance_fraction(moon_lon, nakshatra_num);

    let periods = match build_periods(starting_lord_idx, balance_fraction, jd, &tz, &req.lang) {
        Ok(p)  => p,
        Err(e) => {
            let msg = e.replace('"', "\\\"");
            return format!("{{\"error\":\"period computation failed: {msg}\"}}");
        }
    };

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
