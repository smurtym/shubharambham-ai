# Feature Specification: Testing and Build Tooling

**Feature Branch**: `002-testing-and-build`  
**Created**: 2026-03-14  
**Status**: Draft  
**Input**: User description: "Playwright UI testing, Rust unit tests in build, Parcel minifier via npm — remove test-wasm.mjs, add Playwright for UI testing, add Rust unit tests that run as part of build, implement Parcel minifier via npm build"

## Overview

This feature upgrades the developer toolchain in three independent but complementary areas:

1. **Remove the hand-rolled Node.js polyfill test** (`test-wasm.mjs`) and replace it with Playwright end-to-end tests that run against a real browser environment.
2. **Add Rust unit tests** inside `astro-wasm/` that validate all public Rust functions — including `bridge()` routing logic and Swiss Ephemeris wrapper functions — and wire them into `build.sh` alongside Playwright so they gate every end-to-end build.
3. **Introduce Vite (npm) as the front-end build tool**, so `dist/` is produced by a proper bundler/minifier pipeline instead of raw `cp` commands. Vite's `public/` directory pattern provides a clean, zero-config solution for the Emscripten WASM artifacts, and its architecture scales naturally to the full SPA roadmap (date/place selectors, chart rendering, matchmaking page, client-side routing).

No new user-visible functionality is delivered. All outcomes are developer-experience and code-quality improvements.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Playwright replaces hand-rolled smoke test (Priority: P1)

A developer runs a single command and a real Chromium browser loads the locally served page, exercises the `bridge()` WASM call, and the test reports pass/fail from actual browser output. The fragile Node.js polyfill harness (`test-wasm.mjs`) is deleted.

**Why this priority**: The existing `test-wasm.mjs` requires extensive polyfills for browser APIs and works around the `-sENVIRONMENT=web` constraint via indirect eval — it is brittle and hard to maintain. Playwright runs the real browser, eliminating the mismatch and providing a reliable regression guard for all future work.

**Independent Test**: Run `npm test` from the repo root; Chromium launches headlessly, loads the served page, and the Sun longitude value is asserted in the browser console output. No manual browser interaction is required.

**Acceptance Scenarios**:

1. **Given** the WASM build artifacts are in `dist/`, **When** `npm test` is run, **Then** Playwright launches a local HTTP server on port 4173, opens `http://localhost:4173` in headless Chromium, and asserts the page body contains "Work in Progress".
2. **Given** the WASM module initialises without error, **When** `npm test` is run, **Then** the Playwright test captures the `console.log` output from `onRuntimeInitialized` and asserts the logged Sun longitude is within 0.5° of the known J2000 reference value (280.37°).
3. **Given** `test-wasm.mjs` exists in the repo, **When** this feature is merged, **Then** `test-wasm.mjs` is deleted and is no longer referenced anywhere in the codebase or documentation.
4. **Given** Playwright is not installed, **When** `npm install` is run, **Then** the Playwright Chromium browser binaries are downloaded automatically via the `postinstall` script (`npx playwright install chromium`) — no separate command is required.

---

### User Story 2 — Rust unit tests and Playwright tests gate every build (Priority: P2)

A developer running `./build.sh` sees Rust unit tests execute and fail the build if any test is red. Tests cover all public Rust functions — including `bridge()` routing logic and every wrapper function over Swiss Ephemeris — running against `libswe.a` on the native host. After the Vite build step, `build.sh` also runs `npm test` (Playwright) as the final end-to-end gate. Both test steps can be individually skipped via command-line flags for emergency quick-fix builds.

**Why this priority**: The Rust layer has no automated tests today. Unit tests at this level catch JSON-parsing bugs, routing errors, and incorrect Swiss Ephemeris wrapper behaviour before a full WASM compile cycle is needed, shortening the feedback loop significantly. Running Playwright as the final `build.sh` step ensures every build is end-to-end verified without requiring a separate manual command.

