# shubharambham-ai Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-03-14

## Active Technologies
- Rust stable (≥1.75) for WASM/tests; TypeScript 5.x (dev only, config/tests); Bash + Emscripten ≥4.0 (WASM); Vite 5 (front-end build); `@playwright/test` ≥1.51 (E2E); `cc` crate 1.x (native C compilation in `build.rs`) (002-testing-and-build)
- N/A (static file output only) (002-testing-and-build)

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
- 002-testing-and-build: Added Rust stable (≥1.75) for WASM/tests; TypeScript 5.x (dev only, config/tests); Bash + Emscripten ≥4.0 (WASM); Vite 5 (front-end build); `@playwright/test` ≥1.51 (E2E); `cc` crate 1.x (native C compilation in `build.rs`)
- 002-testing-and-build: Added [if applicable, e.g., PostgreSQL, CoreData, files or N/A]

- 001-infrastructure-tooling-setup: Added Rust 1.88 (stable); C99 (swisseph sources) + `swisseph` C library (Git submodule, `vendor/swisseph/`); Rust `std` only (no external crates for this feature)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
