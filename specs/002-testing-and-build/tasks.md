---
description: "Task list for feature 002 — Testing and Build Tooling"
---

# Tasks: Testing and Build Tooling

**Feature**: `002-testing-and-build`
**Input**: [spec.md](spec.md) (16 FRs, 3 User Stories), [plan.md](plan.md), [data-model.md](data-model.md)
**References**: [research.md](research.md) (11 decisions), [quickstart.md](quickstart.md)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on in-progress tasks)
- **[Story]**: Which user story ([US1], [US2], [US3])
- Tests are **implementation** for this feature — test-writing tasks are not optional

---

## Phase 1: Setup (Cleanup & Scaffolding)

**Purpose**: Remove the obsolete Node.js polyfill harness and update `.gitignore` before any story work begins.

- [X] T001 Delete `test-wasm.mjs` from the repository root (FR-001)
- [X] T002 [P] Update `.gitignore` — append `node_modules/`, `.vite/`, `playwright-report/`, `test-results/`, `public/astro.js`, `public/astro.wasm`, `public/astro.data`; verify `dist/` is already excluded before adding to avoid a duplicate entry (FR-013)

**Checkpoint**: Obsolete test harness gone; VCS hygiene updated — user story work can now begin

---

## Phase 2: Foundational (Blocking Prerequisites)

**No cross-cutting foundational tasks for this feature.** Every prerequisite is either already in Phase 1 or scoped to a specific user story. Proceed directly to Phase 3.

---

## Phase 3: User Story 1 — Playwright Replaces Hand-Rolled Smoke Test (Priority: P1) 🎯 MVP

**Goal**: Create the npm project scaffold and a Playwright E2E test suite that launches headless Chromium, verifies the page body, and captures the Sun longitude from the `onRuntimeInitialized` console output. `npm test` becomes the single command to run all browser tests.

**Independent Test**: Run `npm install` (installs Playwright and Chromium binaries via `postinstall`). Run an unmodified `./build.sh` to produce `dist/` via the existing `cp` step. Then run `npm test` — both Playwright assertions must pass.

### Implementation for User Story 1

- [X] T003 [P] [US1] Create `package.json` at repo root — `devDependencies`: `"@playwright/test": ">=1.51.0"`, `"vite": "^5.0.0"`, `"typescript": "^5.0.0"`; `scripts`: `"build": "vite build"`, `"test": "playwright test"`, `"postinstall": "npx playwright install chromium"` (FR-002)
- [X] T004 [P] [US1] Create `tsconfig.json` at repo root — minimal config: `"include": ["**/*.config.ts", "tests/**/*.ts"]`; `"compilerOptions"`: `"strict": true`, `"module": "ESNext"`, `"moduleResolution": "bundler"`, `"target": "ES2022"` — does not affect the Rust/WASM build (FR-002)
- [X] T005 [US1] Create `playwright.config.ts` at repo root — `webServer: { command: 'npx serve dist/ --listen 4173', port: 4173, reuseExistingServer: !process.env.CI }`, `use: { baseURL: 'http://localhost:4173', headless: true }`, `testDir: './tests'` (FR-003)
- [X] T006 [US1] Create `tests/astro.spec.ts` — (a) `import fs from 'node:fs'`; (b) `test.beforeAll`: check `fs.existsSync('dist/astro.js')` and throw `new Error("dist/astro.js not found — run ./build.sh first")` if absent; (c) test "page body contains Work in Progress": assert `page.textContent('body')` contains `'Work in Progress'`; (d) test "Sun longitude at J2000 ≈ 280.37°": register `page.waitForEvent('console', { predicate: msg => msg.text().startsWith('Sun longitude:'), timeout: 10000 })` **before** `page.goto('/')` — the `onRuntimeInitialized` callback logs in the format `"Sun longitude: <value>"` (e.g. `"Sun longitude: 280.46"`); resolve the event, extract the numeric substring after `"Sun longitude: "`, parse with `parseFloat`, and assert `Math.abs(longitude - 280.37) < 0.5` (FR-003, FR-004, FR-005)

