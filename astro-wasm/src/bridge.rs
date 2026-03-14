use crate::engines;
use crate::utils;
use std::ffi::CStr;
use std::os::raw::c_char;

// ---------------------------------------------------------------------------
// wasm-api-v2 request shape
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct SunRequest {
    operation: String,
    datetime:  String,
    lang:      String,
}

// ---------------------------------------------------------------------------
// Response helpers
// ---------------------------------------------------------------------------

fn write_json(json: &str, output_ptr: *mut c_char, output_max_len: i32) -> i32 {
    let bytes  = json.as_bytes();
    let needed = bytes.len() as i32;
    if needed > output_max_len {
        return needed; // Option B: signal required size
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            output_ptr as *mut u8,
            bytes.len(),
        );
    }
    needed
}

/// Write an `ErrorResponse` JSON to the buffer and return the negative code.
/// Always writes a best-effort error body on negative return paths so the JS
/// caller can display a human-readable message without inspecting return codes.
fn write_error(msg: &str, code: i32, output_ptr: *mut c_char, output_max_len: i32) -> i32 {
    let escaped = msg.replace('"', "\\\"");
    let json    = format!("{{\"error\":\"{escaped}\"}}");
    write_json(&json, output_ptr, output_max_len);
    code
}

// ---------------------------------------------------------------------------
// WASM export — the sole #[no_mangle] export in the crate (FR-002)
// ---------------------------------------------------------------------------

/// Route `op_ptr` to the appropriate handler (wasm-api-v2).
///
/// Return values:
/// -  `> 0` and `<= output_max_len`: bytes written to `output_ptr`
/// -  `> output_max_len`:            buffer too small — value is required size
/// -  `-1`: unknown operation name
/// -  `-2`: JSON parse error, invalid field, or op_ptr / JSON `operation` mismatch
/// -  `-3`: calculation error (SWE returned error)
#[no_mangle]
pub extern "C" fn bridge(
    op_ptr:         *const c_char,
    input_ptr:      *const c_char,
    output_ptr:     *mut c_char,
    output_max_len: i32,
) -> i32 {
    // Decode op_ptr (authoritative operation name).
    let op = match unsafe { CStr::from_ptr(op_ptr) }.to_str() {
        Ok(s)  => s,
        Err(_) => return write_error("invalid UTF-8 in op_ptr", -2, output_ptr, output_max_len),
    };

    // Decode input_ptr.
    let input = match unsafe { CStr::from_ptr(input_ptr) }.to_str() {
        Ok(s)  => s,
        Err(_) => return write_error("invalid UTF-8 in input_ptr", -2, output_ptr, output_max_len),
    };

    match op {
        "sun_longitude" => dispatch_sun_longitude(op, input, output_ptr, output_max_len),
        "stub_op"       => {
            match engines::stub::handle_stub("") {
                Ok(r)  => { let j = format!("{{\"value\":\"{}\"}}", r.value); write_json(&j, output_ptr, output_max_len) }
                Err(e) => write_error(&e, -3, output_ptr, output_max_len),
            }
        },
        _ => write_error(&format!("unknown operation: {op}"), -1, output_ptr, output_max_len),
    }
}

