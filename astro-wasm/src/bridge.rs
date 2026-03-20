use crate::engines;
use crate::data;
use std::ffi::CStr;
use std::os::raw::c_char;

// ---------------------------------------------------------------------------
// Request shapes
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct CitiesRequest {
    operation: String,
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
        "stub_op"       => {
            match engines::stub::handle_stub("") {
                Ok(r)  => { let j = format!("{{\"value\":\"{}\"}}", r.value); write_json(&j, output_ptr, output_max_len) }
                Err(e) => write_error(&e, -3, output_ptr, output_max_len),
            }
        },
        "list_cities"          => dispatch_list_cities(op, input, output_ptr, output_max_len),
        "horoscope_positions"  => {
            let json = engines::horoscope::execute(input);
            write_json(&json, output_ptr, output_max_len)
        },
        _ => write_error(&format!("unknown operation: {op}"), -1, output_ptr, output_max_len),
    }
}

fn dispatch_list_cities(
    op:             &str,
    input:          &str,
    output_ptr:     *mut c_char,
    output_max_len: i32,
) -> i32 {
    let req: CitiesRequest = match serde_json::from_str(input) {
        Ok(r)  => r,
        Err(e) => return write_error(&format!("JSON parse error: {e}"), -2, output_ptr, output_max_len),
    };
    if req.operation != op {
        return write_error(
            &format!("operation mismatch: op_ptr={op} body={}", req.operation),
            -2,
            output_ptr,
            output_max_len,
        );
    }
    let json = data::list_cities(&req.lang);
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
        let mut output = vec![0_u8; 262144]; // 256 KB — enough for the full city list
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
        let (ret, _) = call_bridge(b"list_cities\0", b"not-json\0");
        assert_eq!(ret, -2);
    }

    #[test]
    fn test_bridge_operation_mismatch_returns_minus2() {
        let input = b"{\"operation\":\"other_op\",\"lang\":\"en\"}\0";
        let (ret, _) = call_bridge(b"list_cities\0", input);
        assert_eq!(ret, -2);
    }

    // -----------------------------------------------------------------------
    // T012 — US1 bridge integration test
    // -----------------------------------------------------------------------

    #[test]
    fn test_bridge_list_cities_en() {
        let input = b"{\"operation\":\"list_cities\",\"lang\":\"en\"}\0";
        let (ret, output) = call_bridge(b"list_cities\0", input);
        assert!(ret > 0, "bridge returned error code: {ret}");
        let json: serde_json::Value = serde_json::from_slice(&output[..ret as usize])
            .expect("output is valid JSON");
        let cities = json["cities"].as_array().expect("cities array present");
        // Derive expected count from CITIES so adding en-translated cities
        // requires no test edits.
        let expected = crate::data::cities::CITIES.iter()
            .filter(|r| r.translations.iter().any(|(l, _)| *l == "en"))
            .count();
        assert_eq!(cities.len(), expected,
            "en response has {} cities but {} have en translations", cities.len(), expected);
    }

    // -----------------------------------------------------------------------
    // T014 — US2 bridge integration test
    // -----------------------------------------------------------------------

    #[test]
    fn test_bridge_list_cities_te_filters() {
        let input = b"{\"operation\":\"list_cities\",\"lang\":\"te\"}\0";
        let (ret, output) = call_bridge(b"list_cities\0", input);
        assert!(ret > 0, "bridge returned error code: {ret}");
        let json: serde_json::Value = serde_json::from_slice(&output[..ret as usize])
            .expect("output is valid JSON");
        let cities = json["cities"].as_array().expect("cities array present");
        let canonical_names: std::collections::HashSet<&str> = cities.iter()
            .map(|c| c["canonicalName"].as_str().unwrap())
            .collect();
        // Derive presence/absence expectations from CITIES rather than hardcoding
        // city names — adding a te translation to any city keeps the test correct.
        for rec in crate::data::cities::CITIES {
            let has_te = rec.translations.iter().any(|(l, _)| *l == "te");
            if has_te {
                assert!(canonical_names.contains(rec.canonical_name),
                    "'{}' has a te translation but is absent from the te bridge response",
                    rec.canonical_name);
            } else {
                assert!(!canonical_names.contains(rec.canonical_name),
                    "'{}' has no te translation but appears in the te bridge response",
                    rec.canonical_name);
            }
        }
    }

    // -----------------------------------------------------------------------
    // T016 — US3 bridge integration test
    // -----------------------------------------------------------------------

    #[test]
    fn test_bridge_canonical_name_invariant() {
        // Property: canonicalName is always ASCII (English) for every response
        // language — checking is_ascii() scales to any number of cities.
        for lang in &[b"{\"operation\":\"list_cities\",\"lang\":\"te\"}\0".as_ref(),
                      b"{\"operation\":\"list_cities\",\"lang\":\"en\"}\0".as_ref()] {
            let (ret, output) = call_bridge(b"list_cities\0", lang);
            assert!(ret > 0);
            let json: serde_json::Value = serde_json::from_slice(&output[..ret as usize]).unwrap();
            for city in json["cities"].as_array().unwrap() {
                let name = city["canonicalName"].as_str().unwrap();
                assert!(name.is_ascii(),
                    "canonicalName '{name}' must be ASCII (English) regardless of request language");
            }
        }
    }

    // -----------------------------------------------------------------------
    // T021 — FR-012: missing `lang` field returns -2 + error JSON
    // -----------------------------------------------------------------------

    #[test]
    fn test_bridge_list_cities_missing_lang() {
        // JSON is valid but `lang` field is absent — CitiesRequest deserialisation fails.
        let input = b"{\"operation\":\"list_cities\"}\0";
        let (ret, output) = call_bridge(b"list_cities\0", input);
        assert_eq!(ret, -2, "expected -2 for missing lang field");
        let trimmed: Vec<u8> = output.iter().copied().take_while(|&b| b != 0).collect();
        let json: serde_json::Value = serde_json::from_slice(&trimmed)
            .expect("error response is valid JSON");
        assert!(json["error"].as_str().is_some(), "expected 'error' key in response");
        assert!(json["cities"].is_null(), "must not have 'cities' key on error");
    }

}
