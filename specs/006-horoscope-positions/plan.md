# Implementation Plan: Horoscope Positions

**Branch**: `006-horoscope-positions` | **Date**: 2026-03-15 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `/specs/006-horoscope-positions/spec.md`

## Summary

A new Rust engine (`engines/horoscope.rs`) computes sidereal planetary positions for all ten Vedic celestial bodies using Swiss Ephemeris with the True Chitrapaksha Ayanamsa (`SE_SIDM_TRUE_CITRA`, mode 27). A single `calc_planet` wrapper handles the 9 traditional bodies; the Ascendant uses its own `calc_ascendant` wrapper via `swe_houses_ex`. Local-time-to-UTC conversion using `chrono-tz` is placed in the shared `utils` layer for reuse by future engines. All 71 locale strings (10 planet names + abbrevs, 12 sign names + abbrevs, 27 nakshatra names) are added to the existing `locales/en.rs` and `locales/te.rs` static tables. The bridge gains one new dispatch arm (`"horoscope_positions"`). Output is a raw JSON dump — no formatting required.

## Technical Context

**Language/Version**: Rust 2021 (stable toolchain)  
**Primary Dependencies**: `serde 1` + `serde_json 1` (already present); `chrono` + `chrono-tz` (new — IANA timezone conversion); Swiss Ephemeris via existing C FFI  
**Storage**: N/A — pure computation  
**Testing**: `cargo test` (Rust unit tests against known reference chart values; data-integrity tests)  
**Target Platform**: `wasm32-unknown-emscripten` (production) + native (cargo test)  
**Project Type**: Library (WASM module) — additive extension of existing bridge  
**Performance Goals**: Imperceptible — pure in-memory computation over fixed ephemeris data; no latency target  
**Constraints**: `chrono-tz` permitted (specified in spec); WASM binary size impact is the embedded IANA tz database (~500 KB uncompressed) — acceptable  
**Scale/Scope**: One engine, 10 bodies, 71 locale strings; single operation added to bridge

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Rust-First Computation | **PASS** | All calculation logic in Rust. JS layer calls bridge, receives ready-to-render JSON. |
| II. Lean Web Presentation | **PASS** | Output is raw JSON; JS renders into DOM without transformation. No JS frameworks. |
| III. Contract-Driven WASM API | **PASS** | Contract `contracts/horoscope-api-v1.md` written before implementation tasks start. |
| IV. Mobile-First Design | **N/A** | This feature is a computation engine; UI rendering is out of scope. |
| V. Correctness & Accuracy | **PASS** | Reference chart test required (SC-001 ±0.02°). SE mode 27 locked in FR-004. |
| VI. Localization in Rust | **PASS** | All 71 locale strings compiled into WASM via static `STRINGS` tables. No runtime fetch. |

**Post-Phase-1 re-check**: All principles remain PASS. No crate justification exception needed beyond `chrono-tz` (explicitly accepted in spec clarification, correctness requirement for DST-aware timezone conversion).

## Project Structure

### Documentation (this feature)

```text
specs/006-horoscope-positions/
├── plan.md              ← this file
├── research.md          ← Phase 0 output
├── data-model.md        ← Phase 1 output
├── quickstart.md        ← Phase 1 output
├── contracts/
│   └── horoscope-api-v1.md  ← Phase 1 output
└── tasks.md             ← Phase 2 output (/speckit.tasks — not created by plan)
```

### Source Code (repository root)

```text
astro-wasm/
├── Cargo.toml             MODIFY — add chrono + chrono-tz dependencies
├── build.rs               UNCHANGED
└── src/
    ├── lib.rs             UNCHANGED (engines module already pub)
    ├── bridge.rs          MODIFY — add horoscope_positions dispatch arm
    ├── utils.rs           MODIFY — add local_to_jd() (common layer, reusable by future engines)
    ├── swe_wrappers.rs    MODIFY — add calc_planet(), calc_ascendant(), SE body/ayanamsa constants
    ├── locales/
    │   ├── en.rs          MODIFY — add 71 keys: 10 planet names+abbrevs, 12 sign names+abbrevs, 27 nakshatra names
    │   └── te.rs          MODIFY — same 71 keys in Telugu script
    └── engines/
        ├── mod.rs         MODIFY — add pub mod horoscope
        └── horoscope.rs   NEW — orchestration engine: request parsing, body loop, response assembly

web/
└── data.js                MODIFY — add getHoroscopePositions(cityId, localTime, lang) function
```

**Structure Decision**: Single-project extension. City data, timezone resolution, and JD conversion are handled in the shared `utils` layer — not inside the horoscope engine — so future engines (e.g., dasha, transit) can reuse them without duplication. The existing `bridge.rs` pattern (parse JSON → dispatch → serialize response) is followed exactly. Locale strings stay in static Rust arrays (no build-time codegen) per user direction.

## Layer Responsibilities

| Layer | File | Responsibility |
|---|---|---|
| **Common utility** | `utils.rs` | `local_to_jd(local_time, iana_tz)` — tz conversion + JD; `iso_to_jd`; `normalize_degrees` |
| **SE wrappers** | `swe_wrappers.rs` | `calc_planet(jd, body)` for 9 bodies; `calc_ascendant(jd, lat, lon)` for Lagna; FFI constants |
| **Horoscope engine** | `engines/horoscope.rs` | Input parsing; city lookup; coordinate / timezone resolution; body loop; position maths; response assembly |
| **Localization** | `locales/en.rs`, `locales/te.rs` | Static key-value arrays for planet names+abbrevs, zodiac sign names+abbrevs, nakshatra names |
| **Bridge** | `bridge.rs` | Route `"horoscope_positions"` → engine; serialise result; return byte count |
| **JS** | `web/data.js` | `getHoroscopePositions(cityId, localTime, lang)` — mirrors `listCities` pattern; raw JSON to caller |


## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: [e.g., Python 3.11, Swift 5.9, Rust 1.75 or NEEDS CLARIFICATION]  
**Primary Dependencies**: [e.g., FastAPI, UIKit, LLVM or NEEDS CLARIFICATION]  
**Storage**: [if applicable, e.g., PostgreSQL, CoreData, files or N/A]  
**Testing**: [e.g., pytest, XCTest, cargo test or NEEDS CLARIFICATION]  
**Target Platform**: [e.g., Linux server, iOS 15+, WASM or NEEDS CLARIFICATION]
**Project Type**: [e.g., library/cli/web-service/mobile-app/compiler/desktop-app or NEEDS CLARIFICATION]  
**Performance Goals**: [domain-specific, e.g., 1000 req/s, 10k lines/sec, 60 fps or NEEDS CLARIFICATION]  
**Constraints**: [domain-specific, e.g., <200ms p95, <100MB memory, offline-capable or NEEDS CLARIFICATION]  
**Scale/Scope**: [domain-specific, e.g., 10k users, 1M LOC, 50 screens or NEEDS CLARIFICATION]

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

[Gates determined based on constitution file]

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