**Independent Test**: Run `./build.sh`; the Rust test step (`cargo test`) runs before the WASM link step; `npm test` runs after `npm run build` as the final step; a deliberately broken Rust or Playwright test causes `build.sh` to exit non-zero with a clear message. Run `./build.sh --skip-rust-tests` to bypass `cargo test`; run `./build.sh --skip-playwright-tests` to bypass `npm test`.

**Acceptance Scenarios**:

1. **Given** the `astro-wasm` crate is present, **When** `./build.sh` runs, **Then** `cargo test` is executed for the native target (not WASM) before the WASM compile step, and any test failure aborts the build with a non-zero exit code.
2. **Given** a `#[cfg(test)]` module in `astro-wasm/src/lib.rs`, **When** `cargo test` runs, **Then** every WASM-exported function (`#[no_mangle] pub extern "C"`) and every private handler/wrapper function called from `bridge()` has at least one unit test; tests of Swiss Ephemeris wrapper functions invoke the natively compiled `libswe` to assert correct output values.
3. **Given** all Rust tests pass, **When** `./build.sh` continues, **Then** the WASM compile step proceeds normally as before.
4. **Given** Rust wrapper functions call Swiss Ephemeris via FFI, **When** `cargo test` runs on the native host, **Then** `build.rs` detects the non-Emscripten `TARGET` and uses the `cc` crate to compile the swisseph C sources natively at build time — `cargo test` is fully self-contained without requiring `build.sh` to have run first.
5. **Given** `./build.sh` is invoked with `--skip-rust-tests`, **When** the build executes, **Then** the `cargo test` step is bypassed and a prominent warning is printed; the WASM compile and Vite build steps proceed normally.
6. **Given** `./build.sh` is invoked with `--skip-playwright-tests`, **When** the build executes, **Then** the `npm test` step is bypassed and a prominent warning is printed; all preceding steps complete normally.
7. **Given** `./build.sh` runs without skip flags and all prior steps succeed, **When** `npm run build` completes, **Then** `npm test` is invoked automatically as the final step, and a Playwright test failure causes `build.sh` to exit non-zero.

---

### User Story 3 — Vite builds and minifies web assets via npm build (Priority: P3)

A developer runs `./build.sh` and the web assets (`index.html`, `style.css`) are processed by Vite through an npm build script, producing minified output in `dist/`. The raw `cp web/* dist/` step is replaced by the Vite build. Emscripten WASM artifacts are written to `public/` and Vite copies them verbatim into `dist/` without any processing.

**Why this priority**: Minification reduces real payload for mobile users and establishes a production-grade front-end pipeline that scales to the full SPA roadmap — date/place selectors, horoscope chart rendering, matchmaking page, and client-side routing — without rearchitecting. Vite's `public/` directory solves the Emscripten artifact pass-through problem natively with zero configuration.

**Independent Test**: Run `npm run build` from the repo root; inspect `dist/index.html` and confirm it is minified; confirm `dist/style.css` is minified; confirm `dist/astro.js`, `dist/astro.wasm`, and `dist/astro.data` are present and byte-for-byte identical to the files in `public/`.

**Acceptance Scenarios**:

1. **Given** a `package.json` at repo root with Vite as a dependency and a `build` script, **When** `npm run build` is run, **Then** Vite processes `web/index.html` as the entry point and writes minified output to `dist/`.
2. **Given** the existing `web/style.css`, **When** Vite builds, **Then** the output `dist/style.css` is minified (whitespace-collapsed, comments stripped) and functionally equivalent to the source.
3. **Given** `public/astro.js`, `public/astro.wasm`, `public/astro.data` exist (written there by the WASM build step), **When** Vite builds, **Then** those three files are copied verbatim to `dist/` without renaming, hashing, or any transformation.
4. **Given** `./build.sh` currently calls `cp web/index.html web/style.css dist/`, **When** this feature is merged, **Then** that `cp` step is replaced by `npm run build`, and the Emscripten `-o` output target is changed from `dist/astro.js` to `public/astro.js`, so a single `./build.sh` invocation produces a fully minified `dist/`.
5. **Given** `node_modules/` is absent, **When** `./build.sh` detects this, **Then** it runs `npm install` automatically before the Vite build step, printing a descriptive log message.

