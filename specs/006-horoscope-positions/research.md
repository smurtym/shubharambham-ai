# Research: Horoscope Positions

**Branch**: `006-horoscope-positions` | **Date**: 2026-03-15

All NEEDS CLARIFICATION items from the Technical Context are resolved below.

---

## R-001: SE_SIDM_TRUE_CITRA constant value

**Decision**: Use `SE_SIDM_TRUE_CITRA = 27`  
**Rationale**: Confirmed directly from `vendor/swisseph/swephexp.h` line 265. This is the Swiss Ephemeris "True Chitrapaksha" ayanamsa — Spica/Chitra fixed at exactly 0° Libra using the star's true position, not a mean value. No alternative needed.  
**Alternatives considered**: SE_SIDM_LAHIRI (1) — explicitly excluded by spec; SE_SIDM_USER (255) — unnecessary complexity.  
**Usage**:
```rust
// In swe_wrappers.rs
pub const SE_SIDM_TRUE_CITRA: i32 = 27;
// Before each calculation session:
unsafe { swe_set_sid_mode(SE_SIDM_TRUE_CITRA, 0.0, 0.0) };
```

---

## R-002: Swiss Ephemeris body constants

**Decision**: Use SE constants directly from `vendor/swisseph/swephexp.h`.  
**Rationale**: All constants confirmed present in vendored header. Map to a `SweBody` enum in `swe_wrappers.rs`.

| Body | Constant | Value |
|------|----------|-------|
| Sun | SE_SUN | 0 |
| Moon | SE_MOON | 1 |
| Mercury | SE_MERCURY | 2 |
| Venus | SE_VENUS | 3 |
| Mars | SE_MARS | 4 |
| Jupiter | SE_JUPITER | 5 |
| Saturn | SE_SATURN | 6 |
| Rahu (True Node) | SE_TRUE_NODE | 11 |
| Ketu | derived: Rahu + 180° | — |

`SEFLG_SIDEREAL = 65536` (64×1024, line 210) — added to iflag for sidereal output.

---

## R-003: Ascendant via swe_houses_ex

**Decision**: Use `swe_houses_ex(tjd_ut, iflag, geolat, geolon, hsys='W', cusps[13], ascmc[10])`.  
**Rationale**: Confirmed signature from `swephexp.h` lines 816–818. `ascmc[0]` = Ascendant tropical longitude (`SE_ASC = 0`, line 164). `SE_NASCMC = 8` (array size). Passing `SEFLG_SIDEREAL` in iflag causes the function to apply the ayanamsa to house cusps and `ascmc[0]`, yielding the sidereal Ascendant directly.  
**Alternatives considered**: `swe_houses` (no iflag, tropical only) — cannot produce sidereal Ascendant without manual subtraction. Using `swe_houses_ex` with `SEFLG_SIDEREAL` is cleaner and consistent with how planets are computed.

```rust
// In swe_wrappers.rs
extern "C" {
    pub fn swe_houses_ex(
        tjd_ut: f64, iflag: i32, geolat: f64, geolon: f64,
        hsys: i32, cusps: *mut f64, ascmc: *mut f64,
    ) -> i32;
}
// hsys = b'W' as i32  (Whole Sign)
// ascmc[0] = sidereal Ascendant degree (after ayanamsa subtraction)
```

---

## R-004: Local time → Julian Day via chrono-tz

**Decision**: Add `chrono` + `chrono-tz` as `[dependencies]`. Implement `local_to_jd(local_time: &str, iana_tz: &str) -> Result<f64, String>` in `utils.rs`.  
**Rationale**: `chrono-tz` embeds the complete IANA timezone database as compiled Rust code — no filesystem access, no syscalls. Works correctly on wasm32-unknown-emscripten because it is purely in-memory. DST gap/overlap handling is built into `chrono-tz`'s `from_local_datetime()` returning `LocalResult::Ambiguous` or `LocalResult::None`.  
**Alternatives considered**:  
- Caller-supplied UTC offset — rejected (spec Q1 answer: use chrono-tz).  
- Hardcoded offset table — rejected (does not handle DST; India only).  
- `tzdb` / `tz-rs` — unnecessary; chrono-tz is the standard Rust ecosystem choice.  
**WASM binary size impact**: chrono-tz tz database embeds ~500 KB of string tables; with LTO and wasm-opt (already used in build), the net WASM size increase is typically ~150–200 KB compressed — acceptable per constitution (no strict binary size limit, only "reasonably small").