**Checkpoint**: `npm test` runs headless Chromium and both assertions pass against a pre-built `dist/` — US1 is independently verified

---

## Phase 4: User Story 2 — Rust Unit Tests and Playwright Tests Gate Every Build (Priority: P2)

**Goal**: Add Rust unit tests for all public functions (using the `cc` crate in `build.rs` for self-contained native compilation), and wire both `cargo test` and `npm test` into `build.sh` as mandatory gates with `--skip-*` bypass flags.

**Independent Test**: `cd astro-wasm && cargo test` must compile without Emscripten, link natively via the `cc` crate, and all tests pass. `./build.sh` must show Rust tests passing before the WASM link and Playwright tests passing as the final step. `./build.sh --skip-rust-tests` and `./build.sh --skip-playwright-tests` must each warn and proceed without running the respective test step.

### Implementation for User Story 2

- [X] T007 [US2] Add `[build-dependencies]` section to `astro-wasm/Cargo.toml` with `cc = "1"` (FR-008)
- [X] T008 [US2] Rewrite `astro-wasm/build.rs` to branch on `std::env::var("TARGET").unwrap_or_default()`: **emscripten branch** (target string contains `"emscripten"`) — wrap the existing `cargo:rustc-link-search`, `cargo:rustc-link-lib=static=swe`, and all `cargo:rustc-link-arg` emissions inside this branch; **native branch** (else) — use `cc::Build::new().include("../vendor/swisseph").define("NOT_WINDOWS", None).opt_level(2).warnings(false)` and add exactly these seven `.file()` calls (matching the `SWE_SRCS` variable in `build.sh` Phase 2): `.file("../vendor/swisseph/swedate.c").file("../vendor/swisseph/swehouse.c").file("../vendor/swisseph/swejpl.c").file("../vendor/swisseph/swemmoon.c").file("../vendor/swisseph/swemplan.c").file("../vendor/swisseph/sweph.c").file("../vendor/swisseph/swephlib.c")`, then `.compile("swe")` — this auto-emits `cargo:rustc-link-lib=static=swe` and `cargo:rustc-link-search`; consult research.md RD-005 and RD-006 for `.compile()` semantics and `.warnings(false)` rationale (FR-008)
- [X] T009 [US2] Refactor `astro-wasm/src/lib.rs` in two parts: (a) **move** `unsafe { swe_set_ephe_path(b"/ephe\0".as_ptr() as *const c_char) }` out of `handle_sun_longitude` and into `bridge()` just before the `match op { ... }` dispatch — this makes all wrappers path-agnostic (FR-016); (b) **add** a `#[cfg(test)] mod tests` block containing: a `setup()` helper that calls `swe_set_ephe_path` with `concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe\0")` cast to `*const c_char`; `test_bridge_unknown_op` — allocates a stack output buffer, calls `bridge` with an unrecognised op string, and asserts the return value is `-1`; `test_sun_longitude_j2000` — calls `setup()`, then calls `handle_sun_longitude` with TJD `2451545.0` (J2000.0) and an output buffer, parses the returned JSON, and asserts the longitude field is within `0.5` of `280.37`; add at least one test per public function as required by FR-007 (FR-007, FR-016)
- [X] T010 [US2] Update `build.sh` with five targeted changes: (a) **Phase 0** — add flag parsing near the top: initialise `SKIP_RUST_TESTS=0` and `SKIP_PLAYWRIGHT_TESTS=0`, loop over `"$@"` to set each flag from `--skip-rust-tests` / `--skip-playwright-tests`; (b) **Phase 1** — add `command -v npm >/dev/null 2>&1 || { echo "ERROR: npm not found in PATH"; exit 1; }` to the existing prerequisite check block (FR-012); (c) **Phase 2.5** — after the Phase 2 `libswe.a` emcc block, insert the following two-part block: `if [ "$SKIP_RUST_TESTS" -eq 0 ]; then (cd astro-wasm && unset LIBSWE_DIR && cargo test) || exit 1; else echo "⚠ WARNING: --skip-rust-tests set — skipping cargo test"; fi; export LIBSWE_DIR="$(pwd)/lib"` — **critically**, `export LIBSWE_DIR` MUST be unconditional (outside the if/else) so it is set for the WASM build whether or not `cargo test` was skipped (FR-006, FR-015); (d) **Phase 4** — insert `[ -d node_modules ] || npm install` before the Vite build step (FR-012); (e) **Phase 6** (final line before `exit 0`) — insert: `if [ "$SKIP_PLAYWRIGHT_TESTS" -eq 0 ]; then npm test || exit 1; else echo "⚠ WARNING: --skip-playwright-tests set — skipping npm test"; fi` (FR-014, FR-015)

