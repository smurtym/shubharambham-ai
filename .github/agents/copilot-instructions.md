# shubharambham-ai Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-03-21

## Active Technologies
- Rust stable (≥1.75) for WASM/tests; TypeScript 5.x (dev only, config/tests); Bash + Emscripten ≥4.0 (WASM); Vite 5 (front-end build); `@playwright/test` ≥1.51 (E2E); `cc` crate 1.x (native C compilation in `build.rs`) (002-testing-and-build)
- N/A (static file output only) (002-testing-and-build)
- Rust stable (`cdylib`) compiled via Emscripten; vanilla ES6+ JS + `serde_json 1.x` (alloc feature), `cc 1.x` (build), Swiss Ephemeris C library (FFI via `build.rs`), Emscripten toolchain (003-arch-layout)
- N/A — ephemeris data files embedded in Emscripten virtual filesystem at `/ephe` (003-arch-layout)
- Rust 2021 (stable toolchain), compiled to WASM via Emscripten (`wasm32-unknown-emscripten`) + `serde 1` + `serde_json 1` (alloc features — already in `Cargo.toml`); no new crates required (004-city-data)
- Hardcoded `&'static [CityRecord]` array compiled into the WASM binary (004-city-data)
- Rust 2021 (stable toolchain) + `serde 1` + `serde_json 1` (already present); `chrono` + `chrono-tz` (new — IANA timezone conversion); Swiss Ephemeris via existing C FFI (006-horoscope-positions)
- N/A — pure computation (006-horoscope-positions)
- TypeScript 5.x strict; React 18.3; MUI 6.0 + React 18, MUI v6, Emotion, `@vitejs/plugin-react`, Vite 5 (008-horoscope-ui-polish)
- N/A — all state is in-memory; city data served by WASM (008-horoscope-ui-polish)
- Rust 2021 (stable toolchain) + `serde 1` + `serde_json 1` (serialization); `chrono 0.4` + `chrono-tz 0.9` (date arithmetic, timezone); Swiss Ephemeris via existing C FFI — all already present in `Cargo.toml` (009-vimsottari-dasa)

- Rust 1.88 (stable); C99 (swisseph sources) + `swisseph` C library (Git submodule, `vendor/swisseph/`); Rust `std` only (no external crates for this feature) (001-infrastructure-tooling-setup)

## Project Structure

```text
backend/
frontend/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.88 (stable); C99 (swisseph sources): Follow standard conventions

## Recent Changes
- 009-vimsottari-dasa: Added Rust 2021 (stable toolchain) + `serde 1` + `serde_json 1` (serialization); `chrono 0.4` + `chrono-tz 0.9` (date arithmetic, timezone); Swiss Ephemeris via existing C FFI — all already present in `Cargo.toml`
- 008-horoscope-ui-polish: Added TypeScript 5.x strict; React 18.3; MUI 6.0 + React 18, MUI v6, Emotion, `@vitejs/plugin-react`, Vite 5
- 006-horoscope-positions: Added Rust 2021 (stable toolchain) + `serde 1` + `serde_json 1` (already present); `chrono` + `chrono-tz` (new — IANA timezone conversion); Swiss Ephemeris via existing C FFI


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