---

### Edge Cases

- What if `npm` is not installed on the developer's machine? `build.sh` must detect its absence in the prerequisite check phase and exit with a clear error, just like it does for `emcc`.
- Vite's `public/` directory is the canonical solution to the WASM pass-through problem: files placed there are copied verbatim to `dist/` with no hashing, transformation, or bundling. The Emscripten build step must write `astro.js`/`.wasm`/`.data` to `public/`, not `dist/`, and `vite.config.ts` must set `build.outDir = 'dist'` and leave `publicDir = 'public'` (the default). If any future refactor moves the `<script src>` reference into a JS import, that import must be excluded from Vite's module graph.
- Each Swiss Ephemeris wrapper function is path-agnostic: it never calls `swe_set_ephe_path`. The `bridge()` handler calls `swe_set_ephe_path("/ephe\0")` once at the start of each WASM invocation (Emscripten virtual FS path). Tests call `swe_set_ephe_path` with `concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe\0")` in a `#[cfg(test)]` setup step. `build.sh --skip-rust-tests` bypasses `cargo test` and `build.sh --skip-playwright-tests` bypasses `npm test`; each prints a prominent warning and must never be the default mode.
- What if Playwright cannot find a free port for its web server? `npx serve` does not support dynamic/ephemeral port assignment (`--port 0`). The Playwright `webServer` configuration MUST use fixed **port 4173** (`--listen 4173`) and `playwright.config.ts` MUST declare `port: 4173` so Playwright knows when the server is ready.
- What if `dist/` does not exist when `npm test` tries to serve it? A `beforeAll` hook at the top of the Playwright spec must check for the existence of `dist/astro.js` using Node's `fs` module and throw an error with the message `"dist/astro.js not found — run ./build.sh first"` if the file is absent. This fails the entire test suite immediately with a clear, actionable signal rather than a cryptic 404 or server timeout. `npm test` and `./build.sh` remain fully independent.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The file `test-wasm.mjs` MUST be deleted from the repository.
- **FR-002**: A `package.json` MUST exist at the repository root declaring `@playwright/test`, `vite`, and `typescript` as dev dependencies. Config files (`vite.config.ts`, `playwright.config.ts`) and test files (`.spec.ts`) MUST be written in TypeScript. `package.json` MUST include a `"postinstall": "npx playwright install chromium"` script so that `npm install` is the single command required to install all dependencies including browser binaries.
- **FR-003**: Running `npm test` MUST execute Playwright tests headlessly against `dist/`. Playwright's `webServer` configuration MUST automatically start a static file server (`npx serve dist/ --listen 4173`) on **fixed port 4173** before tests run and tear it down afterwards. (`npx serve` does not support dynamic/ephemeral port assignment; port 4173 is the fixed port used by this project.) `build.sh` MUST NOT start or manage any web server. A `beforeAll` hook at the top of the Playwright spec MUST check for the existence of `dist/astro.js` using Node's `fs` module and throw an error with the message `"dist/astro.js not found — run ./build.sh first"` if the file is absent, failing the suite immediately with an actionable message.
- **FR-004**: Playwright tests MUST assert the page body contains "Work in Progress".
- **FR-005**: Playwright tests MUST assert the Sun longitude logged to the browser console (from `onRuntimeInitialized`) is a floating-point number within 0.5° of 280.37°. The expected console log format is `"Sun longitude: <value>"` (e.g. `"Sun longitude: 280.46"`). The assertion MUST use `page.waitForEvent('console', { predicate: msg => msg.text().startsWith('Sun longitude:'), timeout: 10000 })` registered before `page.goto()` to reliably capture the async WASM init log.
- **FR-006**: `./build.sh` MUST run `cargo test` (native target) for `astro-wasm` before the WASM compile step, and MUST abort the build if any test fails. Before invoking `cargo test`, `build.sh` MUST unset `LIBSWE_DIR` to prevent the WASM library path from leaking into the native build. After `cargo test` completes, `build.sh` MUST export `LIBSWE_DIR` pointing at `lib/` before running `cargo build --target wasm32-unknown-emscripten`.
- **FR-007**: Every WASM-exported function (`#[no_mangle] pub extern "C"`) **and** every private handler/wrapper function called from `bridge()` MUST have at least one unit test. Tests MUST cover: (a) `bridge()` routing — verifying an unrecognised operation string returns a documented error code; (b) each private Swiss Ephemeris wrapper function (e.g. `handle_sun_longitude`) — asserting correct output values by invoking the natively compiled `libswe` on the host. The `#[cfg(test)]` module MUST call `swe_set_ephe_path` with `concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe\0")` in a shared setup step before invoking any wrapper under test.
- **FR-008**: `build.rs` MUST branch on the `TARGET` environment variable set by Cargo. When `TARGET` contains `emscripten`, it MUST emit the existing Emscripten-specific link args (`--preload-file`, `-o`, `LIBSWE_DIR` search path). When `TARGET` does NOT contain `emscripten` (native `cargo test`), it MUST use the `cc` crate to compile the swisseph C sources from `vendor/swisseph/` directly into a static library linked into the test binary — no Emscripten toolchain or pre-compiled `libswe.a` is required. The `cc` crate MUST be added as a `[build-dependencies]` entry in `Cargo.toml`.
- **FR-009**: A `build` script in `package.json` MUST invoke Vite to build and minify `web/index.html` (and its referenced CSS) into `dist/`.
- **FR-010**: The Emscripten WASM build step MUST write `astro.js`, `astro.wasm`, and `astro.data` to the `public/` directory (not `dist/`). Vite MUST copy them verbatim to `dist/` via its `publicDir` pass-through — they must never be processed, renamed, or hashed by Vite.
- **FR-011**: The `cp web/index.html web/style.css dist/` step in `build.sh` MUST be replaced by `npm run build` (Vite). The Emscripten `-o` output path in `build.rs` MUST be updated from `dist/astro.js` to `public/astro.js`.
- **FR-012**: `build.sh` MUST check that `npm` is available in the prerequisite phase, and MUST run `npm install` automatically if `node_modules/` is absent.
- **FR-013**: `node_modules/`, `.vite/`, `dist/` (when produced solely by Vite), and Playwright output directories (`playwright-report/`, `test-results/`) MUST be excluded from Git via `.gitignore`; the `dist/` entry MUST be verified as already present before adding to avoid duplication. `public/astro.js`, `public/astro.wasm`, and `public/astro.data` MUST also be excluded from Git.
- **FR-014**: `./build.sh` MUST invoke `npm test` as the final step, after `npm run build`, running the Playwright suite against the freshly built `dist/`. A failing Playwright test MUST cause `build.sh` to exit non-zero.
- **FR-015**: `./build.sh` MUST accept `--skip-rust-tests` and `--skip-playwright-tests` flags. When `--skip-rust-tests` is set, the `cargo test` step is skipped with a clearly visible warning. When `--skip-playwright-tests` is set, the `npm test` step is skipped with a clearly visible warning. Warnings MUST be prefixed with `⚠ WARNING:` and printed to stdout via `echo`, so they are captured in build logs and visually distinct from normal progress lines. These flags are intended for emergency quick-fix builds only and MUST NOT be set by default.
- **FR-016**: Swiss Ephemeris wrapper functions MUST NOT call `swe_set_ephe_path` internally. The ephemeris path MUST be set by the caller (`bridge()` handler in production; `#[cfg(test)]` setup block in tests) before invoking any wrapper. This removes the hardcoded `"/ephe\0"` Emscripten virtual path from wrapper bodies and ensures wrappers are path-agnostic.

