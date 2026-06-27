# Feature Specification: Cities Engine Refactor

**Feature Branch**: `011-cities-engine-refactor`

**Created**: 2026-06-27

**Status**: Draft

**Input**: User description: "create a new feature and its branch to have city as an engine instead of a different special function. No functional changes, it is just to make the code clean"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Developer Adds a New Engine Without Touching City Logic (Priority: P1)

A developer maintaining the codebase extends the bridge with a new computation engine. Because city lookup is now a proper engine module — not a special private function in `bridge.rs` — the developer does not need to understand or touch any city-specific code. The bridge match arm for the new engine follows exactly the same single-line pattern as `list_cities`, `horoscope_positions`, and `vimsottari_dasa`.

**Why this priority**: The core motivation of the refactor. Structural uniformity in `bridge.rs` reduces the cognitive overhead of adding future engines and eliminates a one-off dispatch pattern that would otherwise diverge from the convention.

**Independent Test**: Read `bridge.rs` and confirm that all three operation dispatch arms — `list_cities`, `horoscope_positions`, `vimsottari_dasa` — are structurally identical single-line calls to their respective engine's `execute` function. No extra struct definitions or private dispatch helpers exist in `bridge.rs`.

**Acceptance Scenarios**:

1. **Given** the refactored codebase, **When** a developer reads `bridge.rs`, **Then** each operation arm is a single line delegating to `engines::<name>::execute(input)` with no per-operation structs or helper functions defined in `bridge.rs`.
2. **Given** the refactored codebase, **When** a developer adds a new engine, **Then** the pattern to follow is immediately clear from the three existing uniform examples.

---

### User Story 2 - All Existing Tests Pass Without Modification (Priority: P1)

The existing bridge integration tests for `list_cities` — covering English city list, Telugu filtering, `canonicalName` ASCII invariant, and missing `lang` field — continue to pass without any test code changes. No test expectations change because no behaviour changes.

**Why this priority**: A refactor with a failing test is a regression. Passing all existing tests without modification is the primary correctness gate.

**Independent Test**: Run `cargo test` in `astro-wasm/` and confirm all `test_bridge_list_cities_*` tests pass.

**Acceptance Scenarios**:

1. **Given** the refactored code, **When** `cargo test` is executed, **Then** all pre-existing tests pass with no changes to test code.
2. **Given** a `list_cities` request with a missing `lang` field, **When** the bridge processes it, **Then** it returns `-2` and an error JSON, identical to the pre-refactor behaviour.
3. **Given** an `operation` mismatch between `op_ptr` and the JSON body, **When** the bridge processes it, **Then** it returns `-2`, identical to the pre-refactor behaviour.

---

### Edge Cases

- What happens to the operation-mismatch check currently inside `dispatch_list_cities`? It must move into `engines/cities.rs` so the engine self-validates its input.
- What happens if `engines/cities.rs` is not registered in `engines/mod.rs`? The code will not compile — this is a hard gate caught at build time.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A new `engines/cities.rs` module MUST be created containing a public `execute(input: &str) -> String` function that encapsulates all city-request deserialisation, operation-mismatch validation, and JSON response serialisation.
- **FR-002**: The `CitiesRequest` struct MUST be defined in `engines/cities.rs` and MUST NOT exist in `bridge.rs`.
- **FR-003**: The private `dispatch_list_cities` function in `bridge.rs` MUST be removed entirely.
- **FR-004**: The `bridge` match arm for `"list_cities"` MUST delegate to `engines::cities::execute(input)` in the same single-line pattern used by `horoscope_positions` and `vimsottari_dasa`.
- **FR-005**: `engines/cities.rs` MUST be declared as a public sub-module in `engines/mod.rs`.
- **FR-006**: No changes to any JSON message shapes, operation names, error codes, or return values are permitted — the WASM API contract is unchanged.
- **FR-007**: No changes to the `data` module (`data::list_cities`, `data::cities::CITIES`, or any related data layer) are permitted.
- **FR-008**: No changes to the TypeScript glue layer (`astro-glue.ts`) are permitted.
- **FR-009**: All existing `bridge` unit tests MUST pass without modification to test code.

### Key Entities

- **`engines/cities.rs`**: New engine module. Contains `CitiesRequest` struct and `execute` function. Mirrors the structure of `engines/horoscope.rs` and `engines/vimsottari.rs`.
- **`bridge.rs`**: Simplified. The `list_cities` arm becomes a single-line delegate. `CitiesRequest` and `dispatch_list_cities` are removed.
- **`engines/mod.rs`**: Updated to declare `pub mod cities`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test` in `astro-wasm/` exits with zero failures after the refactor, with zero modifications to existing test code.
- **SC-002**: The `bridge.rs` match block contains no struct definitions and no private dispatch helper functions — only the `write_json`, `write_error`, and `bridge` declarations plus the `#[cfg(test)]` block.
- **SC-003**: The `list_cities` match arm in `bridge.rs` is a single expression, structurally indistinguishable from the `horoscope_positions` and `vimsottari_dasa` arms.
- **SC-004**: `engines/cities.rs` exists and contains the `execute` function and `CitiesRequest` struct previously split across `bridge.rs`.

## Assumptions

- The `data::list_cities` function signature and behaviour are stable and are not changed as part of this feature.
- The `engines/horoscope.rs` and `engines/vimsottari.rs` modules serve as the structural reference for the new `engines/cities.rs` module.
- The WASM API contract for `list_cities` (operation name, input schema, output schema, error codes) does not change — no contract versioning is required.
- The refactor is internal to the Rust crate; no changes are needed in the Emscripten build configuration, Vite config, or deployment pipeline.
