# Feature Specification: Modular Architecture Layout

**Feature Branch**: `003-arch-layout`
**Created**: 2026-03-14
**Status**: Draft

## Clarifications

### Session 2026-03-14

- Q: What shape does the bridge return on error — extended SunResult with optional `error` field, separate error shape, or JS exception? → A: Option A — extend SunResult: `{ label?, longitude?, error? }`; on error both `label` and `longitude` are absent; JS caller checks `error` field first.
- Q: What does the page show while the WASM call is in progress — disabled submit button, "Calculating…" text, or nothing? → A: Option A — disable the submit button during the call; re-enable after result or error is displayed; Playwright waits for button re-enabled.
- Q: Should the bridge contract be versioned at runtime (version field in request), spec-time only (named contract, no runtime field), or not versioned? → A: Option B — name the contract "wasm-api-v2" in the spec; no runtime version field; old wasm-api-v1 contract is fully replaced with no backward compatibility.
- Q: What is the latency target for a single bridge call (submit → result displayed)? → A: 500 ms — starting point; will be tightened in a future feature.
- Q: Should ISO→JD conversion call `swe_utc_to_jd` via `swe_wrappers` or be implemented in pure Rust in `utils`? → A: Pure Rust in `utils` — no SWE dependency for this step; timezone support deferred to a future feature.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Developer can add a new engine without touching other layers (Priority: P1)

A developer can drop a new engine module into the `engines/` directory and register it in the bridge layer. No other layer needs to change. The architecture enforces this by clear module boundaries.

**Why this priority**: Without enforced boundaries, every new engine addition (horoscope, panchang, etc.) will create uncontrolled coupling. This layout is the foundation all future features build on.

**Independent Test**: The existing `sun_longitude` operation is refactored to live in `engines/sun/`. Adding a second stub engine (`engines/stub/`) and registering it in the bridge requires zero changes to `swe_wrappers`, `utils`, or `localization`. `cargo test` passes.

**Acceptance Scenarios**:

1. **Given** the new module layout is in place, **When** a developer adds a stub engine module and registers one operation in the bridge, **Then** `cargo build` and `cargo test` pass with no modifications to any other layer.
2. **Given** the existing `sun_longitude` operation, **When** the bridge dispatches it, **Then** the result is identical to the current behaviour — the refactor is transparent to JS callers.

---

### User Story 2 — User submits a date/time and language and receives a localized sun position (Priority: P2)

A developer or tester opens the index page, enters a date/time in ISO format and selects a language (English or Telugu), submits the form, and sees the Sun's ecliptic longitude formatted with a localized label. This is the end-to-end validation that all five layers connect correctly.

**Why this priority**: This is the only user-visible test in this feature. It exercises every layer in sequence: web input → bridge → utils (ISO→JD) → engine (SWE call) → localization (formatted label) → web output.

**Independent Test**: Playwright opens the page, enters `2000-01-01T12:00:00Z` and selects `te` (Telugu), submits, and asserts the result element contains the Telugu label for Sun (`సూర్యుడు`) alongside a numeric longitude.

**Acceptance Scenarios**:

1. **Given** the page is loaded, **When** a user enters an ISO datetime and selects `en` and submits, **Then** the submit button becomes disabled, and once re-enabled the result element displays the English label "Sun" alongside the numeric longitude value.
2. **Given** the page is loaded, **When** a user enters the same ISO datetime and selects `te` and submits, **Then** once the button is re-enabled the result element displays the Telugu label ("సూర్యుడు") with the same numeric longitude.
3. **Given** `web/astro-glue.js` exists, **When** a new page imports it, **Then** it gets WASM initialisation and the `bridge()` helper without copy-pasting any boilerplate.

---

### Edge Cases

- If a registered bridge operation name collides with an existing one, the build or a test must catch it — not silent runtime override.
- If `web/astro-glue.js` fails to load (e.g., missing `<script>` tag), the error should be visible in the browser console immediately, not a silent blank page.
- If the ISO datetime string is malformed, the utility layer must return an error before any SWE call is made.
- If a requested locale key is missing in the localization layer, a clearly marked fallback string must be returned — not an empty string or a panic.

## Requirements *(mandatory)*

### Functional Requirements

**Request/Response Contract (wasm-api-v2)**

The bridge accepts a JSON request and returns a JSON response. This is the `wasm-api-v2` contract — it fully replaces `wasm-api-v1` with no backward compatibility. For this feature the only operation is `sun_longitude`.

*Request* (JS → WASM):
```
{
  "operation": "sun_longitude",
  "datetime": "<ISO 8601 string, e.g. 2000-01-01T12:00:00Z>",
  "lang": "en" | "te"
}
```

*Response* (WASM → JS) — success:
```
{
  "label": "<localized planet name>",
  "longitude": <decimal degrees, e.g. 280.46>
}
```

*Response* (WASM → JS) — error:
```
{
  "error": "<human-readable error message string>"
}
```

