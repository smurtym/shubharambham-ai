# Implementation Plan: Cities Engine Refactor

**Branch**: `011-cities-engine-refactor` | **Date**: 2026-06-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/011-cities-engine-refactor/spec.md`

## Summary

Move city-listing logic out of `bridge.rs` and into a new `engines/cities.rs` module, making `list_cities` consistent with the engine-per-operation pattern used by `horoscope_positions` and `vimsottari_dasa`. The only files that change are `bridge.rs`, `engines/mod.rs`, and the new `engines/cities.rs`. No data layer, TypeScript layer, WASM contract, or test code changes.

## Technical Context

**Language/Version**: Rust (stable toolchain, dual-target: native `cargo test` + `wasm32-unknown-emscripten` release)

**Primary Dependencies**: `serde`, `serde_json` (same crates already used by `bridge.rs` and all existing engines)

**Storage**: N/A

**Testing**: `cargo test` in `astro-wasm/` — existing bridge tests are the acceptance gate with no modifications

**Target Platform**: WASM (Emscripten) + native (cargo test); no Emscripten toolchain needed for this change

**Project Type**: Internal Rust library refactor

**Performance Goals**: Unchanged — zero runtime overhead introduced

**Constraints**: All existing `bridge` unit tests MUST pass unchanged; WASM API contract `city-data-v1` MUST NOT change

**Scale/Scope**: ~30 lines moved/restructured; 1 new file (`engines/cities.rs`); 2 files modified (`bridge.rs`, `engines/mod.rs`)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Rust-First Computation | PASS | All changes are within the Rust crate; no computation logic moves to JS |
| II. React + TypeScript Presentation Layer | N/A | No frontend changes |
| III. Contract-Driven WASM API | PASS | `city-data-v1` contract is unchanged; no new contract version required because the external API surface (operation name, request/response shapes, error codes) does not change |
| IV. Mobile-First Design | N/A | No UI changes |
| V. Correctness & Accuracy | PASS | No calculation logic changes; all existing reference-value tests pass |
| VI. Localization in Rust | PASS | No localization changes |

**Post-design re-check**: All six principles remain satisfied after the Phase 1 design (see data-model.md and quickstart.md).

## Key Architectural Decision

`engines/cities::execute` returns `Result<String, String>` rather than `String` (the pattern used by horoscope/vimsottari). This distinction is intentional and required:

- `horoscope` and `vimsottari` return all outcomes (success and error) as a JSON String, and the bridge writes them with `write_json` → always returning a positive byte count.
- Existing tests `test_bridge_malformed_json_returns_minus2` and `test_bridge_operation_mismatch_returns_minus2` specifically assert that the bridge returns `-2` for `list_cities` parse/mismatch errors. This behavior must be preserved (FR-009 / SC-001).
- The `Result<String, String>` return type lets the bridge arm use `write_json` for success and `write_error(..., -2, ...)` for failures, preserving the `-2` return code without the 20-line `dispatch_list_cities` function.

The bridge arm for `list_cities` is therefore a 3-line match expression — a dramatic improvement over the current approach even if not strictly identical to the horoscope/vimsottari single-line pattern.

## Project Structure

### Documentation (this feature)

```text
specs/011-cities-engine-refactor/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 output (/speckit-tasks command)
```

### Source Code Changes

```text
astro-wasm/src/
├── engines/
│   ├── mod.rs           ← MODIFY: add `pub mod cities;`
│   ├── cities.rs        ← CREATE: CitiesRequest struct + execute() -> Result<String, String>
│   ├── horoscope.rs     ← unchanged
│   ├── stub.rs          ← unchanged
│   └── vimsottari.rs    ← unchanged
└── bridge.rs            ← MODIFY: remove CitiesRequest + dispatch_list_cities; update list_cities arm
```

**Structure Decision**: Single Rust crate. The new `engines/cities.rs` follows the existing engine module convention (`pub fn execute(input: &str) -> ...` as the sole public API). The `Result` return type is chosen over `String` to preserve the existing `-2` error code semantics asserted in bridge tests.

## Complexity Tracking

No Constitution Check violations.
