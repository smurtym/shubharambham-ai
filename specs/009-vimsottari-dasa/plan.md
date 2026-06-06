# Implementation Plan: Vimsottari Dasa

**Branch**: `009-vimsottari-dasa` | **Date**: 2026-03-21 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/009-vimsottari-dasa/spec.md`

## Summary

A new Rust engine (`engines/vimsottari.rs`) computes Vimsottari Mahadasa and Antardasa periods based on the Moon's sidereal nakshatra at birth. The engine reuses the existing Swiss Ephemeris infrastructure (`swe_wrappers::calc_planet` for Moon longitude, `utils::decompose_longitude` for nakshatra identification, `utils::local_to_jd` for timezone-aware Julian Day conversion) without modifying any of those modules. Date arithmetic uses the `chrono` crate (already a dependency) to add fractional days, with rounding to whole days only at final formatting. Fourteen new localization keys (dasa labels + 12 month names) are added to both `en.rs` and `te.rs`. The bridge gains one new dispatch arm (`"vimsottari_dasa"`). No existing engine logic is modified.

## Technical Context

**Language/Version**: Rust 2021 (stable toolchain)
**Primary Dependencies**: `serde 1` + `serde_json 1` (serialization); `chrono 0.4` + `chrono-tz 0.9` (date arithmetic, timezone); Swiss Ephemeris via existing C FFI — all already present in `Cargo.toml`
**Storage**: N/A — pure computation
**Testing**: `cargo test` (Rust unit tests against reference birth chart SC-001/SC-002; contiguity assertion SC-003)
**Target Platform**: `wasm32-unknown-emscripten` (production) + native (`cargo test`)
**Project Type**: Library (WASM module) — additive extension of existing bridge
**Performance Goals**: Imperceptible — single Moon longitude lookup + in-memory date arithmetic; no latency target
**Constraints**: No new crate dependencies; zero modifications to existing engines; WASM binary size increase limited to new Rust code (~2–3 KB compiled)
**Scale/Scope**: One engine, 9 Mahadasas × 9 Antardasas each, 14 locale strings; single operation added to bridge

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Rust-First Computation | **PASS** | All dasa calculation, date arithmetic, and localization in Rust. JS/TS layer only invokes bridge and renders returned JSON. |
| II. React + TypeScript Presentation Layer | **N/A** | This feature is a computation engine only. No UI changes in scope. |
| III. Contract-Driven WASM API | **PASS** | Contract `contracts/vimsottari-dasa-api-v1.md` written before implementation tasks start. Same bridge signature; new operation name only. |
| IV. Mobile-First Design | **N/A** | No UI component in this feature. |
| V. Correctness & Accuracy | **PASS** | Reference chart tests (SC-001, SC-002) verify exact date match ±0 days. Solar year = 365.2425 days per FR-006. Fractional precision maintained per FR-009. |
| VI. Localization in Rust | **PASS** | 14 new keys (`dasa.maha`, `dasa.antar`, `month.1`–`month.12`) compiled into `en.rs` and `te.rs`. No runtime locale fetch. |

**Post-Phase-1 re-check**: All principles remain PASS. No new crate dependencies required — `chrono`, `chrono-tz`, `serde`, `serde_json` already in `Cargo.toml`. No complexity exceptions needed.

## Project Structure

### Documentation (this feature)

```text
specs/009-vimsottari-dasa/
├── plan.md              ← this file
├── research.md          ← Phase 0 output
├── data-model.md        ← Phase 1 output
├── quickstart.md        ← Phase 1 output
├── contracts/
│   └── vimsottari-dasa-api-v1.md  ← Phase 1 output
└── tasks.md             ← Phase 2 output (/speckit.tasks — not created by plan)
```

### Source Code (repository root)

```text
astro-wasm/
├── Cargo.toml             UNCHANGED — all deps already present
├── build.rs               UNCHANGED
└── src/
    ├── lib.rs             UNCHANGED (engines module already pub)
    ├── bridge.rs          MODIFY — add "vimsottari_dasa" dispatch arm (additive only)
    ├── utils.rs           UNCHANGED — reuse local_to_jd(), decompose_longitude()
    ├── swe_wrappers.rs    UNCHANGED — reuse calc_planet(), SE_MOON, swe_set_sid_mode()
    ├── localization.rs    UNCHANGED — reuse get_string(key, lang)
    ├── locales/
    │   ├── en.rs          MODIFY — add 14 keys: dasa.maha, dasa.antar, month.1–month.12
    │   └── te.rs          MODIFY — same 14 keys in Telugu
    └── engines/
        ├── mod.rs         MODIFY — add "pub mod vimsottari;"
        ├── horoscope.rs   UNCHANGED — no modifications
        ├── stub.rs        UNCHANGED
        └── vimsottari.rs  NEW — dasa engine: request parsing, Moon lookup, period computation, response assembly
```

**Structure Decision**: Single-project additive extension. The vimsottari engine is a new file (`engines/vimsottari.rs`) that calls into existing shared infrastructure (`utils`, `swe_wrappers`, `localization`, `data`) without modifying any of those modules. The bridge gains one new `match` arm. This pattern is identical to how `horoscope.rs` was added in feature 006 — proven safe and non-breaking.

## Layer Responsibilities

| Layer | File | Responsibility | Modified? |
|---|---|---|---|
| **Common utility** | `utils.rs` | `local_to_jd()`, `decompose_longitude()` | NO — reuse only |
| **SE wrappers** | `swe_wrappers.rs` | `calc_planet(jd, SE_MOON)`, `swe_set_sid_mode()` | NO — reuse only |
| **City data** | `data/` | City lookup by `city_id` | NO — reuse only |
| **Localization** | `locales/en.rs`, `locales/te.rs` | Add 14 new keys per locale | YES — additive |
| **Localization** | `localization.rs` | `get_string(key, lang)` dispatch | NO — reuse only |
| **Vimsottari engine** | `engines/vimsottari.rs` | Input parsing; Moon lookup; nakshatra→lord; balance fraction; period computation; date formatting; response assembly | YES — NEW |
| **Engine registry** | `engines/mod.rs` | Register `pub mod vimsottari` | YES — additive |
| **Bridge** | `bridge.rs` | Route `"vimsottari_dasa"` → engine; serialise result; return byte count | YES — additive |

## Isolation Strategy

The following guarantees ensure no existing logic is broken:

1. **No modification to existing engine files**: `horoscope.rs` and `stub.rs` are UNCHANGED.
2. **No modification to shared utilities**: `utils.rs`, `swe_wrappers.rs`, `localization.rs` are UNCHANGED.
3. **Additive bridge routing**: The new `"vimsottari_dasa"` arm is added to the `match` statement alongside existing arms. The `_` (unknown op) arm remains last.
4. **Additive locale keys**: New keys are appended to the existing `STRINGS` arrays. The compile-time length assertion is updated from 71 to 85 — existing 71 keys untouched.
5. **Independent testing**: Vimsottari unit tests are self-contained within `vimsottari.rs` and do not alter test fixtures for other engines.

## Complexity Tracking

No constitution violations. No complexity exceptions needed.
