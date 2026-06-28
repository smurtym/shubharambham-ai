// Stub engine — demonstrates that a new engine can be registered in
// fewer than 15 lines with zero changes to swe_wrappers, utils, or
// localization (SC-004).

pub fn execute(_input: &str) -> String {
    r#"{"value":"stub_ok"}"#.to_string()
}