On error, `label` and `longitude` are absent. The JS caller MUST check for the `error` field before reading `label` or `longitude`. No other fields are required in this feature.

**Rust Layer**

- **FR-001**: The Rust crate MUST be reorganised into five modules: `swe_wrappers`, `utils`, `engines` (sub-modules per engine), `localization`, and `bridge`. The `localization` module MUST use separate per-locale data files under `src/locales/` (e.g., `en.rs`, `te.rs`) so that string data is fully separated from lookup logic and can be edited by translators without touching logic code.
- **FR-002**: `swe_wrappers` MUST be the sole location for `extern "C"` Swiss Ephemeris declarations. No other module may call Swiss Ephemeris FFI directly.
- **FR-003**: The `sun_longitude` handler MUST be moved into `engines/sun/` as the reference engine. It MUST accept a Julian Day number and a locale identifier, call the SWE wrapper via `swe_wrappers`, and return a `SunResult` struct.
- **FR-004**: `utils` MUST provide: (a) ISO 8601 UTC string → Julian Day number conversion implemented in pure Rust (standard calendar arithmetic, no Swiss Ephemeris dependency), and (b) degree normalisation (wrap to 0–360). Both functions MUST have unit tests with reference values.
- **FR-005**: `bridge` MUST parse the incoming JSON request, call `utils` to convert the ISO datetime to a Julian Day, dispatch to the engine, and return the serialised JSON response. Adding a new operation MUST require a change only in `bridge`.
- **FR-006**: `localization` MUST store all locale strings in separate per-locale Rust data files (`src/locales/en.rs`, `src/locales/te.rs`) as a simple `const STRINGS: &[(&str, &str)]` array — key/value pairs only, no logic. The lookup logic in `localization.rs` imports these files. Adding a new language MUST require only a new data file and two one-line additions to the module declarations and dispatch — zero changes to any engine or bridge code.

**Web Layer**

- **FR-007**: WASM initialisation, buffer management, and the `bridge()` JS helper MUST move to `web/astro-glue.js`. `web/index.html` MUST source this file rather than containing the boilerplate inline.
- **FR-008**: `web/index.html` MUST contain an ISO datetime input field and a language selector (`en` / `te`). On submit, the submit button MUST be disabled until the bridge returns (success or error), then re-enabled. The localized label and longitude (or error message) MUST then be displayed in a dedicated result element.
- **FR-009**: A file `web/data.js` MUST define the `getSunLongitude(isoDatetime, lang)` function that constructs the bridge request and returns the parsed response. This is the web-side registry of WASM features.
- **FR-010**: A file `web/components.js` MUST exist as a stub, ready to hold reusable UI components. It can be empty or contain a placeholder comment in this feature.
- **FR-011**: The Playwright E2E tests MUST be updated to submit an ISO datetime and language via the new form inputs and assert the localized label and a numeric longitude appear in the result.

### Key Entities

- **SunRequest**: `{ operation: "sun_longitude", datetime: string (ISO 8601), lang: "en" | "te" }` — the JSON payload sent from JS to the WASM bridge.
- **SunResult**: `{ label?: string, longitude?: number, error?: string }` — the JSON response returned from the WASM bridge to JS. Exactly one of (`label` + `longitude`) or `error` is present.
- **Engine module**: A Rust sub-module under `engines/` that accepts a Julian Day and locale, calls `swe_wrappers`, and returns a typed result struct. Pattern established by `engines/sun/`.
- **Bridge operation**: A string key (`"sun_longitude"`) mapped to an engine handler in the bridge dispatch table.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test` passes after the Rust refactor with the `sun_longitude` unit test producing the same longitude value as before (within 0.01°).
- **SC-002**: The Playwright E2E test submits `2000-01-01T12:00:00Z` with `lang=te` and the result element contains the Telugu label `సూర్యుడు` and a numeric longitude.
- **SC-003**: The same E2E test run with `lang=en` produces the English label `Sun` — confirming the localization layer branches correctly.
- **SC-004**: Adding a stub second engine and registering it requires fewer than 15 lines of new code and zero changes to existing modules.
- **SC-005**: `web/index.html` contains no WASM buffer management or FFI boilerplate — those live entirely in `web/astro-glue.js`.
- **SC-006**: The full round-trip from submit click to result displayed (button re-enabled) completes within 500 ms, measured by the Playwright test wall-clock timeout.

## Assumptions

- The Sun longitude operation is the only operation in scope; no new astronomical computations are added.
- The bridge implements the `wasm-api-v2` contract (SunRequest / SunResult as defined above). The `wasm-api-v1` contract is fully retired; no backward compatibility is provided. The JS `bridge()` helper function signature is unchanged — only the JSON payload it carries is updated.
- ISO datetime input is always UTC; the ISO→JD conversion is pure Rust with no Swiss Ephemeris dependency. Timezone offset handling is out of scope for this feature and deferred to a future feature.
- Ayanamsha, house systems, and full horoscope output are out of scope and belong to a future feature.
- Telugu script rendering relies on the user's browser/OS font support; no custom font is bundled.
