# Feature Specification: Runtime City CSV

**Feature Branch**: `012-runtime-city-csv`

**Created**: 2026-06-27

**Status**: Draft

**Input**: User description: "I want to remove clumsiness of build time city list generation etc. Just keep cities file in ephe directory and parse/provide during runtime needed. Make it simple"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Developer Adds a City Without Recompiling (Priority: P1)

A developer wants to add a new city (or a new language translation for an existing city) to the app. Today this requires editing `data/cities.csv`, triggering a full Rust recompile, and rebuilding the WASM binary. After this change, the developer edits `ephe/cities.csv` and redeploying the data file is sufficient — no Rust recompile required.

**Why this priority**: The core motivation of this feature. Decoupling city data from the compiled binary eliminates the recompile cycle for what is a pure data change.

**Independent Test**: Add a new test city row to `ephe/cities.csv`, run the app (or `cargo test`), and confirm the new city appears in the city list without any recompile step.

**Acceptance Scenarios**:

1. **Given** a new city row appended to `ephe/cities.csv`, **When** the application serves a `list_cities` request, **Then** the new city appears in the response with correct fields, without rebuilding the Rust crate.
2. **Given** a city row deleted from `ephe/cities.csv`, **When** the application serves a `list_cities` request, **Then** the city is absent from the response, without rebuilding the Rust crate.

---

### User Story 2 - Build Process Has No City Code Generation Step (Priority: P1)

A developer running the build notices the build process no longer generates Rust source files from city data. There is no `generate_cities()` step, no `cities_generated.rs` artifact, and `data/cities.rs` (formerly auto-generated) no longer exists. The build is simpler and faster.

**Why this priority**: Eliminates a build-time coupling between data and code that has no technical justification — city records are data, not code.

**Independent Test**: Inspect the `build.rs` file and confirm it contains no city CSV reading or Rust code generation. Confirm `cargo build` succeeds without a `cities_generated.rs` file being produced.

**Acceptance Scenarios**:

1. **Given** a clean build environment, **When** `cargo build` runs, **Then** no `cities_generated.rs` file is written to `$OUT_DIR`.
2. **Given** the refactored codebase, **When** a developer reads `build.rs`, **Then** it contains only Emscripten link configuration and native swisseph compilation — no city CSV parsing or code emission.

---

### User Story 3 - All App Features Continue to Work Identically (Priority: P1)

End users see no change: the city dropdown populates with the same cities, horoscope calculations and Vimsottari Dasa calculations still work correctly for all cities, and the Telugu/English filtering and sort order are unchanged.

**Why this priority**: This is a refactor. Zero regressions is a hard requirement.

**Independent Test**: Run `cargo test`; all existing tests pass without modification.

**Acceptance Scenarios**:

1. **Given** the refactored system, **When** a user requests the city list in English, **Then** all cities with English translations appear, sorted identically to before.
2. **Given** the refactored system, **When** a user requests the city list in Telugu, **Then** only cities with Telugu translations appear, sorted identically to before.
3. **Given** the refactored system, **When** a user submits a horoscope or Vimsottari Dasa calculation with a valid city ID, **Then** the correct timezone and coordinates are used and the result is identical to before.
4. **Given** the refactored system, **When** `ephe/cities.csv` is absent or unreadable, **Then** the city listing operation returns a clear error rather than silently returning an empty list or crashing.

---

### Edge Cases

- What happens if `ephe/cities.csv` has malformed rows (missing columns, bad city_id)? Malformed rows must be skipped or produce a clear error — the valid rows must still load.
- What if the same `city_id` appears in multiple rows for the same language? The first occurrence wins (or the behaviour is defined and documented).
- What if the CSV is valid but empty (no data rows)? The city list returns an empty array — same as requesting an unsupported language today.
- In native `cargo test`, the CSV must be readable at the expected relative path; tests that directly iterate `data::cities::CITIES` must be updated to use the runtime-loaded data instead.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: City data MUST be loaded from `ephe/cities.csv` at runtime rather than compiled into the binary as a static Rust array.
- **FR-002**: The CSV file MUST be co-located with the ephemeris data files in the `ephe/` directory so a single directory deploy covers both ephemeris and city data.
- **FR-003**: The `list_cities` operation MUST return the same fields, filtering behaviour, and sort order as before — no change to the WASM API response shape.
- **FR-004**: City lookup by `city_id` (used by the horoscope and Vimsottari Dasa operations) MUST continue to resolve correctly using the runtime-loaded data.
- **FR-005**: `build.rs` MUST NOT contain any city CSV parsing or Rust code generation logic after this change.
- **FR-006**: `astro-wasm/src/data/cities.rs` (the auto-generated file) MUST be removed; the generated content is replaced by runtime parsing.
- **FR-007**: The CSV source file MUST move from `data/cities.csv` to `ephe/cities.csv`; `data/cities.csv` no longer exists.
- **FR-008**: When `ephe/cities.csv` cannot be read or parsed, the city operations MUST return a structured error response — the application MUST NOT panic or crash.
- **FR-009**: All existing `cargo test` tests MUST pass. Tests that previously referenced the `data::cities::CITIES` static array MUST be updated to use the runtime-loaded city data.
- **FR-010**: The CSV file format (columns, comment syntax, encoding) MUST remain unchanged so existing city entries require no edits.

### Key Entities

- **`ephe/cities.csv`**: The single source of truth for city data; moves from `data/cities.csv`; preloaded into the WASM virtual FS alongside ephemeris files.
- **Runtime City Record**: Equivalent to the current `CityRecord` struct but with owned string fields (loaded at runtime rather than static references to compiled-in strings).
- **City Cache**: The parsed city collection, loaded once on first use and reused for all subsequent requests within a WASM execution context.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test` exits with zero failures after the refactor, with zero modifications to test logic (only test data access path updates permitted).
- **SC-002**: `build.rs` contains no reference to `cities.csv`, no CSV parsing, and no Rust code generation for city data — verified by code inspection.
- **SC-003**: `astro-wasm/src/data/cities.rs` does not exist in the repository after this change.
- **SC-004**: Adding a city row to `ephe/cities.csv` causes the new city to appear in `list_cities` responses without any Rust recompile step.
- **SC-005**: The end-to-end Playwright tests (if run) pass without changes — users see no difference in the city dropdown.

## Assumptions

- The `ephe/` directory is already preloaded into the WASM virtual FS as part of the Emscripten build; placing `cities.csv` there means no Emscripten configuration changes are needed beyond the file's presence in the directory.
- Native `cargo test` can read `../ephe/cities.csv` relative to the `astro-wasm/` directory — the same convention used for ephemeris data in tests.
- The CSV column format (city_id, canonical_name, timezone, lang, city_name, region1, region2, region1_order, region2_order) is stable and does not change as part of this feature.
- City data is small enough (~160 rows at present) that parsing the CSV on first request and caching it for the lifetime of the WASM execution context introduces no perceptible latency.
- The `data/cities.csv` source file is deleted after being moved to `ephe/cities.csv`; no redirect or symlink is kept.
