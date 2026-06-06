# Research: Modular Architecture Layout

**Feature**: `003-arch-layout`  
**Date**: 2026-03-14  
**Status**: Complete — all NEEDS CLARIFICATION resolved

---

## 1. ISO 8601 UTC → Julian Day Number (pure Rust)

**Decision**: Implement using the standard proleptic Gregorian calendar arithmetic described in Jean Meeus, *Astronomical Algorithms* (2nd ed., 1998), Chapter 7.

**Algorithm** (integer arithmetic variant, correct for all dates after 1582-10-15):

```
Given: year Y, month M, day D, hour H, minute Mi, second S (all from parsed ISO string)

If M <= 2: Y -= 1; M += 12
A = floor(Y / 100)
B = 2 - A + floor(A / 4)
JD = floor(365.25 * (Y + 4716))
   + floor(30.6001 * (M + 1))
   + D + B - 1524.5
   + (H + Mi/60.0 + S/3600.0) / 24.0
```

**Reference value**: J2000.0 = 2000-01-01T12:00:00Z → JD 2451545.0 (exactly). This will be the unit test reference value in `utils.rs`.

**Rationale**: Pure Rust arithmetic with no dependencies. Does not call `swe_utc_to_jd`, keeping `swe_wrappers` limited to FFI only (FR-002). Result matches Swiss Ephemeris own JD to full double precision for UTC dates.

**Alternatives considered**:
- Call `swe_utc_to_jd` via `swe_wrappers`: rejected — violates FR-004 (pure Rust) and adds an FFI call for what is standard arithmetic.
- Use a Rust crate (e.g., `chrono`): rejected — adds a dependency solely for calendar arithmetic; the one-function formula is smaller and keeps WASM binary lean.

---

## 2. Rust Module Layout for a `cdylib` Crate

**Decision**: Use Rust's standard file-per-module layout with `lib.rs` as the crate root. Each top-level module is a `.rs` file; the `engines` sub-module uses a directory with `mod.rs`.

**Pattern**:
```
src/
├── lib.rs              ← declares: mod swe_wrappers; mod utils; mod localization; mod bridge; mod engines;
│                         re-exports: pub use bridge::bridge;
├── swe_wrappers.rs     ← extern "C" block only
├── utils.rs            ← iso_to_jd(), normalize_degrees()
├── localization.rs     ← get_label()
├── bridge.rs           ← bridge() fn, JSON parse/dispatch/serialize
└── engines/
    ├── mod.rs          ← pub mod sun;
    └── sun.rs          ← handle_sun_longitude()
```

**Key constraint**: The `#[no_mangle] pub extern "C" fn bridge(...)` MUST remain in `bridge.rs` (or re-exported from `lib.rs` with `#[no_mangle]`). The `#[no_mangle]` attribute applies to the function's final exported symbol — re-export via `pub use` does **not** propagate `#[no_mangle]`, so the attribute must sit directly on the function definition in `bridge.rs`.

**Rationale**: File-per-module is the idiomatic Rust pattern. No macros, no proc-macros, no additional build complexity. The `cdylib` crate type and Emscripten toolchain are unaffected by internal module structure.

**Alternatives considered**:
- Inline modules (`mod swe_wrappers { ... }`) in `lib.rs`: rejected — defeats the purpose of the refactor; modules remain invisible as separate units.
- Separate Rust crates per layer: rejected — over-engineering for this scale; single `cdylib` crate with internal modules achieves the same isolation.

---

## 3. Localization Pattern — Separate Data Files per Locale

**Decision**: Split locale *data* from lookup *logic*. Each locale lives in its own Rust file under `src/locales/`. The `localization.rs` module contains only lookup logic and imports the locale modules. Translators edit only the data files — no logic code involved.

**Directory layout**:
```
astro-wasm/src/
├── localization.rs          ← lookup logic only; no strings here
└── locales/
    ├── mod.rs               ← declares: pub mod en; pub mod te;
    ├── en.rs                ← English strings (editable by translators)
    └── te.rs                ← Telugu strings (editable by translators)
```

**Locale data file format** (e.g., `locales/en.rs`):
```rust
// locales/en.rs
// Translator-editable: add or change values; do NOT change the key strings.
// Key format: "<category>.<item>"
pub const STRINGS: &[(&str, &str)] = &[
    ("planet.sun", "Sun"),
];
```

