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

    // Set the ephemeris path once before dispatching to any handler.
    // Safety: constant C string literal; pointer is valid for the call duration.
    unsafe { swe_set_ephe_path(b"/ephe\0".as_ptr() as *const c_char) };

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

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Point the Swiss Ephemeris at the local ephe/ directory for tests.
    /// Uses CARGO_MANIFEST_DIR so the path is correct regardless of cwd.
    fn setup() {
        let ephe_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe\0");
        unsafe { swe_set_ephe_path(ephe_path.as_ptr() as *const c_char) };
    }

    /// bridge() must return -1 for an unrecognised operation.
    #[test]
    fn test_bridge_unknown_op() {
        let op = b"unknown_op\0";
        let input = b"{}\0";
        let mut output = [0_u8; 256];
        let ret = bridge(
            op.as_ptr() as *const c_char,
            input.as_ptr() as *const c_char,
            output.as_mut_ptr() as *mut c_char,
            output.len() as i32,
        );
        assert_eq!(ret, -1);
    }

    /// Sun longitude at J2000.0 (TJD 2451545.0) should be ≈ 280.37°.
    #[test]
    fn test_sun_longitude_j2000() {
        setup();
        let input = b"{\"tjd\":2451545.0}\0";
        let mut output = [0_u8; 256];
        let ret = handle_sun_longitude(
            std::str::from_utf8(&input[..input.len() - 1]).unwrap(),
            output.as_mut_ptr() as *mut c_char,
            output.len() as i32,
        );
        assert!(ret > 0, "handle_sun_longitude returned error: {ret}");
        let written = &output[..ret as usize];
        let json: serde_json::Value =
            serde_json::from_slice(written).expect("output is valid JSON");
        let longitude = json["longitude"].as_f64().expect("longitude is f64");
        assert!(
            (longitude - 280.37).abs() < 0.5,
            "Sun longitude {longitude} not within 0.5° of 280.37°"
        );
    }
}