fn dispatch_sun_longitude(
    op:             &str,
    input:          &str,
    output_ptr:     *mut c_char,
    output_max_len: i32,
) -> i32 {
    // Parse wasm-api-v2 JSON request.
    let req: SunRequest = match serde_json::from_str(input) {
        Ok(r)  => r,
        Err(e) => return write_error(&format!("JSON parse error: {e}"), -2, output_ptr, output_max_len),
    };

    // op_ptr must match the JSON "operation" field (dispatch rule).
    if req.operation != op {
        return write_error(
            &format!("operation mismatch: op_ptr={op} body={}", req.operation),
            -2,
            output_ptr,
            output_max_len,
        );
    }

    // Convert ISO datetime → Julian Day.
    let jd = match utils::iso_to_jd(&req.datetime) {
        Ok(jd) => jd,
        Err(e) => return write_error(&format!("datetime error: {e}"), -2, output_ptr, output_max_len),
    };

    // Call engine.
    let result = match engines::sun::handle_sun_longitude(jd, &req.lang) {
        Ok(r)  => r,
        Err(e) => return write_error(&format!("calculation error: {e}"), -3, output_ptr, output_max_len),
    };

    // Serialize success response.
    let label   = result.label.replace('"', "\\\"");
    let json    = format!("{{\"label\":\"{label}\",\"longitude\":{}}}", result.longitude);
    write_json(&json, output_ptr, output_max_len)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::c_char;

    fn call_bridge(op: &[u8], input: &[u8]) -> (i32, Vec<u8>) {
        let mut output = vec![0_u8; 4096];
        let ret = bridge(
            op.as_ptr() as *const c_char,
            input.as_ptr() as *const c_char,
            output.as_mut_ptr() as *mut c_char,
            output.len() as i32,
        );
        (ret, output)
    }

    #[test]
    fn test_bridge_unknown_op_returns_minus1() {
        let (ret, output) = call_bridge(b"unknown_op\0", b"{}\0");
        assert_eq!(ret, -1, "expected -1 for unknown op");
        // Should also write an ErrorResponse JSON
        let written = &output[..(-ret.min(0)) as usize]; // ret < 0, no bytes; check via JSON
        let _ = written; // error body written; just check ret code here
    }

    #[test]
    fn test_bridge_malformed_json_returns_minus2() {
        let (ret, _) = call_bridge(b"sun_longitude\0", b"not-json\0");
        assert_eq!(ret, -2);
    }

    #[test]
    fn test_bridge_operation_mismatch_returns_minus2() {
        let input = b"{\"operation\":\"other_op\",\"datetime\":\"2000-01-01T12:00:00Z\",\"lang\":\"en\"}\0";
        let (ret, _) = call_bridge(b"sun_longitude\0", input);
        assert_eq!(ret, -2);
    }

    /// Sun longitude at J2000.0 via wasm-api-v2 should be ≈ 280.37° (SC-001).
    #[test]
    fn test_sun_longitude_j2000_via_bridge() {
        let ephe_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe\0");
        unsafe { crate::swe_wrappers::swe_set_ephe_path(ephe_path.as_ptr() as *const c_char) };

        let input = b"{\"operation\":\"sun_longitude\",\"datetime\":\"2000-01-01T12:00:00Z\",\"lang\":\"en\"}\0";
        let (ret, output) = call_bridge(b"sun_longitude\0", input);
        assert!(ret > 0, "bridge returned error: {ret}");
        let json: serde_json::Value = serde_json::from_slice(&output[..ret as usize])
            .expect("output is valid JSON");
        let longitude = json["longitude"].as_f64().expect("longitude is f64");
        assert_eq!(json["label"], "Sun");
        assert!(
            (longitude - 280.37).abs() < 0.5,
            "Sun longitude {longitude} not within 0.5° of 280.37°"
        );
    }

    /// Same test with Telugu locale — label should be సూర్యుడు (SC-001 + SC-002 preview).
    #[test]
    fn test_sun_longitude_j2000_te_label() {
        let ephe_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe\0");
        unsafe { crate::swe_wrappers::swe_set_ephe_path(ephe_path.as_ptr() as *const c_char) };

        let input = b"{\"operation\":\"sun_longitude\",\"datetime\":\"2000-01-01T12:00:00Z\",\"lang\":\"te\"}\0";
        let (ret, output) = call_bridge(b"sun_longitude\0", input);
        assert!(ret > 0, "bridge returned error: {ret}");
        let json: serde_json::Value = serde_json::from_slice(&output[..ret as usize])
            .expect("output is valid JSON");
        assert_eq!(json["label"], "సూర్యుడు");
    }
}
