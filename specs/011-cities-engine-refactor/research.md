# Research: Cities Engine Refactor

**Feature**: `011-cities-engine-refactor` | **Date**: 2026-06-27

## Phase 0 Research Summary

This feature is a pure internal structural refactor. All decisions are grounded in the existing codebase patterns — no external research or ambiguous technology choices needed.

---

## Decision 1: Engine return type — `String` vs `Result<String, String>`

**Decision**: `engines::cities::execute` returns `Result<String, String>`

**Rationale**: The existing tests `test_bridge_malformed_json_returns_minus2` and `test_bridge_operation_mismatch_returns_minus2` assert that the bridge returns `-2` for `list_cities` parse/validation failures. The other engines (`horoscope`, `vimsottari`) return `String` in all cases, so the bridge always calls `write_json` and gets a positive byte count — even on errors. Adopting `String` for cities would change the return code from `-2` to a positive value and break those tests. `Result<String, String>` lets `Ok` map to `write_json` and `Err` map to `write_error(..., -2, ...)`, preserving the asserted behavior with no test modifications.

**Alternatives considered**:
- `String` (match horoscope/vimsottari exactly): breaks two existing tests that assert `-2`; rejected per FR-009
- `(String, Option<i32>)` tuple: heavier API surface than needed; rejected in favour of idiomatic `Result`
- Keep `dispatch_list_cities` in `bridge.rs`: defeats the purpose of the refactor; rejected

---

## Decision 2: Struct visibility — `CitiesRequest` pub vs private

**Decision**: `CitiesRequest` is a private struct inside `engines/cities.rs`

**Rationale**: The struct is an implementation detail of the engine's input parsing. It has no use outside `engines/cities.rs`. Keeping it private enforces the encapsulation goal and matches how `HoroscopeRequest` and `VimsottariRequest` are private within their engine modules.

**Alternatives considered**:
- `pub struct CitiesRequest`: unnecessary exposure of internal parsing type; rejected

---

## Decision 3: Operation validation — in engine or in bridge?

**Decision**: Operation mismatch validation (`req.operation != "list_cities"`) moves into `engines/cities::execute`

**Rationale**: The validation is input-specific to the `list_cities` operation and belongs alongside the parsing logic in the engine module. This removes all cities-specific logic from `bridge.rs`. The bridge only routes; engines validate.

**Alternatives considered**:
- Validate in bridge before calling execute: mixing routing and validation concerns; keeps per-operation logic in bridge; rejected

---

## Decision 4: WASM contract versioning

**Decision**: No new contract version; `city-data-v1` remains at 1.0.0

**Rationale**: The external API surface is unchanged — same operation name, same request schema, same response schema, same error codes, same return code semantics. Per Constitution Principle III, a contract version increment is only required for breaking or additive API changes. A pure internal restructure does not qualify.

**References**: [specs/004-city-data/contracts/city-data-v1.md](../004-city-data/contracts/city-data-v1.md)
