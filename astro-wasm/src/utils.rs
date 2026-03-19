// ---------------------------------------------------------------------------
// utils — pure Rust helpers, no Swiss Ephemeris dependency
// ---------------------------------------------------------------------------

/// Convert an ISO 8601 UTC datetime string to Julian Day Number (UT).
///
/// Implements the Meeus proleptic Gregorian calendar arithmetic described in
/// "Astronomical Algorithms" (2nd ed.) chapter 7.  The algorithm is exact for
/// any date on the proleptic Gregorian calendar.
///
/// # Reference value
/// `"2000-01-01T12:00:00Z"` → exactly `2451545.0` (J2000.0 epoch).
///
/// # Errors
/// Returns `Err` if the string is malformed, fields are out of range, or the
/// timezone suffix is not `Z` (UTC-only; timezone offsets are out of scope).
pub fn iso_to_jd(dt: &str) -> Result<f64, String> {
    // Expected format: YYYY-MM-DDTHH:MM:SSZ  (length 20, exactly)
    let dt = dt.trim();
    if dt.len() < 20 {
        return Err(format!("iso_to_jd: too short: '{dt}'"));
    }
    if !dt.ends_with('Z') {
        return Err(format!("iso_to_jd: must end with Z (UTC only): '{dt}'"));
    }
    let dt = &dt[..dt.len() - 1]; // strip trailing Z

    let parse_u32 = |s: &str, field: &str| -> Result<u32, String> {
        s.parse::<u32>()
            .map_err(|_| format!("iso_to_jd: invalid {field}: '{s}'"))
    };
    let parse_f64 = |s: &str, field: &str| -> Result<f64, String> {
        s.parse::<f64>()
            .map_err(|_| format!("iso_to_jd: invalid {field}: '{s}'"))
    };

    // Split on 'T'
    let (date_part, time_part) = dt
        .split_once('T')
        .ok_or_else(|| format!("iso_to_jd: missing T separator: '{dt}'"))?;

    // Parse date: YYYY-MM-DD
    let date_parts: Vec<&str> = date_part.splitn(3, '-').collect();
    if date_parts.len() != 3 {
        return Err(format!("iso_to_jd: invalid date '{date_part}'"));
    }
    let year  = parse_u32(date_parts[0], "year")? as i32;
    let month = parse_u32(date_parts[1], "month")?;
    let day   = parse_u32(date_parts[2], "day")?;

    if !(1..=12).contains(&month) {
        return Err(format!("iso_to_jd: month out of range: {month}"));
    }
    if !(1..=31).contains(&day) {
        return Err(format!("iso_to_jd: day out of range: {day}"));
    }

    // Parse time: HH:MM:SS
    let time_parts: Vec<&str> = time_part.splitn(3, ':').collect();
    if time_parts.len() != 3 {
        return Err(format!("iso_to_jd: invalid time '{time_part}'"));
    }
    let hour   = parse_f64(time_parts[0], "hour")?;
    let minute = parse_f64(time_parts[1], "minute")?;
    let second = parse_f64(time_parts[2], "second")?;

    // Fractional day  
    let day_frac = day as f64 + (hour + minute / 60.0 + second / 3600.0) / 24.0;

    // Meeus algorithm (Chapter 7): adjust year/month for Jan/Feb
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };

    let a = (y as f64 / 100.0).floor() as i64;
    let b = 2 - a + (a / 4) as i64;

    let jd = (365.25 * (y as f64 + 4716.0)).floor()
        + (30.600_1 * (m as f64 + 1.0)).floor()
        + day_frac
        + b as f64
        - 1524.5;

    Ok(jd)
}

/// Wrap an ecliptic longitude in degrees to the range [0.0, 360.0).
pub fn normalize_degrees(deg: f64) -> f64 {
    ((deg % 360.0) + 360.0) % 360.0
}

// ---------------------------------------------------------------------------
// T005 — Local time + IANA timezone → Julian Day Number (UT)
// ---------------------------------------------------------------------------

