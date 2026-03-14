// Stub engine — demonstrates that a new engine can be registered in
// fewer than 15 lines with zero changes to swe_wrappers, utils, or
// localization (SC-004).

pub struct StubResult {
    pub value: &'static str,
}

pub fn handle_stub(_lang: &str) -> Result<StubResult, String> {
    Ok(StubResult { value: "stub_ok" })
}
