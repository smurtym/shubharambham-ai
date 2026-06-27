# Tasks: Cities Engine Refactor

**Input**: Design documents from `specs/011-cities-engine-refactor/`
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓

## Phase 1: Core Refactor

**Purpose**: Move city logic into `engines/cities.rs` and clean up `bridge.rs`

- [x] T001 [US1] Create `astro-wasm/src/engines/cities.rs` with private `CitiesRequest` struct and `pub fn execute(input: &str) -> Result<String, String>`
- [x] T002 [US1] Add `pub mod cities;` to `astro-wasm/src/engines/mod.rs`
- [x] T003 [US1] Update `astro-wasm/src/bridge.rs`: remove `CitiesRequest` struct and `dispatch_list_cities` fn; update `list_cities` arm to call `engines::cities::execute(input)`

---

## Phase 2: Validation

- [x] T004 [US2] Run `cargo test` in `astro-wasm/` and confirm all tests pass (including all `test_bridge_list_cities_*`)

---

## Dependencies & Execution Order

- T001 → T002 (engine must exist before registering module)
- T001 → T003 (engine must exist before bridge calls it)
- T002, T003 can run in parallel (different files)
- T004 depends on T001 + T002 + T003