**Checkpoint**: `cargo test` passes natively unaided; `./build.sh` runs Rust tests before WASM compile and Playwright tests last; both skip flags suppress the respective step with a visible warning

---

## Phase 5: User Story 3 — Vite Builds and Minifies Web Assets (Priority: P3)

**Goal**: Replace `cp web/index.html web/style.css dist/` with `npm run build` (Vite), redirect the Emscripten build output from `dist/` to `public/`, and configure Vite to copy the WASM artifacts verbatim to `dist/` via its `publicDir` pass-through.

**Independent Test**: With `public/astro.{js,wasm,data}` already present (from a prior WASM build), run `npm run build`. Inspect `dist/index.html` — it must be smaller in bytes than `web/index.html`. Inspect `dist/style.css` — it must be minified (whitespace/comments stripped). Confirm `dist/astro.js` byte-count matches `public/astro.js` exactly (SC-003).

### Implementation for User Story 3

- [X] T011 [P] [US3] Create `vite.config.ts` at repo root: `import { defineConfig } from 'vite'; export default defineConfig({ root: 'web', publicDir: '../public', build: { outDir: '../dist', emptyOutDir: true, minify: true } })` — `minify: true` (Vite 5 production default) enables JS minification via esbuild; Vite automatically minifies HTML in production mode via its internal build pipeline — no separate `minifyHtml` option exists in `BuildOptions` and adding it would cause a TypeScript type error under the strict `tsconfig.json` from T004; CSS is minified by default via lightningcss; SC-003 (`dist/index.html` smaller than `web/index.html`) is satisfied by Vite's built-in HTML minification (FR-009, FR-010)
- [X] T012 [P] [US3] Update `astro-wasm/build.rs` emscripten branch: change the `cargo:rustc-link-arg=-o` emission from `dist/astro.js` to `public/astro.js`; if `dist/astro.data` (or similar) is separately referenced, update that path to `public/astro.data` as well (FR-010, FR-011)
- [X] T013 [US3] Update `build.sh`: (a) **add** `mkdir -p "$REPO_ROOT/public"` before the Phase 3 Emscripten block so the directory exists when Emscripten writes to it; the existing `mkdir -p "$REPO_ROOT/dist"` can be removed since Vite's `emptyOutDir: true` creates `dist/` in Phase 5 — only remove it after confirming no earlier phase writes directly to `dist/`; (b) remove the `cp web/index.html web/style.css dist/` line; (c) insert `npm run build` as the new Phase 5 step immediately after the `cargo build --target wasm32-unknown-emscripten` Phase 3 block and the Phase 4 `npm install` check, and before the Phase 6 `npm test` step added in T010 (FR-011)

**Checkpoint**: `./build.sh` end-to-end produces minified `dist/index.html`, minified `dist/style.css`, and verbatim `dist/astro.{js,wasm,data}` — all SC-001 through SC-005 success criteria are met

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: End-to-end validation of the complete pipeline.

