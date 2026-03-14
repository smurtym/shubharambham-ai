# Quickstart: Testing and Build Tooling (Feature 002)

**Branch**: `002-testing-and-build`

---

## Prerequisites

Before running any build or test command, ensure the following are installed and available on `PATH`:

| Tool | Version | Check |
|------|---------|-------|
| Rust (stable toolchain) | ≥ 1.75 | `rustc --version` |
| Emscripten (`emcc`) | ≥ 4.0.0 | `emcc --version` (source `$EMSDK/emsdk_env.sh` first) |
| Node.js | ≥ 18 | `node --version` |
| npm | ≥ 9 | `npm --version` |
| `rustup` target | `wasm32-unknown-emscripten` | `rustup target list --installed` |

Ensure the git submodule is populated:
```bash
git submodule update --init
```

Ensure ephemeris data files are present:
```bash
ls ephe/semo_18.se1 ephe/sepl_18.se1
```

---

## One-Time Setup (after fresh clone)

```bash
# Source Emscripten environment
source $EMSDK/emsdk_env.sh

# Install npm dependencies + Playwright Chromium binaries (postinstall runs automatically)
npm install
```

`npm install` runs `postinstall`, which calls `npx playwright install chromium`. This downloads the Chromium binary used for E2E tests. Internet access is required.

---

## Full Build + Test

```bash
source $EMSDK/emsdk_env.sh
./build.sh
```

This runs all phases in order:
1. Prerequisite checks
2. Compile Swiss Ephemeris → `lib/libswe.a`
3. `cargo test` (native — compiles swisseph natively via `cc` crate, runs Rust unit tests)
4. `cargo build --target wasm32-unknown-emscripten --release` → `public/astro.{js,wasm,data}`
5. `npm run build` (Vite — minifies `web/` → `dist/`, copies `public/` verbatim)
6. `npm test` (Playwright — serves `dist/` on port 4173, runs headless Chromium E2E tests)

Expected output:
```
[build.sh] ✓ Rust tests passed
[build.sh] ✓ WASM artifacts written to public/
[build.sh] ✓ Web assets built to dist/
[build.sh] ✓ Playwright tests passed
[build.sh] Build complete.
```

---

## Individual Commands

### Rust Unit Tests Only

```bash
cargo test --manifest-path astro-wasm/Cargo.toml
```

> No Emscripten environment needed. The `cc` crate compiles swisseph natively.
> Ephemeris path is hardcoded in tests via `CARGO_MANIFEST_DIR`.

### Front-End Build Only

```bash
npm run build
```

> Requires `public/astro.{js,wasm,data}` to already exist (run `./build.sh` first, or at least through Phase 4).

### E2E Tests Only

```bash
npm test
```

> Requires `dist/astro.js` to exist. If absent, the test will fail immediately with:
> `dist/astro.js not found — run ./build.sh first`

### Build Without Tests (Emergency)

```bash
./build.sh --skip-rust-tests --skip-playwright-tests
```

> Both test phases are skipped. A warning is printed for each skipped step.
> Intended for emergency quick-fix builds only — never use as the default invocation.

---

## Output Directories

| Directory | Contents | Committed? |
|-----------|----------|------------|
| `lib/` | `libswe.a` (WASM-compiled Swiss Ephemeris) | No |
| `public/` | `astro.js`, `astro.wasm`, `astro.data` (Emscripten WASM artifacts) | No |
| `dist/` | `index.html`, `style.css` (minified), `astro.*` (verbatim from `public/`) | No |
| `playwright-report/` | HTML test report | No |
| `test-results/` | Playwright trace artifacts | No |

---

## Verification Checklist

After a successful `./build.sh`:

- [ ] `dist/index.html` exists and is smaller than `web/index.html`
- [ ] `dist/style.css` is minified (no whitespace, no comments)
- [ ] `dist/astro.js` byte count equals `public/astro.js` byte count (`diff dist/astro.js public/astro.js`)
- [ ] `test-wasm.mjs` does NOT exist in the repo root
- [ ] `npm test` runs and all tests pass (if Playwright binaries installed)
- [ ] `cargo test` runs and all tests pass (no Emscripten needed)
