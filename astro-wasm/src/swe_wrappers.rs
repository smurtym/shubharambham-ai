use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

// ---------------------------------------------------------------------------
// Swiss Ephemeris constants
// ---------------------------------------------------------------------------

pub const SE_SUN: i32 = 0;
pub const SEFLG_SWIEPH: i32 = 2; // use Swiss Ephemeris file-based data
pub const SEFLG_SPEED: i32 = 256; // return speed (tropical coordinates)

// ---------------------------------------------------------------------------
// Raw FFI declarations — sole location for extern "C" Swiss Ephemeris symbols
// ---------------------------------------------------------------------------

#[allow(dead_code)]
extern "C" {
    pub fn swe_set_ephe_path(path: *const c_char);
    pub fn swe_calc_ut(
        tjd_ut: f64,
        ipl: c_int,
        iflag: c_int,
        xx: *mut f64,
        serr: *mut c_char,
    ) -> c_int;
}

// ---------------------------------------------------------------------------
// Safe wrapper — called only by engines/*
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
