# Feature Specification: Infrastructure and Tooling Setup

**Feature Branch**: `001-infrastructure-tooling-setup`  
**Created**: 2026-03-14  
**Status**: Draft  
**Input**: User description: "Our first feature does not ship value to end user. It just helps us to build the infrastructure and tooling. After building this feature, I should see: 1. a web page that says 'Work in progress', 2. Successful compilation of swisseph library, 3. Creating a simple wrapper function in rust to find ephemeris of sun for a sample date and show on console."

## Overview

This is the foundational infrastructure feature. It establishes the full build pipeline
needed for all subsequent features: a working Emscripten/WASM build, the Swiss Ephemeris C
library compiled and embedded in the WASM virtual filesystem, a minimal Rust FFI wrapper
proven to call into the library, and a placeholder webpage confirming the delivery pipeline
is end-to-end functional.

No end-user product value is delivered. All three outcomes are developer-facing verification
milestones.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Static Placeholder Webpage Served (Priority: P1)

A developer opens the project's root URL in a mobile browser and sees a webpage
that displays the text "Work in Progress". This confirms the static file hosting
pipeline (HTML + any WASM glue JS) is wired end-to-end.

**Why this priority**: This is the simplest delivery checkpoint. It validates that
the HTML/JS layer can be built, served, and loaded before any WASM is involved.
It also establishes the minimum shell into which all future features will be placed.

**Independent Test**: Open `index.html` in a browser (or via `python3 -m http.server`).
The page renders and displays "Work in Progress" without JavaScript errors in the
browser console. No external network requests are made.

**Acceptance Scenarios**:

1. **Given** the repository is freshly cloned, **When** the developer opens
   `index.html` in Chrome/Firefox/Safari on mobile or desktop, **Then** the page
   displays "Work in Progress" as visible text with no broken-resource errors and
   no JavaScript console errors.
2. **Given** the page is loaded on a slow 2G connection simulation, **When** the
   page fully loads, **Then** the initial payload (HTML + CSS + any WASM glue) is
   under 500 KB total (uncompressed).

---

### User Story 2 - Swiss Ephemeris Library Compiles Successfully (Priority: P2)

A developer runs the build command and observes that the Swiss Ephemeris C library
(`swisseph`) compiles without errors using the Emscripten toolchain, and the
required ephemeris data files are embedded in the WASM virtual filesystem.

**Why this priority**: The ephemeris compilation is the most uncertain and risky
part of the toolchain. Validating it early de-risks all future calculation features.
It depends on the placeholder page being deliverable (P1) to have a host context.

**Independent Test**: Running the build script produces a `.wasm` and accompanying
JS module file with zero compiler errors or warnings treated as errors. The build
log confirms ephemeris data files are present in the Emscripten virtual filesystem
preload list.

**Acceptance Scenarios**:

1. **Given** Emscripten (`emcc`) is installed and available on PATH, **When** the
   developer runs the build command, **Then** the Swiss Ephemeris C sources compile
   to a WASM module without errors.
2. **Given** a successful build, **When** the WASM module is loaded in a browser
   environment, **Then** the ephemeris data files are accessible at their expected
   virtual paths inside the WASM module (verified by `FS.analyzePath()` or
   equivalent in the Emscripten runtime).

---

### User Story 3 - Rust Sun Ephemeris Wrapper Executes (Priority: P3)

A developer runs the build and opens the browser developer console. They see a
single line of output showing the Sun's ecliptic longitude for a hard-coded sample
date (e.g., 2000-01-01 12:00 UTC), proving that Rust code compiled to WASM can
call into the Swiss Ephemeris C library via FFI and return a result.

**Why this priority**: This is the full end-to-end pipeline proof: Rust → WASM
→ Emscripten → swisseph C library → result visible in browser console. Depends on
P1 (webpage) and P2 (swisseph compiled).

**Independent Test**: Open the page in a browser. The browser console shows one
log line of the form `Sun longitude (J2000): <degrees>°` where `<degrees>` is a
number in the range 0.0–360.0. No runtime exceptions are thrown.

**Acceptance Scenarios**:

