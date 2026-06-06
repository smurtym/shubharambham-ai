# Data Model: Modular Architecture Layout

**Feature**: `003-arch-layout`  
**Date**: 2026-03-14

---

## Rust Types

### `SunRequest` (parsed from bridge JSON input)

```rust
// Deserialized in bridge.rs from the wasm-api-v2 JSON payload
struct SunRequest {
    operation: String,   // always "sun_longitude" for this feature
    datetime: String,    // ISO 8601 UTC string, e.g. "2000-01-01T12:00:00Z"
    lang: String,        // "en" | "te"  (unknown → fallback to "en")
}
```

### `SunResponse` (serialized to bridge JSON output — success)

```rust
// Serialized in bridge.rs after engine returns
struct SunResponse {
    label: String,       // localized planet name from localization::get_label()
    longitude: f64,      // ecliptic longitude in decimal degrees
}
```

### `ErrorResponse` (serialized to bridge JSON output — error)

```rust
// Serialized in bridge.rs on any error path
struct ErrorResponse {
    error: String,       // human-readable error message
}
```

*Note*: `SunResponse` and `ErrorResponse` are mutually exclusive. The JS caller checks for the `error` field first (clarification Q1, Option A).

### Localization Types

**Locale data files** (one per language, translator-editable):

```rust
// locales/en.rs  — values are the only thing translators change
pub const STRINGS: &[(&str, &str)] = &[
    ("planet.sun", "Sun"),
];
```

```rust
// locales/te.rs
pub const STRINGS: &[(&str, &str)] = &[
    ("planet.sun", "సూర్యుడు"),
];
```

**Lookup function** (logic-only file, not edited by translators):

```rust
// localization.rs
pub fn get_string(key: &str, lang: &str) -> &'static str {
    // returns localized value, falls back to English, then "[missing]"
}
```

**Adding a new language** (e.g., Hindi):
1. Create `src/locales/hi.rs` with the same `STRINGS` array pattern.
2. Add `pub mod hi;` to `src/locales/mod.rs`.
3. Add one `match` arm in `localization.rs`.
4. Zero changes to any engine, bridge, or utility code.

### `SunEngineResult` (internal, returned by engine)

```rust
// Returned by engines::sun::handle_sun_longitude() to bridge.rs
// Named SunEngineResult to distinguish from the JS-side SunResult response shape
pub struct SunEngineResult {
    pub label: &'static str,  // from localization::get_string("planet.sun", lang)
    pub longitude: f64,
}
```

---

## Module Dependency Graph

```
lib.rs
  │
  ├── bridge.rs              ← public WASM export (#[no_mangle])
  │     ├── uses utils::iso_to_jd()
  │     ├── uses engines::sun::handle_sun_longitude()
  │     └── serializes SunResponse / ErrorResponse
  │
  ├── engines/sun.rs         ← called by bridge only
  │     ├── uses swe_wrappers::calc_sun_longitude()
  │     ├── uses localization::get_string("planet.sun", lang)
  │     └── uses utils::normalize_degrees()  (if needed)
  │
  ├── swe_wrappers.rs        ← extern "C" only; called by engines only
  ├── utils.rs               ← pure Rust; callable by bridge and engines
  ├── localization.rs        ← lookup logic only; imports locales/*
  └── locales/
        ├── mod.rs           ← pub mod en; pub mod te;
        ├── en.rs            ← translator-editable data
        └── te.rs            ← translator-editable data
```

**Dependency rule enforced by design** (not compiler-enforced in this feature):
- `swe_wrappers` ← called only by `engines/*`
- `localization` ← called only by `engines/*`
- `utils` ← callable by `bridge` and `engines/*`
- `bridge` ← only module with `#[no_mangle]`

---

## JavaScript Types

### `SunRequest` (constructed in `data.js`)

```js
// getSunLongitude() builds this object, JSON.stringify'd before passing to bridge()
{
  operation: "sun_longitude",
  datetime: "<ISO 8601 string>",
  lang: "en" | "te"
}
```

### `SunResult` (returned to caller of `getSunLongitude`)

```js
// Parsed from bridge() return value
{
  label: string,       // e.g. "Sun" or "సూర్యుడు"
  longitude: number    // e.g. 280.46
}
// OR on error:
{
  error: string
}
```

---

## State Transitions (Web UI)

```
idle
  │  user submits form
  ▼
calculating  (submit button disabled)
  │  bridge() returns
  ├─ success ──► display label + longitude  →  idle (button re-enabled)
  └─ error   ──► display error message      →  idle (button re-enabled)
```
