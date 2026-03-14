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
}
