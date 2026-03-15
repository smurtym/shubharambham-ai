use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

// ---------------------------------------------------------------------------
// Swiss Ephemeris constants
// ---------------------------------------------------------------------------

pub const SE_SUN:          i32 = 0;
pub const SE_MOON:         i32 = 1;
pub const SE_MERCURY:      i32 = 2;
pub const SE_VENUS:        i32 = 3;
pub const SE_MARS:         i32 = 4;
pub const SE_JUPITER:      i32 = 5;
pub const SE_SATURN:       i32 = 6;
pub const SE_TRUE_NODE:    i32 = 11;
pub const SEFLG_SWIEPH:    i32 = 2;     // use Swiss Ephemeris file-based data
pub const SEFLG_SPEED:     i32 = 256;   // return speed
pub const SEFLG_SIDEREAL:  i32 = 65536; // sidereal coordinates
pub const SE_SIDM_TRUE_CITRA: i32 = 27; // True Chitrapaksha ayanamsa

// ---------------------------------------------------------------------------
// Raw FFI declarations — sole location for extern "C" Swiss Ephemeris symbols
// ---------------------------------------------------------------------------

#[allow(dead_code)]
extern "C" {
    pub fn swe_set_ephe_path(path: *const c_char);
    pub fn swe_set_sid_mode(sid_mode: c_int, t0: f64, ayan_t0: f64);
    pub fn swe_calc_ut(
        tjd_ut: f64,
        ipl: c_int,
        iflag: c_int,
        xx: *mut f64,
        serr: *mut c_char,
    ) -> c_int;
    pub fn swe_houses_ex(
        tjd_ut: f64,
        iflag: c_int,
        geolat: f64,
        geolon: f64,
        hsys: c_int,
        cusps: *mut f64,
        ascmc: *mut f64,
    ) -> c_int;
}

// ---------------------------------------------------------------------------
// Safe wrappers — called only by engines/*
// ---------------------------------------------------------------------------

/// Compute the tropical ecliptic longitude of the Sun for a given Julian Day.
///
/// Uses `SEFLG_SPEED` flag (tropical coordinates; `SEFLG_SIDEREAL` is not set).
/// No ayanamsha is applied — see contracts/wasm-api-v2.md §Coordinate System.
pub fn calc_sun_longitude(jd: f64) -> Result<f64, String> {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_u8; 256];
    let ret = unsafe {
        swe_calc_ut(
            jd,
            SE_SUN,
            SEFLG_SWIEPH | SEFLG_SPEED,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        let msg = CStr::from_bytes_until_nul(&serr)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "swe_calc_ut error".to_string());
        return Err(msg);
    }
    Ok(xx[0]) // ecliptic longitude in decimal degrees
}

/// Compute the sidereal ecliptic longitude of a planet for a given Julian Day.
///
/// Uses `SEFLG_SIDEREAL | SEFLG_SPEED`. Caller MUST call `swe_set_sid_mode`
/// before this function to set the ayanamsa (e.g. SE_SIDM_TRUE_CITRA).
/// Returns the sidereal longitude in [0, 360).
pub fn calc_planet(jd: f64, body: i32) -> Result<f64, String> {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_u8; 256];
    let ret = unsafe {
        swe_calc_ut(
            jd,
            body as c_int,
            SEFLG_SIDEREAL | SEFLG_SPEED,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        let msg = CStr::from_bytes_until_nul(&serr)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "swe_calc_ut error".to_string());
        return Err(msg);
    }
    Ok(xx[0])
}

/// Compute the sidereal Ascendant degree using the Whole Sign house system.
///
/// Caller MUST call `swe_set_sid_mode(SE_SIDM_TRUE_CITRA, 0.0, 0.0)` before
/// this function. Returns `ascmc[0]` (sidereal ascendant degree).
pub fn calc_ascendant(jd: f64, lat: f64, lon: f64) -> Result<f64, String> {
    let mut cusps = [0.0_f64; 13];
    let mut ascmc = [0.0_f64; 10];
    let ret = unsafe {
        swe_houses_ex(
            jd,
            SEFLG_SIDEREAL,
            lat,
            lon,
            b'W' as c_int,
            cusps.as_mut_ptr(),
            ascmc.as_mut_ptr(),
        )
    };
    if ret < 0 {
        return Err(format!("swe_houses_ex returned error code {ret}"));
    }
    Ok(ascmc[0])
}