1. **Given** a successful build, **When** the page loads in a browser, **Then**
   the browser console displays the Sun's ecliptic longitude for the hard-coded
   date in degrees, with at least 4 decimal places of precision.
2. **Given** the console output, **When** compared against a reference value from
   an independent ephemeris tool for J2000 (2000-01-01 12:00 UTC), **Then** the
   printed value is within 0.001° of the reference.

---

### Edge Cases

- What happens when the browser does not support WebAssembly? The page still
  displays "Work in Progress" but shows a human-readable error in the console
  (not a cryptic WASM instantiation failure).
- What happens when the ephemeris data files are missing from the build? The build
  script fails with a clear error message before producing a WASM artifact rather
  than producing a silently broken module.
- What happens when the Emscripten toolchain version is incompatible? The build
  script checks and prints the required Emscripten version and exits with a
  non-zero status code if it is not met.

## Clarifications

### Session 2026-03-14

- Q: How does Rust call into Swiss Ephemeris C code inside the WASM build? → A: Rust uses `wasm32-unknown-emscripten` compile target; `emcc` acts as the linker; Rust FFI calls C directly as part of the same Emscripten link step, producing a single WASM module.
- Q: How does Swiss Ephemeris C source code enter the repository? → A: Git submodule pointing to `https://github.com/aloistr/swisseph` at a pinned commit/tag; initialized with `git submodule update --init`.
- Q: Where do ephemeris data files come from and which files are required? → A: Download `semo_18.se1` and `sepl_18.se1` only from Swiss Ephemeris FTP into a local untracked `ephe/` directory. `seas_18.se1` (asteroid data) is not required for Vedic astrology calculations.
- Q: What build coordinator is used to invoke the multi-step Emscripten + Cargo + asset pipeline? → A: `build.sh` shell script; it handles `emcc`, `cargo`, and any future HTML/JS minification steps in one invocable command.
- Q: How does `index.html` load and invoke the WASM module? → A: Emscripten emits a separate `.js` glue file alongside the `.wasm` binary; `index.html` loads it with `<script src="...">` and calls the exported function inside the module's `onRuntimeInitialized` callback. No base64 inlining.
- Q: How should Rust functions be exported to JS — one per function or a generic bridge? → A: Single `bridge(op_ptr, input_ptr, output_ptr, output_max_len) → i32` function. JS owns and frees all three buffers. Rust only reads inputs and writes to the output buffer. `-sEXPORTED_FUNCTIONS=_bridge` is set once and never changed. New operations are pure Rust additions.
- Q: How large should the JS output buffer be, and what if it is too small? → A: Option B retry pattern. JS starts with 4 KB (`INITIAL_OUTPUT_SIZE = 4096`). If `bridge` returns a value greater than `output_max_len`, JS frees the buffer, re-allocates to exactly that size, and retries. Rust must NOT write to the buffer when returning a "too small" indicator. Rationale: minimises transient memory on low-RAM (512 MB) devices; 4 KB covers all expected Vedic calculation payloads on first try.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The build system MUST compile the Swiss Ephemeris C library using
  the Emscripten toolchain and produce a WASM module.
- **FR-002**: The build MUST embed exactly two Swiss Ephemeris data files —
  `semo_18.se1` (Moon) and `sepl_18.se1` (planets) — into the WASM virtual
  filesystem so they are available at runtime without a network request.
  `seas_18.se1` (asteroid data) MUST NOT be included; it is not required for
  Vedic astrology calculations.
- **FR-003**: The Rust WASM crate MUST implement a single public exported function
  `bridge(op_ptr, input_ptr, output_ptr, output_max_len) → i32` (see
  `contracts/wasm-api-v1.md`) that dispatches named operations. The
  `sun_longitude` operation MUST be handled as an internal route: it parses a
  Julian Day Number in UTC from a JSON input payload, calls
  the Swiss Ephemeris C library via FFI (`swe_calc_ut` with `SEFLG_SWIEPH`) to
  calculate the Sun's ecliptic longitude, and writes a JSON result into the
  caller-owned output buffer. The Rust crate MUST be compiled with the
  `wasm32-unknown-emscripten` target, with `emcc` acting as the linker, so that
  Rust and C are linked into a single WASM module in one Emscripten link step.
  The `sun_longitude` handler is a pipeline-verification proof-of-concept only;
  it will not be part of production implementation.
