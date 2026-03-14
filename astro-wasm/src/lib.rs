use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

// ---------------------------------------------------------------------------
// Swiss Ephemeris FFI
// ---------------------------------------------------------------------------

const SE_SUN: i32 = 0;
const SEFLG_SWIEPH: i32 = 2; // use Swiss Ephemeris file-based data

extern "C" {
    fn swe_set_ephe_path(path: *const c_char);
    fn swe_calc_ut(
        tjd_ut: f64,
        ipl: c_int,
        iflag: c_int,
        xx: *mut f64,
        serr: *mut c_char,
    ) -> c_int;
}

// ---------------------------------------------------------------------------
// Bridge — sole public WASM export (see contracts/wasm-api-v1.md)
// ---------------------------------------------------------------------------

/// Route `op` to the appropriate handler.
///
/// Returns:
/// - `> 0` and `<= output_max_len`: bytes written to `output_ptr`
/// - `> output_max_len`:            buffer too small; value is required size
/// - `-1`: unknown operation
/// - `-2`: JSON parse error
/// - `-3`: calculation error
#[no_mangle]
pub extern "C" fn bridge(
    op_ptr: *const c_char,
    input_ptr: *const c_char,
    output_ptr: *mut c_char,
    output_max_len: i32,
) -> i32 {
    // Safety: JS owns and manages the lifetime of all three buffers.
    // Rust must not free, reallocate, or store these pointers beyond this call.
    let op = match unsafe { CStr::from_ptr(op_ptr) }.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    let input = match unsafe { CStr::from_ptr(input_ptr) }.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    match op {
        "sun_longitude" => handle_sun_longitude(input, output_ptr, output_max_len),
        _ => -1,
    }
}

// ---------------------------------------------------------------------------
// sun_longitude handler
// ---------------------------------------------------------------------------

fn handle_sun_longitude(input: &str, output_ptr: *mut c_char, output_max_len: i32) -> i32 {
    // Parse JSON input: { "tjd": <f64> }
    let parsed: serde_json::Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(_) => return -2,
    };
    let tjd_ut = match parsed.get("tjd").and_then(|v| v.as_f64()) {
        Some(t) => t,
        None => return -2,
    };

    // Call Swiss Ephemeris via FFI.
    // Safety: constant C string literal; pointer is valid for the call duration.
    let ephe_path = b"/ephe\0";
    unsafe { swe_set_ephe_path(ephe_path.as_ptr() as *const c_char) };

    let mut xx = [0.0_f64; 6];
    let mut serr = [0_u8; 256];
    let ret = unsafe {
        swe_calc_ut(
            tjd_ut,
            SE_SUN,
            SEFLG_SWIEPH,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };

    if ret < 0 {
        return -3;
    }

    // Serialize output: { "longitude": <f64> }
    let longitude = xx[0]; // ecliptic longitude, degrees
    let json_out = format!("{{\"longitude\":{longitude}}}");
    let bytes = json_out.as_bytes();
    let needed = bytes.len() as i32;

    if needed > output_max_len {
        return needed; // signal Option B retry
    }

    // Safety: output_ptr points to a buffer of at least output_max_len bytes
    // allocated by JS. We only write `needed` bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), output_ptr as *mut u8, bytes.len());
    }
    needed
}