### Key Entities

- **`package.json`**: Root-level npm manifest declaring Playwright and Vite as dev dependencies, with `build` (Vite) and `test` (Playwright) scripts.
- **`vite.config.ts`**: Vite configuration (TypeScript) — entry point `web/index.html`, `build.outDir = 'dist'`, `publicDir = 'public'` (default, passes WASM artifacts through unchanged).
- **`public/`**: Vite pass-through directory; Emscripten writes `astro.js`, `astro.wasm`, `astro.data` here; Vite copies them verbatim to `dist/`. Not committed to Git.
- **`playwright.config.ts`**: Playwright configuration (TypeScript) — base URL, headless mode, test directory, and built-in `webServer` configuration pointing at `dist/`.
- **`tests/`**: Directory of Playwright `.spec.ts` browser test files (TypeScript).
- **`astro-wasm/src/lib.rs` — `#[cfg(test)]` module**: Rust unit tests covering all public functions — `bridge()` routing logic and Swiss Ephemeris wrapper functions. Tests of wrapper functions invoke Swiss Ephemeris via FFI against the natively compiled library (built by `build.rs` via `cc` crate).
- **`astro-wasm/build.rs`**: Branches on Cargo's `TARGET` env var. Emscripten target: existing link args (`--preload-file`, `-o public/astro.js`, `LIBSWE_DIR`). Native target: uses `cc` crate to compile `vendor/swisseph/*.c` into a static library for native linking, making `cargo test` fully self-contained.
- **`cc` crate** (`[build-dependencies]`): Rust build dependency used by `build.rs` on native targets to compile swisseph C sources. Not present in the final WASM binary.
- **`tsconfig.json`**: Minimal TypeScript config scoping the config and test files; does not affect the Rust/WASM build.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `npm test` completes in under 60 seconds on a machine with Playwright browsers already installed, with all tests green.
- **SC-002**: `./build.sh` output includes lines confirming: (a) Rust tests passed before the WASM compile step; (b) Playwright tests passed as the final step. Total build time (including all test steps) completes in under 3 minutes on a machine where Playwright browser binaries are already installed and Cargo's build cache is warm.
- **SC-003**: `dist/index.html` produced by Vite is smaller in bytes than the unminified `web/index.html` (whitespace and comment removal confirmed); `dist/astro.js` byte count matches `public/astro.js` exactly (no transformation).
- **SC-004**: Zero manual browser steps are required to verify Sun longitude correctness — the result is confirmed automatically by the Playwright test suite on every build.
- **SC-005**: `test-wasm.mjs` is absent from the repository after this feature is merged.