/// Convert a naive local datetime string and an IANA timezone name to a
/// Julian Day Number (UT).
///
/// # Format
/// `local_time` must be `"YYYY-MM-DDTHH:MM:SS"` (no timezone suffix).
/// `iana_tz` must be a valid IANA timezone string, e.g. `"Asia/Kolkata"`.
///
/// # DST handling
/// - Ambiguous times (clocks fall back, time exists twice): uses the earlier
///   UTC offset (pre-transition), per spec FR-002.
/// - Gap times (clocks spring forward, time doesn't exist): advances 1 hour
///   to the post-gap UTC time, per spec FR-002.
///
/// # Errors
/// Returns `Err` if `local_time` cannot be parsed or `iana_tz` is unknown.
pub fn local_to_jd(local_time: &str, iana_tz: &str) -> Result<f64, String> {
    use chrono::NaiveDateTime;
    use chrono::TimeZone as _;
    use chrono_tz::Tz;

    let naive = NaiveDateTime::parse_from_str(local_time, "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| format!("invalid localTime: {local_time}"))?;

    let tz: Tz = iana_tz
        .parse()
        .map_err(|_| format!("invalid timezone: {iana_tz}"))?;

    let dt = match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(early, _) => early, // pre-transition (FR-002)
        chrono::LocalResult::None => {
            // DST spring-forward gap: advance 1 hour to post-gap time (FR-002)
            #[allow(deprecated)]
            let advanced = naive + chrono::Duration::hours(1);
            match tz.from_local_datetime(&advanced) {
                chrono::LocalResult::Single(dt) => dt,
                chrono::LocalResult::Ambiguous(dt, _) => dt,
                chrono::LocalResult::None => {
                    return Err(format!("localTime falls in unresolvable DST gap: {local_time}"))
                }
            }
        }
    };

    // JD from Unix timestamp: 1970-01-01T00:00:00Z = JD 2440587.5
    let unix_secs = dt.timestamp() as f64;
    Ok(unix_secs / 86400.0 + 2440587.5)
}

// ---------------------------------------------------------------------------
// T006 — Decompose sidereal longitude into zodiac/nakshatra components
// ---------------------------------------------------------------------------

/// Decompose a sidereal longitude (degrees) into its Vedic components.
///
/// Returns `(zodiac_num, deg_in_sign, minutes, seconds, nakshatra_num, pada)`
/// where all values are truncated (not rounded) per spec FR-007.
///
/// - `zodiac_num`: 1–12 (Aries = 1)
/// - `deg_in_sign`: 0–29 (degrees within the sign)
/// - `minutes`: 0–59 (arc-minutes within the degree)
/// - `seconds`: 0–59 (arc-seconds within the arc-minute)
/// - `nakshatra_num`: 1–27 (Ashwini = 1)
/// - `pada`: 1–4 (quarter within the nakshatra)
pub fn decompose_longitude(lon: f64) -> (u8, u8, u8, u8, u8, u8) {
    let lon = normalize_degrees(lon);

    // Zodiac sign (each sign = 30°)
    let sign_idx    = (lon / 30.0).floor() as u8;
    let zodiac_num  = sign_idx + 1;

    // Degrees, arc-minutes, arc-seconds within the sign (all truncated)
    let deg_in_sign_f = lon % 30.0;
    let deg_in_sign   = deg_in_sign_f.floor() as u8;
    let total_min_f   = (deg_in_sign_f - deg_in_sign as f64) * 60.0;
    let minutes       = total_min_f.floor() as u8;
    let seconds       = ((total_min_f - minutes as f64) * 60.0).floor() as u8;

    // Nakshatra (each nakshatra = 360°/27 = 13°20')
    let nak_size    = 360.0 / 27.0;
    let nak_idx     = (lon / nak_size).floor() as u8;
    let nakshatra_num = nak_idx + 1;

    // Pada (each pada = 360°/108 = 3°20')
    let pada_size   = 360.0 / 108.0;
    let lon_in_nak  = lon % nak_size;
    let pada        = (lon_in_nak / pada_size).floor() as u8 + 1;

    (zodiac_num, deg_in_sign, minutes, seconds, nakshatra_num, pada)
}