**Lookup logic** (`localization.rs`):
```rust
use crate::locales;

/// Returns the localized string for `key` in `lang`.
/// Falls back to English if the key is missing in the requested locale.
/// Returns "[missing:<key>]" if the key is absent from English too.
pub fn get_string(key: &str, lang: &str) -> &'static str {
    let table: &[(&str, &str)] = match lang {
        "te" => locales::te::STRINGS,
        _    => locales::en::STRINGS,   // unknown lang → English
    };
    if let Some(&(_, v)) = table.iter().find(|(k, _)| *k == key) {
        return v;
    }
    // Key missing in requested locale — fall back to English
    if let Some(&(_, v)) = locales::en::STRINGS.iter().find(|(k, _)| *k == key) {
        return v;
    }
    "[missing]"   // never a panic; edge case handled (spec Edge Cases)
}
```

**Adding a new language** (e.g., Hindi):
1. Create `src/locales/hi.rs` with the same `STRINGS` array.
2. Add `pub mod hi;` to `src/locales/mod.rs`.
3. Add `"hi" => locales::hi::STRINGS,` to the `match` in `localization.rs`.
4. No other files change.

**Key constraints**:
- Strings are `&'static str` — compiled into the WASM binary; no runtime file fetch (Constitution Principle VI).
- The key namespace (`"planet.sun"`) is the contract between engine code and locale files; keys are stable identifiers.
- Locale files contain *only* a `const STRINGS` array — a translator can edit the right-hand value strings without knowing Rust syntax beyond strings.

**Rationale**: Data/logic separation makes locale files genuinely maintainable by non-programmers. Adding a new language requires one new file + two one-line additions — no changes to engine or bridge code. Linear array lookup is acceptable for the small number of strings per locale in this project; a `HashMap` would add allocation overhead for no practical benefit at this scale.

**Alternatives considered**:
- Hardcoded `match (planet, lang)` in `localization.rs`: rejected — mixes data and logic in one place; a translator must understand Rust match syntax and cannot add a language without touching logic code.
- External JSON/TOML locale files fetched at runtime: rejected — explicitly forbidden by Constitution Principle VI; adds network dependency.
- `phf` (perfect hash function crate): rejected — adds a proc-macro dependency for a lookup table of <20 entries; over-engineering for current scale.

---

## 4. JS Module Strategy for `astro-glue.js` / `data.js` / `components.js`

**Decision**: Use standard `<script src="...">` tags with ES module `import`/`export` syntax where supported, but keep it compatible with plain `<script>` ordering since the existing Emscripten-generated `astro.js` is not an ES module.

**Pattern**: `astro-glue.js` declares `bridge()` as a plain function on the global scope (same as current inline implementation). `data.js` and `components.js` are loaded after `astro-glue.js` and can reference `bridge` directly. No module bundler needed.

**Rationale**: Emscripten's `astro.js` uses the global `Module` object. Wrapping in ES modules would require setting `EXPORT_ES6=1` in the Emscripten build flags, which changes the build pipeline. The current approach (global scope, load-order dependency) is the smallest change from the existing working code.

**Alternatives considered**:
- ES modules with `import`: rejected for this feature — requires Emscripten `EXPORT_ES6` flag change and `<script type="module">` on `index.html`; constitutes a build pipeline change out of scope here.
- Bundled (Vite): rejected — Vite is used for testing/minification only; the source files must remain plain JS per Constitution Principle II.

---

## 5. Playwright Test Update Strategy

**Decision**: The existing `tests/astro.spec.ts` tests will be replaced (not amended alongside). The new test:
1. Navigates to `index.html`
2. Fills the datetime input with `2000-01-01T12:00:00Z`
3. Selects `te` from the language selector
4. Clicks submit
5. Waits for submit button to become re-enabled (per clarification Q2)
6. Asserts result element text contains `సూర్యుడు` and a numeric value matching `/\d+\.\d+/`
7. Repeats steps 2–6 with `en` and asserts `Sun`

**Latency timeout**: 500 ms for the button re-enable wait (per SC-006 / clarification Q4).

**Rationale**: The old test asserted a console log value (`Sun longitude: 280.46`). The new UI has a DOM result element, making assertions more reliable and independent of console behaviour.

---

## Summary Table

| Unknown | Decision | Rationale |
|---------|----------|-----------|
| ISO→JD algorithm | Meeus proleptic Gregorian, pure Rust | No deps; matches SWE precision; consistent with FR-004 |
| `cdylib` module layout | File-per-module; `#[no_mangle]` stays on fn definition in `bridge.rs` | Idiomatic; no build change; compiler-enforced isolation |
| Localization pattern | Separate locale data files (`locales/en.rs`, `locales/te.rs`) + lookup logic in `localization.rs`; translators edit data files only | Data/logic separation; translator-editable; scales to new languages with one new file + two lines |
| JS file split strategy | Plain `<script>` with global scope; no ES modules | Avoids Emscripten build flag change; smallest delta |
| Playwright test update | Replace old console assertion with DOM result + button re-enable wait; 500 ms timeout | DOM assertions more reliable; consistent with new contract |
