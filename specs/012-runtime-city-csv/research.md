# Research: Runtime City CSV

**Feature**: `012-runtime-city-csv` | **Date**: 2026-06-27

## Phase 0 Research Summary

All decisions are grounded in existing codebase patterns (ephemeris file loading, `OnceLock` idiom in Rust std) and the user's explicit goal of simplicity. No external research needed.

---

## Decision 1: Cache mechanism — `std::sync::OnceLock<Vec<CityRecord>>`

**Decision**: Use `std::sync::OnceLock<Vec<CityRecord>>` in `data/mod.rs` as a module-level lazy singleton.

**Rationale**: `OnceLock` is in `std` (no new dependency), stable since Rust 1.70, thread-safe (important for native `cargo test` which runs tests in parallel), and gives `&'static [CityRecord]` semantics identical to the current `CITIES` static slice. `get_or_init` either returns the cached value or calls the initializer exactly once.

**Alternatives considered**:
- `once_cell::Lazy`: excellent but adds a dependency; rejected since `OnceLock` is now in std
- `lazy_static!`: same reason; rejected
- `static mut` with unsafe: simpler in single-threaded WASM but unsafe and breaks native test parallelism; rejected
- Re-read on every call: no caching, repeated I/O on every request; rejected

---

## Decision 2: Error propagation from `data::cities()`

**Decision**: `pub fn cities() -> Result<&'static [CityRecord], String>`

**Rationale**: If `cities.csv` is missing or unreadable, the error must propagate to the bridge so the caller receives a structured error JSON rather than a panic. `Result<_, String>` is the error type already used throughout the engine layer. The bridge maps this to a `-3` return code (calculation/data error) and writes `{"error":"..."}`.

**Alternatives considered**:
- `panic!` on load failure: simpler code but kills the WASM instance; rejected (FR-008 requires graceful error)
- `Option<&'static [CityRecord]>`: loses the error message; rejected

---

## Decision 3: CSV file location — `ephe/cities.csv`

**Decision**: Move `data/cities.csv` → `ephe/cities.csv`. No `build.rs` changes needed for WASM.

**Rationale**: `build.rs` already emits `--preload-file {repo}/ephe/@/ephe/` which preloads the entire `ephe/` directory into the WASM virtual FS. Adding `cities.csv` to `ephe/` makes it automatically available at `/ephe/cities.csv` in WASM — exactly the path pattern used for ephemeris data (`/ephe/semo_18.se1`, etc.). For native tests, `../ephe/cities.csv` relative to `astro-wasm/` mirrors the existing `../ephe/` ephemeris path.

**Alternatives considered**:
- Keep in `data/` and add a separate preload arg: two preload directories, more complex; rejected
- Embed CSV as a `include_str!()` string: still a compile-time coupling (defeats the purpose); rejected

---

## Decision 4: Data structure — owned `String` fields

**Decision**: Change `CityRecord.canonical_name`, `.timezone`, and `TranslationEntry` fields from `&'static str` to `String`.

**Rationale**: Runtime-loaded strings cannot be `&'static str`. The existing calling code already calls `.to_owned()` on most fields before putting them into `CityResponse`, so the change is largely mechanical. `horoscope.rs` and `vimsottari.rs` access `city.timezone` as a `&str` reference — `String` implements `Deref<Target=str>` so `&city.timezone` works without change.

**Alternatives considered**:
- `Box<str>`: slightly smaller than `String` but identical for our use case; no benefit; rejected
- `Arc<str>`: overkill for single-owner static cache; rejected

---

## Decision 5: Compile-time `const fn` validation removal

**Decision**: Remove `const _: () = validate_canonical_names()` and `fn validate_canonical_names()`.

**Rationale**: These functions reference `cities::CITIES` which no longer exists. The runtime equivalent — skipping or rejecting rows with empty `canonical_name` during CSV loading — plus the existing `test_canonical_name_always_english` test provides equivalent coverage without compile-time evaluation.

---

## Decision 6: Malformed row behaviour — skip silently

**Decision**: Rows with wrong column count or unparseable numeric fields are skipped. Rows with empty string fields are accepted.

**Rationale**: User directive is "Make it simple." Panicking on a bad row at build time (current behaviour) made sense for a code-gen step. At runtime, a bad row should not take down the whole operation. The CSV is managed by developers, not untrusted input, so a skip-and-continue policy is safe and simple.

---

## Decision 7: Tests referencing `cities::CITIES`

**Decision**: Replace all `cities::CITIES` references in tests with `data::cities().expect("cities.csv must be readable in test environment")` (or equivalent).

**Rationale**: Tests depend on the same CSV used at runtime, which is the source-of-truth file (`ephe/cities.csv`). This makes the tests test the actual data path, not a separate static copy.
