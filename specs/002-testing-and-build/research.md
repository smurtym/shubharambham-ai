# Research: Testing and Build Tooling (Feature 002)

**Branch**: `002-testing-and-build` | **Date**: 2026-03-14  
**Purpose**: Resolve implementation unknowns before Phase 1 design.

---

## 1. Vite 5 `publicDir` Pass-Through Behavior

**Decision**: Use `publicDir: '../public'` (relative to `root: 'web'`); files are copied verbatim — no transformation, renaming, or hashing.

**Rationale**: Vite 5 docs state explicitly that `publicDir` files are "served or copied **as-is without transform**". The path is resolved relative to project `root`, so `../public` from `web/` resolves to `public/` at repo root. `build.outDir: '../dist'` is similarly relative to `root`.

**Alternatives considered**: Embedding WASM artifacts as Rollup assets (rejected — Rollup adds content hashes to all asset filenames, breaking Emscripten's runtime loading of `.wasm`/`.data` paired with `.js`).

---

## 2. HTML Minification in Vite 5

**Decision**: HTML whitespace/comment removal is controlled by `build.minifyHtml` (defaults to `true` in Vite 5). **Verify during implementation** — if SC-003 fails (index.html not smaller), set `build.minify: 'esbuild'` explicitly and confirm HTML plugin activation.

**Rationale**: Research produced conflicting signals. Vite 5 introduced `build.minifyHtml` defaulting to `true` via its internal HTML plugin. If the default is insufficient, `@rollup/plugin-html` or `vite-plugin-html` can be added as a dev dependency. This is a low-risk one-line fix during implementation and does not change any other plan decision.

**Alternatives considered**: `html-minifier-terser` as a separate build step (rejected — adds shell complexity; Vite plugin is the correct integration point).

---

## 3. `<script src="astro.js">` Bundling Risk

**Decision**: The bare `<script src="astro.js"></script>` tag in `web/index.html` is safe — Vite only processes `<script type="module" src>` for bundling. No `vite-ignore` attribute is needed.

**Rationale**: Vite's HTML plugin scans only module scripts (`type="module"`) for bundling. The existing inline `<script>` block (containing the bridge helper) has no `src` attribute and won't be processed. The `<script src="astro.js">` loads the Emscripten-generated file from `public/` which Vite copies verbatim — no circular bundling occurs.

**Risk**: If a future refactor converts the script to `type="module"`, Vite would attempt to resolve `astro.js` and fail (it's not in `web/` source). Add `vite-ignore` attribute at that point.

---

## 4. CSS Minification in Vite 5

**Decision**: CSS minification via lightningcss is enabled by default in Vite 5 (`build.cssMinify: 'lightningcss'` is the default for client builds).

**Rationale**: `web/style.css` is referenced via `<link rel="stylesheet">` in `index.html`; Vite processes it through lightningcss producing a minified `dist/style.css`. No configuration change required.

---

## 5. `cc` Crate — Native libswe Compilation in `build.rs`

**Decision**: Use `cc::Build::new().include(...).define(...).opt_level(2).file(...).warnings(false).compile("swe")`. This auto-emits `cargo:rustc-link-lib=static=swe` and `cargo:rustc-link-search=native=$OUT_DIR` — no manual `println!` needed for the native path.

**Rationale**: The `.compile()` call handles all Cargo integration automatically. `.warnings(false)` (emits `-w`) suppresses warnings from the third-party swisseph C code without affecting the Rust build. `cc = "1"` in `[build-dependencies]` is sufficient; no feature flags required.

**Emscripten safety**: The `cc` crate selects the compiler based on the `TARGET` triple, not `PATH`. When `TARGET` is `x86_64-unknown-linux-gnu` (or similar native), `cc` uses `gcc`/`clang` even if `emcc` is on `PATH` from a sourced Emscripten environment. No accidental `emcc` usage.

**Alternatives considered**: Pre-compiling `lib/libswe-native.a` in `build.sh` and pointing `LIBSWE_DIR` at it (rejected — couples `cargo test` to `build.sh`; the `cc` crate approach makes `cargo test` fully self-contained).

---

## 6. `build.rs` TARGET Detection

**Decision**: `std::env::var("TARGET").unwrap_or_default().contains("emscripten")` is the correct predicate. When `true`: emit Emscripten link args. When `false`: invoke `cc::Build`.

**Rationale**: Cargo guarantees `TARGET` is set to the full target triple at build-script runtime. The triple `wasm32-unknown-emscripten` contains "emscripten" as a substring. Native triples (`x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, etc.) never contain "emscripten". This check is robust for all current and foreseeable CI environments.

---

## 7. `LIBSWE_DIR` Isolation for `cargo test`

**Decision**: `build.sh` calls `unset LIBSWE_DIR` before `cargo test`, then `export LIBSWE_DIR="$LIB_DIR"` before `cargo build --target wasm32-unknown-emscripten`.

**Rationale**: On the native path, `build.rs` uses `cc::Build` and never reads `LIBSWE_DIR` — but unsetting it is a safety measure that prevents the env var from accidentally being picked up by any future `build.rs` change. The unset is a no-op in practice given the TARGET branch logic.

---

## 8. `swe_set_ephe_path` Caller Responsibility

**Decision**: `swe_set_ephe_path` is removed from all wrapper function bodies. The `bridge()` handler calls it once with `b"/ephe\0"` (Emscripten virtual FS) at the start of each invocation. Tests call it in a `#[cfg(test)]` shared setup with `concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe\0")`.

**Rationale**: Making wrappers path-agnostic eliminates the hardcoded Emscripten path from function bodies, making them testable natively without `#[cfg(not(test))]` guards. The test path uses `CARGO_MANIFEST_DIR` (always set by Cargo — no config or env needed).

**Note**: `CARGO_MANIFEST_DIR` resolves to `<repo>/astro-wasm/`. Therefore `concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe")` = `<repo>/ephe` at compile time. The `\0` null terminator is appended for FFI compatibility: `concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe\0")`.

---

## 9. Playwright `webServer` Configuration

**Decision**: Use a fixed port `4173` for `npx serve dist/ --listen 4173`. Set `reuseExistingServer: !process.env.CI`.

**Rationale**: The `serve` npm package does not support ephemeral port selection (`--port 0`). Port `4173` is the standard Vite preview port and unlikely to be in use during tests. `reuseExistingServer: !process.env.CI` follows the documented Playwright recommendation: reuse allows faster local dev iteration; fresh server on CI ensures determinism.

**Note**: This is a minor deviation from the spec wording ("ephemeral port chosen automatically") — the spec intent (no port conflicts, no manual management) is fully met. The port is hardcoded to a predictable dev-only value with no user configuration needed.

**Alternatives considered**: `vite preview` as the webServer command (rejected — would require Vite to be running in preview mode, coupling the test command to the Vite dev server; `serve` is a simpler static file server with no Vite dependency at test time).

---

## 10. `page.waitForEvent('console')` API

**Decision**: Use `page.waitForEvent('console', { predicate: (msg) => ..., timeout: 10000 })` registered before `page.goto()`.

**Rationale**: In Playwright ≥1.51, `waitForEvent` accepts either a predicate function or an options object `{ predicate, timeout }`. Registering before `goto()` ensures the handler captures the async `onRuntimeInitialized` console.log regardless of when it fires relative to page load. Timeout 10s gives WASM adequate time to initialize on CI machines.

---

## 11. TypeScript Configuration Scope

**Decision**: `tsconfig.json` at repo root with `include: ["vite.config.ts", "playwright.config.ts", "tests/**/*.ts"]`. No `strict` mode initially.

**Rationale**: Scoping TS to config/test files only ensures the TypeScript compiler never touches `web/` JavaScript or Rust/WASM build artifacts. The `tsconfig.json` is used by the TS language server in the IDE for config/test files only; `vite build` and `npx playwright test` each invoke their own TS transpilation.

---

## Summary of Key Decisions

| # | Topic | Decision |
|---|-------|----------|
| 1 | Vite publicDir | Verbatim copy confirmed; `../public` relative to `root: 'web'` |
| 2 | HTML minification | `build.minifyHtml: true` default; verify during implementation |
| 3 | `<script src>` safety | Bare non-module script — Vite will not bundle it |
| 4 | CSS minification | lightningcss default in Vite 5; no config needed |
| 5 | cc crate API | `.compile("swe")` auto-emits Cargo link metadata |
| 6 | TARGET detection | `.contains("emscripten")` is robust and idiomatic |
| 7 | LIBSWE_DIR isolation | `unset LIBSWE_DIR` before `cargo test` as safety measure |
| 8 | ephe path | Callers set path; tests use `concat!(env!("CARGO_MANIFEST_DIR"), "/../ephe\0")` |
| 9 | Playwright webServer | Fixed port 4173; `reuseExistingServer: !process.env.CI` |
| 10 | console capture | `waitForEvent('console', { predicate, timeout: 10000 })` |
| 11 | TypeScript scope | Root `tsconfig.json` scoped to config/test files only |