---

## Clarifications

### Session 2026-03-14

- Q: How should native `libswe.a` be produced for `cargo test`? → A: `build.rs` branches on Cargo's `TARGET` env var using the `cc` crate — when `TARGET` is not emscripten, it compiles the swisseph C sources from `vendor/swisseph/` natively at build time. `cargo test` is fully self-contained without requiring `build.sh` to run first. `cc` is added as a `[build-dependencies]` entry in `Cargo.toml`.
- Q: How should Playwright serve `dist/` during the test run? → A: Playwright's built-in `webServer` config launches `npx serve dist/ --listen 4173` on **fixed port 4173** when `npm test` runs and tears it down after. (`npx serve` does not support ephemeral ports; port 4173 was chosen to avoid conflicts with common dev servers.) `build.sh` has no responsibility for serving — it only produces build artifacts.
- Q: How should Playwright capture the async `console.log` from `onRuntimeInitialized` to assert the Sun longitude? → A: Use `page.waitForEvent('console', filter)` registered before `page.goto()` — the built-in await-friendly API available in Playwright ≥ 1.44 (we are pinning ≥ 1.51).
- Q: Should config and test files use TypeScript or plain JavaScript? → A: TypeScript from the start — `vite.config.ts`, `playwright.config.ts`, and `.spec.ts` test files; `typescript` added as a dev dependency.
- Q: What should happen when `npm test` is run but `dist/` is missing or empty? → A: A `beforeAll` hook in the Playwright spec checks for `dist/astro.js` and throws immediately with a descriptive message ("dist/astro.js not found — run ./build.sh first") if the file is absent. `npm test` and `./build.sh` remain fully independent.
- Q: How should Playwright browser binaries be installed? → A: A `postinstall` script in `package.json` runs `npx playwright install chromium` automatically after `npm install`, making a single `npm install` the only setup step required.
- Q: How should tests supply the ephemeris path to `swe_set_ephe_path` on the native host? → A: Tests hardcode the path using `concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe")` directly in the `#[cfg(test)]` module. `CARGO_MANIFEST_DIR` is always set by Cargo; no env var, config, or `build.rs` change is needed.
- Q: How should production wrapper functions handle the ephemeris path so they work under both WASM and native `cargo test`? → A: Wrapper functions MUST NOT call `swe_set_ephe_path` internally. Callers (bridge handler in production, `#[cfg(test)]` setup block in tests) are responsible for calling `swe_set_ephe_path` once before invoking any wrapper.
- Q: How should `build.sh` invoke `cargo test` to avoid the WASM `LIBSWE_DIR` interfering with the native build? → A: `build.sh` explicitly unsets `LIBSWE_DIR` before running `cargo test` (native), then exports `LIBSWE_DIR` pointing at `lib/` before running `cargo build --target wasm32-unknown-emscripten`. The unset is a safety measure; `build.rs` on native targets uses the `cc` crate and ignores `LIBSWE_DIR` regardless.

