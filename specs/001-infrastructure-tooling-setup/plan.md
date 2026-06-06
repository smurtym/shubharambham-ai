# Implementation Plan: Infrastructure and Tooling Setup

**Branch**: `001-infrastructure-tooling-setup` | **Date**: 2026-03-14 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `specs/001-infrastructure-tooling-setup/spec.md`

## Summary

Establish the full build pipeline for all future features: compile the Swiss Ephemeris C
library (`swisseph` as a Git submodule) using Emscripten 4.0.13, link it with a Rust 1.88
crate targeting `wasm32-unknown-emscripten` via `build.rs` + `emcc` linker, embed the two
required ephemeris data files (`semo_18.se1`, `sepl_18.se1`) into the WASM virtual
filesystem via `--preload-file`, and serve a minimal `index.html` that displays "Work in
Progress" and prints the Sun's ecliptic longitude for J2000 to the browser console via the
Emscripten `onRuntimeInitialized` callback. No end-user value is delivered; this is a
developer-facing verification milestone.

## Technical Context

**Language/Version**: Rust 1.88 (stable); C99 (swisseph sources)  
**Emscripten**: 4.0.13 — `emcc` used as both C compiler and Rust linker  
**Primary Dependencies**: `swisseph` C library (Git submodule, `vendor/swisseph/`); `serde_json` 1.x (`alloc` feature, no_std-compatible) for JSON parsing inside bridge  
**Storage**: N/A — ephemeris data files preloaded into WASM virtual FS at build time  
**Testing**: `cargo test` (native, for Rust unit tests); manual browser smoke test for WASM integration  
**Target Platform**: WASM (`wasm32-unknown-emscripten`); static HTML served over HTTP  
**Project Type**: WASM library + static web application  
**Performance Goals**: Build completes in < 5 min on a developer machine; page load < 3 s on 2G; total initial payload (HTML + CSS + JS glue + WASM) < 5 MB uncompressed (ephemeris `.data` file counted separately)  
**Constraints**: No JS frameworks; no wasm-bindgen; no base64-inlined WASM; offline-capable after first load; ephemeris `.se1` files not committed to Git  
**Scale/Scope**: Single developer verification milestone; no concurrent users in scope

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Rust-First Computation | ✅ PASS | All computation (sun longitude) is in Rust/WASM; JS layer only calls the export |
| II. Lean Web Presentation | ✅ PASS | `index.html` is plain HTML + vanilla JS `<script src>`; zero JS libraries or frameworks |
| III. Contract-Driven WASM API | ✅ PASS | Contract file (`contracts/wasm-api-v1.md`) defined in Phase 1 before implementation |
| IV. Mobile-First Design | ✅ PASS | `index.html` uses responsive CSS-only layout; no JS-driven layout |
| V. Correctness & Accuracy | ✅ PASS | SC-003 requires output within 0.001° of reference; `swe_set_ephe_path("/ephe")` ensures file-based (not Moshier) ephemeris is used |
| VI. Localization in Rust | ✅ PASS | No user-facing strings in this feature; console output is developer-only |

**Post-design re-check**: See bottom of this file. ✅ Passes all six principles.

## Project Structure

### Documentation (this feature)

```text
specs/001-infrastructure-tooling-setup/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── wasm-api-v1.md   # Phase 1 output — WASM export contract
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created here)
```

### Source Code (repository root)

```text
vendor/
└── swisseph/            # Git submodule — Swiss Ephemeris C source (pinned tag)

ephe/                    # NOT committed (listed in .gitignore)
├── semo_18.se1          # Moon ephemeris data — download from Astrodienst FTP
└── sepl_18.se1          # Planet ephemeris data — download from Astrodienst FTP

astro-wasm/              # Rust crate — wasm32-unknown-emscripten target
├── Cargo.toml
├── build.rs             # Links libswe.a; sets LIBSWE_DIR search path
└── src/
    └── lib.rs           # pub extern "C" fn bridge(op_ptr, input_ptr, output_ptr, output_max_len) → i32

web/                     # Static web assets
├── index.html           # "Work in Progress" page + WASM loader
└── style.css            # Minimal mobile-first stylesheet

dist/                    # Build output — NOT committed (.gitignore)
├── index.html           # Copied from web/
├── style.css            # Copied from web/
├── astro.js             # Emscripten JS glue — emitted by build
├── astro.wasm           # WASM binary — emitted by build
└── astro.data           # Preloaded ephemeris data bundle — emitted by build

build.sh                 # Single build entry point
.cargo/
└── config.toml          # [target.wasm32-unknown-emscripten] linker = "emcc"
.gitignore               # Includes: /ephe/, /dist/, /astro-wasm/target/
```

**Structure Decision**: Hybrid static-web + WASM library layout. A dedicated `astro-wasm/`
Rust crate keeps Cargo concerns isolated. `vendor/swisseph/` is a Git submodule so its
version is pinned without committing C source. `web/` holds hand-authored HTML/CSS kept
separate from generated `dist/`. `build.sh` is the sole orchestrator for the sequence:
(1) compile `vendor/swisseph/*.c` → `libswe.a` via `emcc`, (2) `cargo build --target
wasm32-unknown-emscripten` → produces `.js` + `.wasm`, (3) re-link with `--preload-file`
to embed ephemeris data, (4) copy all outputs + `web/` assets into `dist/`.

## Complexity Tracking

No constitution violations. No complexity justification required.
