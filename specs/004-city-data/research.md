# Research: City Data

**Phase**: 0  
**Branch**: `004-city-data`  
**Date**: 2026-03-15

---

## R-001: Data integrity validation strategy

**Question**: How should we validate `cityId` uniqueness, non-empty `canonicalName`, and valid IANA timezones at build/test time without external crates?

**Decision**:
- `canonicalName` non-empty → `const fn` evaluated at compile time (`const _: () = validate_cities();`). `&str::is_empty()` works in stable Rust 2021 const contexts; catch is at compile time before any binary is produced.
- `cityId` uniqueness → `#[test]` function using a `HashSet`. Compile-time loops for uniqueness produce cryptic error messages; a test gives actionable output.
- IANA timezone validity → `#[test]` function asserting each city's timezone is in a `const VALID_TIMEZONES: &[&str]` list co-located with the data. List starts with `["Asia/Kolkata", "America/New_York"]` and grows as cities are added.

**Rationale**: Split by error visibility. Structural invariants (empty canonical name) are truly compile-time errors — catch them early. Data correctness checks (uniqueness, timezone strings) produce better developer experience as tests with descriptive failure messages.

**Alternative considered**: `build.rs` validation script — rejected because a test function achieves the same result with less tooling overhead, staying consistent with the constitution's preference for simple build tooling.

---

## R-002: JSON serialization approach for variable-length arrays

**Question**: Manual `format!()` strings (existing pattern) vs. `serde::Serialize` derive for the `{"cities": [...]}` response?

**Decision**: Derive `serde::Serialize` on lightweight response structs. Use `#[serde(skip)]` on `region1Order` and `region2Order` to explicitly exclude sort keys from the wire format.

**Rationale**:
- Manual `format!()` is viable for 2-field objects (existing pattern) but becomes error-prone with 7 fields per city and a variable-length array — requires manual comma-joining, string escaping, and is easy to accidentally include a sort key.
- `serde` and `serde_json` are already in `Cargo.toml`. No new dependency is introduced.
- `#[serde(skip)]` is compiler-enforced documentation: it makes the exclusion of sort keys explicit and auditable, not a silent omission.
- Binary size impact is negligible: serde derive generates specialized serialization for the exact struct shape.

**Alternative considered**: Manual `format!()` — rejected for 7-field structs due to escaping complexity and maintainability cost as city count grows.

---

## R-003: Sort strategy

**Question**: How to sort the filtered city list by `(region2Order, region1Order, cityName)` in the WASM target?

**Decision**: `Vec::sort_by_key()` with a tuple key `(region2_order, region1_order, city_name.as_str())`. `std` is available on `wasm32-unknown-emscripten` (Emscripten provides a full libc), so `std::vec::Vec` and stable sort are fully available.

**Rationale**: Straightforward stdlib usage. Stable sort preserves relative order of equal-key entries. The spec states relative order is unspecified for fully-equal keys, so stability is a bonus, not a requirement.

**Alternative considered**: Sorting the static `CITIES` array at source by hand — rejected because it couples data layout to language-specific sort order. Different languages have different `region2Order` values, so in-source ordering cannot satisfy all languages simultaneously.

---

## R-004: Data store representation in Rust

**Question**: Struct layout for `CityRecord` and `TranslationEntry` in a hardcoded static array.

**Decision**:
- `CityRecord`: `city_id: u32`, `canonical_name: &'static str`, `timezone: &'static str`, `translations: &'static [(&'static str, TranslationEntry)]` (slice of (lang, entry) pairs — zero heap allocation, compatible with `const` / `static` context).
- `TranslationEntry`: `city_name: &'static str`, `region1: &'static str`, `region2: &'static str`, `region1_order: u16`, `region2_order: u16`.
- A separate `CityResponse` struct (with `#[derive(serde::Serialize)]`) is used for serialization only; it holds `String` / `u32` fields and is constructed per-request from `CityRecord` + `TranslationEntry`.

**Rationale**: Static string slices (`&'static str`) keep the city data entirely in the binary's read-only data segment with zero heap allocation. Integer sort keys fit in `u16` (0-65535 ordering range is ample). The separate response struct cleanly separates the internal data model from the wire format.

**Alternative considered**: `HashMap<&'static str, TranslationEntry>` for translations — rejected because `HashMap` requires heap allocation and `std` hashing, adding complexity for a small fixed dataset. A linear scan over a short `&[(&'static str, TranslationEntry)]` slice is simpler and adequate for hundreds of cities.

---

## R-005: Web layer integration

**Question**: Where does the JS `listCities(lang)` function live and what pattern does it follow?

**Decision**: Add `listCities(lang)` to `web/data.js`, mirroring the existing `getSunLongitude(isoDatetime, lang)` function. The function constructs `{"operation":"list_cities","lang":lang}`, calls `bridge("list_cities", ...)` via `astro-glue.js`, and returns the parsed response. The JS caller checks `response.error` before reading `response.cities`.

**Rationale**: Consistent with the established web data layer pattern. No changes to `astro-glue.js` (buffer management) or `components.js` needed.

---

## Summary: All unknowns resolved

| # | Unknown | Resolution |
|---|---------|------------|
| R-001 | Build-time validation approach | const fn for empty fields; #[test] for uniqueness + timezone |
| R-002 | JSON serialization for array response | serde::Serialize derive + #[serde(skip)] for sort keys |
| R-003 | Sort strategy on WASM target | Vec::sort_by_key() with tuple; std available on emscripten |
| R-004 | Rust data store struct layout | &'static str slices; separate response struct for serde |
| R-005 | Web layer integration point | listCities() in web/data.js, mirrors getSunLongitude pattern |
