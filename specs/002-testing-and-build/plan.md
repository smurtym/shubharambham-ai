# Implementation Plan: Testing and Build Tooling

**Branch**: `002-testing-and-build` | **Date**: 2026-03-14 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `/specs/002-testing-and-build/spec.md`

## Summary

Replace `test-wasm.mjs` with Playwright E2E tests; add Rust unit tests for all public functions (including Swiss Ephemeris wrappers, using the `cc` crate to compile swisseph natively in `build.rs`); introduce Vite 5 as the front-end build tool (replacing `cp web/* dist/`) with `public/` as the Emscripten WASM artifact pass-through directory. `build.sh` gains `cargo test` (before WASM compile), `npm run build` (replaces `cp`), `npm test` (final E2E gate), and `--skip-rust-tests` / `--skip-playwright-tests` flags.

## Technical Context

**Language/Version**: Rust stable (≥1.75) for WASM/tests; TypeScript 5.x (dev only, config/tests); Bash  
**Primary Dependencies**: Emscripten ≥4.0 (WASM); Vite 5 (front-end build); `@playwright/test` ≥1.51 (E2E); `cc` crate 1.x (native C compilation in `build.rs`)  
**Storage**: N/A (static file output only)  
**Testing**: `cargo test` (native host, Rust unit tests); `npx playwright test` (E2E browser tests)  
**Target Platform**: `wasm32-unknown-emscripten` (production WASM); native Linux/macOS host (Rust tests); headless Chromium (E2E tests)  
**Project Type**: Static web application with WASM computation core  
**Performance Goals**: `npm test` completes ≤60s (browsers installed); full `./build.sh` completes ≤3min (warm cache)  
**Constraints**: WASM artifacts must not be renamed or hashed by Vite; `cargo test` must not require Emscripten; `<script src="astro.js">` in HTML must not be processed by Vite bundler

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Rust-First Computation | ✅ PASS | No computation moves to JS. `cc` crate is build-time only; not in WASM binary. |
| II. Lean Web Presentation | ⚠️ JUSTIFIED | Vite is a bundler (cautioned against in constitution). Justified: see Complexity Tracking. Build output remains plain HTML/CSS/JS with no framework. |
| III. Contract-Driven WASM API | ✅ PASS | No new WASM API contracts. `wasm-api-v1.md` is unchanged. FR-016 refactor is internal and invisible to JS callers. |
| IV. Mobile-First Design | ✅ N/A | No UI changes in this feature. |
| V. Correctness & Accuracy | ✅ PASS | FR-007 requires reference-value tests for all wrapper functions (directly mandated by constitution §V). |
| VI. Localization in Rust | ✅ N/A | No new strings introduced. |

**Post-design re-check**: All 6 principles still pass. No new violations introduced by Phase 1 design.

## Project Structure

### Documentation (this feature)

```text
specs/002-testing-and-build/
├── plan.md         ← this file
├── research.md     ← Phase 0 output
├── data-model.md   ← Phase 1 output
├── quickstart.md   ← Phase 1 output
└── tasks.md        ← Phase 2 output (speckit.tasks — not yet created)
```

Note: No new `contracts/` files — the existing `wasm-api-v1.md` contract is unchanged.

### Source Code (repository root)

```text
/                               ← repo root
├── build.sh                    ← MODIFIED: skip flags, cargo test, npm steps
├── package.json                ← NEW
├── vite.config.ts              ← NEW
├── playwright.config.ts        ← NEW
├── tsconfig.json               ← NEW
├── tests/
│   └── astro.spec.ts           ← NEW (Playwright E2E)
├── public/                     ← NEW dir (created by build.sh; gitignored)
│   └── astro.{js,wasm,data}    ← build artifacts (Emscripten output)
├── web/                        ← UNCHANGED source (Vite entry point)
│   ├── index.html
│   └── style.css
├── dist/                       ← produced by `npm run build` (was: cp)
├── astro-wasm/
│   ├── Cargo.toml              ← MODIFIED: add [build-dependencies] cc = "1"
│   ├── build.rs                ← MODIFIED: TARGET branching
│   └── src/
│       └── lib.rs              ← MODIFIED: remove swe_set_ephe_path from wrappers; add #[cfg(test)]
├── .gitignore                  ← MODIFIED: add node_modules/, public/astro.*, etc.
└── test-wasm.mjs               ← DELETED
```

**Structure Decision**: Single-project layout with in-place modifications. No new source directories other than `tests/` and `public/`. All changes are additive or targeted modifications to existing files.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| Vite (bundler) introduced despite constitution preferring shell scripts | Vite's `publicDir` pass-through is the only clean solution for WASM artifact verbatim copy. The SPA roadmap (date/place selectors, chart, matchmaking) requires a module bundler to scale without rearchitecting. | Pure `cp`-based approach: cannot minify HTML/CSS correctly, cannot handle future JS module splitting. `html-minifier-terser` + `lightningcss` as separate npm tools: two tools with custom shell wiring = more complexity than one Vite config. Shell script hash-renaming workaround: fragile and non-idiomatic. |