## Assumptions

- Node.js ≥ 18 and npm ≥ 9 are available on the developer machine. `build.sh` will check for `npm` in the prerequisite phase.
- Playwright Chromium browser binaries are installed automatically via a `postinstall` script (`npx playwright install chromium`) in `package.json`. A single `npm install` is therefore the complete setup step. Internet access is assumed for initial setup.
- The WASM artifacts in `dist/` are always produced by the Rust/Emscripten build step before the Playwright tests are run. If `dist/astro.js` is absent, a `beforeAll` hook in the Playwright spec throws immediately with `"dist/astro.js not found — run ./build.sh first"`, keeping `npm test` and `./build.sh` fully independent.
- TypeScript (dev dependency, no `strict` mode required initially) is used for `vite.config.ts`, `playwright.config.ts`, and all `.spec.ts` test files. A minimal `tsconfig.json` scopes TS to the config/test files only and does not affect the Rust/WASM build pipeline.
- Vite 5 is used. `npm run build` invokes `vite build` in production mode, which minifies HTML (via built-in rollup pipeline) and CSS (via lightningcss, included in Vite 5).
- `vite.config.ts` sets `root = 'web'`, `publicDir = '../public'` (relative to root), and `build.outDir = '../dist'`. This mirrors the Vite convention for projects where the source root is a subdirectory.
- `build.rs` branches on Cargo's `TARGET` env var: when `TARGET` contains `emscripten`, it emits the existing Emscripten link args and uses `LIBSWE_DIR`; when `TARGET` is native, it uses the `cc` crate to compile `vendor/swisseph/*.c` into a static library (`LIBSWE_DIR` is unset by `build.sh` before this step as a safety measure, but `build.rs` ignores it on native targets regardless).
- `public/` is a build-time artefact directory, not a source directory. It is created by `./build.sh` (Emscripten step) and populated before `npm run build` runs. It is listed in `.gitignore`.
- Playwright's `webServer` in `playwright.config.ts` uses `command: 'npx serve dist/ --listen 4173'`, `port: 4173`, and `reuseExistingServer: !process.env.CI`. The `serve` package is invoked via `npx` (no install required) on fixed port 4173. `build.sh` is entirely decoupled from test serving.