- **FR-004**: The Rust WASM module MUST expose exactly one JS-callable export:
  `_bridge`. It MUST be the only entry in `-sEXPORTED_FUNCTIONS`. No per-operation
  function is exported; all routing occurs inside Rust. `_malloc` and `_free` are
  available via `-sALLOW_MEMORY_GROWTH=1` without additional
  `EXPORTED_RUNTIME_METHODS` entries.
- **FR-005**: The main `index.html` MUST load the WASM module by referencing the
  Emscripten-generated `.js` glue file via a `<script src="astro.js">` tag
  declared after a `var Module = { onRuntimeInitialized: function() { … } }`
  block. Inside `onRuntimeInitialized`, the JS MUST call
  `bridge('sun_longitude', JSON.stringify({ tjd: 2451545.0 }))` using the
  `bridge()` JS helper (Option B retry pattern; see `contracts/wasm-api-v1.md`)
  and print the returned longitude to the browser console via `console.log`.
  The `.wasm` binary MUST remain a separate file; WASM MUST NOT be inlined as
  a base64 data URI.
- **FR-006**: The main `index.html` MUST display the text "Work in Progress"
  visibly, regardless of whether the WASM module loads successfully.
- **FR-007**: The build MUST be invocable via `./build.sh` at the repository root.
  The script MUST orchestrate, in order: dependency checks (Emscripten, ephemeris
  files, submodule), Swiss Ephemeris C compilation via `emcc`, Rust crate
  compilation via `cargo` with the `wasm32-unknown-emscripten` target, and
  copying build outputs to the web-serving directory. It MUST be extensible to
  support HTML/JS minification steps in future without restructuring.
- **FR-008**: If WebAssembly is not supported by the browser, the JS glue code
  MUST print a clear fallback message to the console rather than throwing an
  unhandled exception.

### Assumptions

- Emscripten (`emcc`) is pre-installed in the development environment.
- The Rust toolchain MUST support the `wasm32-unknown-emscripten` target
  (`rustup target add wasm32-unknown-emscripten`). `wasm-bindgen` and the
  `wasm32-unknown-unknown` target are NOT used in this project.
- The Swiss Ephemeris source code is managed as a Git submodule pointing to
  `https://github.com/aloistr/swisseph`, pinned to a specific commit or release
  tag. Developers initialize it with `git submodule update --init`. The submodule
  MUST be initialized before the build script is run; the build script MUST check
  for its presence and exit with a clear error if absent.
- Exactly two ephemeris data files are required: `semo_18.se1` and `sepl_18.se1`.
  They MUST be placed in an `ephe/` directory at the repository root before
  building. This directory is listed in `.gitignore` and is never committed.
  The build script MUST document the download URL (Swiss Ephemeris FTP:
  `https://www.astro.com/ftp/swisseph/ephe/`) and exit with a clear error if
  either file is absent. `seas_18.se1` is explicitly excluded.
- Target browsers are modern enough to support WASM (Chrome 57+, Firefox 53+,
  Safari 11+); no polyfill is required.
- Ayanamsha for this initial wrapper call is not required; the raw ecliptic
  longitude (tropical) is acceptable for the proof-of-concept console output.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can clone the repository, follow the README setup steps,
  run a single build command, open `index.html`, and see "Work in Progress"
  displayed — all within 30 minutes on a machine with the toolchain pre-installed.
- **SC-002**: The build produces zero compiler errors. Compiler warnings are
  permitted but MUST be listed in the build output.
- **SC-003**: The browser console shows a Sun ecliptic longitude value for J2000
  that is within 0.001° of the reference value from an independent source.
- **SC-004**: The total initial page payload (HTML + CSS + JS glue + WASM
  binary) is under 5 MB uncompressed. (Ephemeris data files embedded in the WASM
  virtual filesystem are counted separately and documented.)
- **SC-005**: The build script exits with a non-zero status code if any required
  dependency (Emscripten, ephemeris data files) is absent, and prints a
  human-readable error explaining what is missing.
