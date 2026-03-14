use crate::localization;
use crate::swe_wrappers;
use crate::utils;
use std::os::raw::c_char;

pub struct SunEngineResult {
    pub label: &'static str,
    pub longitude: f64,
}

/// Compute the Sun's tropical ecliptic longitude and return it with a
/// localized label.
///
/// # Parameters
/// - `jd`: Julian Day number (UT), pre-computed by `utils::iso_to_jd`
/// - `lang`: locale identifier (`"en"` or `"te"`; unknown values fall back to `"en"`)
///
/// # Errors
/// Returns `Err` if the Swiss Ephemeris calculation fails.
pub fn handle_sun_longitude(jd: f64, lang: &str) -> Result<SunEngineResult, String> {
    // Set the ephemeris path before every call to ensure it is always correct.
    unsafe { swe_wrappers::swe_set_ephe_path(b"/ephe\0".as_ptr() as *const c_char) };

    let raw_longitude = swe_wrappers::calc_sun_longitude(jd)?;
    let longitude = utils::normalize_degrees(raw_longitude);
    let label = localization::get_string("planet.sun", lang);

    Ok(SunEngineResult { label, longitude })
}