```rust
// In utils.rs
use chrono::{NaiveDateTime, TimeZone, LocalResult};
use chrono_tz::Tz;

pub fn local_to_jd(local_time: &str, iana_tz: &str) -> Result<f64, String> {
    let tz: Tz = iana_tz.parse().map_err(|_| format!("unknown timezone: {iana_tz}"))?;
    let ndt = NaiveDateTime::parse_from_str(local_time, "%Y-%m-%dT%H:%M:%S")
        .map_err(|e| format!("invalid localTime '{local_time}': {e}"))?;
    let dt = match tz.from_local_datetime(&ndt) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(dt, _) => dt,   // DST fall-back: use earlier offset
        LocalResult::None => {                   // DST spring-forward gap: advance
            let advanced = ndt + chrono::Duration::hours(1);
            tz.from_local_datetime(&advanced)
                .earliest()
                .ok_or_else(|| format!("localTime '{local_time}' is invalid in {iana_tz}"))?
        }
    };
    let unix_secs = dt.timestamp() as f64;
    // JD = unix_epoch_jd + unix_secs / 86400
    Ok(2_440_587.5 + unix_secs / 86_400.0)
}
```

---

## R-005: Position decomposition arithmetic

**Decision**: All intermediate floats kept as f64; truncation (not rounding) applied only to final integer output fields per spec.  
**Rationale**: Constitution V prohibits rounding of intermediate values without justification.

```
longitude ∈ [0, 360)       (normalised sidereal)

sign_idx       = (longitude / 30.0).floor() as u8          // 0..11
sign_number    = sign_idx + 1                               // 1..12
deg_in_sign    = (longitude % 30.0).floor() as u8          // 0..29
remainder_deg  = longitude % 30.0 - deg_in_sign as f64
minutes        = (remainder_deg * 60.0).floor() as u8      // 0..59
remainder_min  = remainder_deg * 60.0 - minutes as f64
seconds        = (remainder_min * 60.0).floor() as u8      // 0..59

nak_idx        = (longitude * 27.0 / 360.0).floor() as u8  // 0..26
nakshatra      = nak_idx + 1                                // 1..27
nak_part       = (longitude % (360.0/27.0)) / (360.0/108.0)
pada           = nak_part.floor() as u8 + 1                // 1..4
```

---

## R-006: Navamsa (D9) algorithm

**Decision**: Standard Vedic D9 via Chara/Sthira/Ubhaya sign classification.  
**Rationale**: Confirmed by spec FR-015 and Assumptions.

```
part_in_sign     = (longitude % 30.0) / (30.0 / 9.0)   // 0..9, float
part_index       = part_in_sign.floor() as u8            // 0..8

navamsa_start (0-based):
  Chara (sign_idx % 3 == 0, i.e. 0,3,6,9):   start = 0   (Aries)
  Sthira (sign_idx % 3 == 1, i.e. 1,4,7,10): start = 9   (Capricorn)
  Ubhaya (sign_idx % 3 == 2, i.e. 2,5,8,11): start = 3   (Cancer)

navamsa_idx      = (navamsa_start + part_index as usize) % 12
navamsa_number   = navamsa_idx + 1                        // 1..12
```

---

## R-007: Locale key plan — 49 new keys per language

**Decision**: Use dot-namespaced keys consistent with existing `"planet.sun"` pattern.

| Namespace | Count | Example key |
|-----------|-------|-------------|
| `planet.<canonical>` | 10 | `planet.Sun`, `planet.Ascendant` |
| `planet.abbrev.<canonical>` | 10 | `planet.abbrev.Sun` → "Su" |
| `sign.<canonical>` | 12 | `sign.Aries`, `sign.Capricorn` |
| `sign.abbrev.<canonical>` | 12 | `sign.abbrev.Aries` → "Ar" |
| `nakshatra.<number>` | 27 | `nakshatra.1` (Ashwini), `nakshatra.27` (Revati) |
| **Total** | **71** | |