- [X] T014 Run `npm install` then `./build.sh` end-to-end from a clean state; verify every item in the `quickstart.md` verification checklist: Rust tests print "test result: ok" before the WASM compile step; `dist/index.html` is smaller than `web/index.html`; `dist/astro.js` byte-count matches `public/astro.js`; Playwright tests report all passing; `build.sh` exits 0; `test-wasm.mjs` is absent; `npm test` run standalone also exits 0 (SC-001, SC-002, SC-003, SC-004, SC-005). Also perform the **payload audit** required by Constitution Dev Workflow §4: run `du -sh dist/` and compare combined `dist/` payload against the feature 001 baseline; record the result and confirm no regression before marking this feature ready for merge.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **US1 (Phase 3)**: Depends on Phase 1 completion only
- **US2 (Phase 4)**: Depends on US1 — `package.json` must exist for the `npm test` step wired into `build.sh` in T010
- **US3 (Phase 5)**: Depends on US1 — `package.json` must exist for `npm run build`; can proceed in parallel with US2 after US1 since they modify different files and different `build.sh` phases
- **Polish (Phase 6)**: All three user stories must be complete

### User Story Dependencies

| Story | Depends On | Can Parallelise With |
|-------|-----------|----------------------|
| US1 (P1) | Phase 1 | — |
| US2 (P2) | US1 (`package.json`) | US3 (different files / different build.sh phases) |
| US3 (P3) | US1 (`package.json`) | US2 (different files / different build.sh phases) |

### Within Each User Story

- **US1**: T003 and T004 are independent (different files) → parallel; T005 depends on T003; T006 depends on T003 and T005
- **US2**: T007 must precede T008 (Cargo.toml cc dep required before build.rs cc::Build code compiles); T008 and T009 can proceed in parallel after T007 (different files); T010 depends on T003 (package.json must be established)
- **US3**: T011 and T012 are independent (different files) → parallel; T013 depends on T011 (vite.config.ts must exist before `npm run build` is wired into build.sh)

---

## Parallel Execution Examples

### User Story 1

```bash
# T003 and T004 in parallel (different files, no dependencies):
# Terminal A: create package.json
# Terminal B: create tsconfig.json

# Then sequentially:
# T005: create playwright.config.ts
# T006: create tests/astro.spec.ts
```

### User Story 2

```bash
# T007 first: update astro-wasm/Cargo.toml

# Then T008 and T009 in parallel (different files):
# Terminal A: rewrite astro-wasm/build.rs (T008)
# Terminal B: refactor astro-wasm/src/lib.rs + add tests (T009)

# Then T010: update build.sh
```

### User Story 3

```bash
# T011 and T012 in parallel (different files):
# Terminal A: create vite.config.ts (T011)
# Terminal B: update build.rs -o path to public/ (T012)

# Then T013: update build.sh (replace cp with npm run build)
```

---

## Implementation Strategy

**MVP Scope (US1 only)**: Completing Phase 1 + Phase 3 (T001–T006) delivers a working Playwright test suite runnable with `npm test`. This eliminates `test-wasm.mjs` and establishes the regression guard. The remaining user stories improve the build pipeline but are not required for the core test suite to function.

**Recommended Delivery Order**:

1. Phase 1 (T001–T002) — no dependencies, 2 small changes
2. Phase 3 US1 (T003–T006) — npm scaffold + Playwright tests (~4 files)
3. Phase 4 US2 (T007–T010) — Rust tests + build.sh gates (highest complexity)
4. Phase 5 US3 (T011–T013) — Vite build pipeline (~3 file changes)
5. Phase 6 (T014) — end-to-end validation

**Highest Complexity Task**: T009 (`lib.rs` refactor + Rust unit tests) involves unsafe Rust FFI, `CARGO_MANIFEST_DIR` path construction, and Swiss Ephemeris wrapper invocation. Read data-model.md (lib.rs structural changes section) and research.md (RD-005 cc crate API, RD-007 LIBSWE_DIR isolation, RD-008 ephe path in tests) before starting.

---

## Summary

| Metric | Value |
|--------|-------|
| Total tasks | 14 |
| Phase 1 (Setup) | 2 tasks (T001–T002) |
| US1 (P1) tasks | 4 tasks (T003–T006) |
| US2 (P2) tasks | 4 tasks (T007–T010) |
| US3 (P3) tasks | 3 tasks (T011–T013) |
| Polish tasks | 1 task (T014) |
| Parallelisable tasks | T002, T003, T004, T008+T009 (after T007), T011, T012 |
| Format | All tasks: checkbox + sequential ID + [P] + [USn] + description with file path ✅ |