// ---------------------------------------------------------------------------
// T007 — Navamsa (D9) zodiac sign
// ---------------------------------------------------------------------------

/// Compute the Navamsa (D9) zodiac sign number (1–12, Aries = 1) for a given
/// sidereal longitude.
///
/// Divides 0–360° into 108 equal parts of 3°20' (360/108) each.
/// The navamsa sign cycles through 1–12 repeatedly:
///   navamsa = (floor(longitude / (360/108)) % 12) + 1
pub fn navamsa_sign(longitude: f64) -> u8 {
    let lon       = normalize_degrees(longitude);
    let part_size = 360.0 / 108.0;              // 3.333... degrees per part
    let part      = (lon / part_size).floor() as u32;
    ((part % 12) + 1) as u8
}

// Old Chara/Sthira/Ubhaya implementation (kept for reference):
// pub fn navamsa_sign(longitude: f64) -> u8 {
//     let lon       = normalize_degrees(longitude);
//     let sign_idx  = (lon / 30.0).floor() as u8;           // 0-based (0 = Aries)
//     let nav_idx   = ((lon % 30.0) / (30.0 / 9.0)).floor() as u8; // 0–8
//
//     let start: u8 = match sign_idx % 3 {
//         0 => 0,  // Chara  → Aries
//         1 => 9,  // Sthira → Capricorn
//         _ => 3,  // Ubhaya → Cancer
//     };
//
//     ((start + nav_idx) % 12) + 1
// }

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_j2000_epoch() {
        let jd = iso_to_jd("2000-01-01T12:00:00Z").expect("should parse J2000.0");
        assert!(
            (jd - 2451545.0).abs() < 1e-9,
            "J2000.0 should be exactly 2451545.0, got {jd}"
        );
    }

    #[test]
    fn test_malformed_returns_err() {
        assert!(iso_to_jd("not-a-date").is_err());
        assert!(iso_to_jd("2000-01-01T12:00:00").is_err()); // missing Z
        assert!(iso_to_jd("2000-01-01").is_err());           // no time part
    }

    #[test]
    fn test_normalize_degrees_wraps() {
        assert!((normalize_degrees(360.0) - 0.0).abs() < 1e-12);
        assert!((normalize_degrees(400.0) - 40.0).abs() < 1e-12);
        assert!((normalize_degrees(-10.0) - 350.0).abs() < 1e-12);
        assert!((normalize_degrees(0.0) - 0.0).abs() < 1e-12);
        assert!((normalize_degrees(359.9) - 359.9).abs() < 1e-12);
    }

    // T005 tests
    #[test]
    fn test_local_to_jd_utc_j2000() {
        // 2000-01-01 12:00:00 UTC = J2000.0 = JD 2451545.0
        let jd = local_to_jd("2000-01-01T12:00:00", "UTC").expect("UTC should parse");
        assert!((jd - 2451545.0).abs() < 1e-5, "J2000.0 via UTC, got {jd}");
    }

    #[test]
    fn test_local_to_jd_kolkata_j2000() {
        // IST = UTC+5:30 → 2000-01-01 17:30:00 IST = 2000-01-01 12:00:00 UTC
        let jd = local_to_jd("2000-01-01T17:30:00", "Asia/Kolkata").expect("Kolkata OK");
        assert!((jd - 2451545.0).abs() < 1e-5, "J2000.0 via IST, got {jd}");
    }

    #[test]
    fn test_local_to_jd_malformed() {
        assert!(local_to_jd("not-a-date", "UTC").is_err());
        assert!(local_to_jd("2000-01-01T12:00:00", "Invalid/Zone").is_err());
    }

    // T006 tests
    #[test]
    fn test_decompose_aries_0() {
        let (zn, d, m, s, nak, pada) = decompose_longitude(0.0);
        assert_eq!(zn, 1);    // Aries
        assert_eq!(d, 0);
        assert_eq!(m, 0);
        assert_eq!(s, 0);
        assert_eq!(nak, 1);   // Ashwini
        assert_eq!(pada, 1);
    }

    #[test]
    fn test_decompose_capricorn_start() {
        // 270° = Capricorn 0°00'00"
        let (zn, d, m, s, _nak, _) = decompose_longitude(270.0);
        assert_eq!(zn, 10); // Capricorn
        assert_eq!(d, 0);
        assert_eq!(m, 0);
        assert_eq!(s, 0);
    }

    #[test]
    fn test_decompose_truncation() {
        // 15.999° — minutes/seconds should be truncated not rounded
        let (_, d, m, s, _, _) = decompose_longitude(15.999);
        assert_eq!(d, 15);
        assert_eq!(m, 59);   // 0.999 × 60 = 59.94 → 59
        assert_eq!(s, 56);   // 0.94 × 60 = 56.4 → 56
    }

    // T007 tests — 108-part linear navamsa: (floor(lon / (360/108)) % 12) + 1
    #[test]
    fn test_navamsa_start() {
        // 0°: part 0 → (0 % 12) + 1 = 1 (Aries)
        assert_eq!(navamsa_sign(0.0), 1);
    }

    #[test]
    fn test_navamsa_fourth_part() {
        // 10°: floor(10 / 3.333) = 3 → (3 % 12) + 1 = 4
        assert_eq!(navamsa_sign(10.0), 4);
    }

    #[test]
    fn test_navamsa_cycles_at_40_degrees() {
        // 40°: part 12 → (12 % 12) + 1 = 1 (wraps back to Aries)
        assert_eq!(navamsa_sign(40.0), 1);
    }

    #[test]
    fn test_navamsa_60_degrees() {
        // 60°: floor(60 / 3.333) = 18 → (18 % 12) + 1 = 7
        assert_eq!(navamsa_sign(60.0), 7);
    }

    #[test]
    fn test_navamsa_wraps_at_360() {
        // 359.99°: floor(359.99 / 3.333) = 107 → (107 % 12) + 1 = 12
        assert_eq!(navamsa_sign(359.99), 12);
        // 360° normalises to 0° → part 0 → 1
        assert_eq!(navamsa_sign(360.0), 1);
    }

    // Old Chara/Sthira/Ubhaya test cases (kept for reference):
    // #[test]
    // fn test_navamsa_aries_first() {
    //     // Aries 0°: Chara → start Aries (1), navamsa 0 → Aries = 1
    //     assert_eq!(navamsa_sign(0.0), 1);
    // }
    // #[test]
    // fn test_navamsa_taurus_first() {
    //     // Taurus 0° = 30°: Sthira → start Capricorn (10, idx=9), navamsa 0 = 10
    //     assert_eq!(navamsa_sign(30.0), 10);
    // }
    // #[test]
    // fn test_navamsa_gemini_first() {
    //     // Gemini 0° = 60°: Ubhaya → start Cancer (4, idx=3), navamsa 0 = 4
    //     assert_eq!(navamsa_sign(60.0), 4);
    // }
    // #[test]
    // fn test_navamsa_capricorn_first() {
    //     // Capricorn 0° = 270°: Chara → start Aries (1), navamsa 0 = 1
    //     assert_eq!(navamsa_sign(270.0), 1);
    // }
    // #[test]
    // fn test_navamsa_second_navamsa_taurus() {
    //     // Taurus 3°21' = 30° + 3.35° — navamsa idx = 1 → Aquarius (10+1=11)
    //     assert_eq!(navamsa_sign(33.35), 11);
    // }
}
