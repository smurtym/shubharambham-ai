# Research: Vimsottari Dasa

**Branch**: `009-vimsottari-dasa` | **Date**: 2026-03-21

All NEEDS CLARIFICATION items from the Technical Context are resolved below.

---

## R-001: Moon longitude reuse strategy

**Decision**: Call `swe_wrappers::calc_planet(jd, SE_MOON)` directly from the vimsottari engine — the same function `horoscope.rs` uses.
**Rationale**: The function is a thin FFI wrapper that takes a Julian Day and Swiss Ephemeris body constant, returning the sidereal longitude as `f64`. It is stateless (beyond the global SE mode set by `swe_set_sid_mode`) and safe to call from any engine. The vimsottari engine sets `swe_set_sid_mode(SE_SIDM_TRUE_CITRA, 0.0, 0.0)` and `swe_set_ephe_path` before calling `calc_planet` — identical to what `horoscope.rs` does.
**Alternatives considered**: Extracting Moon longitude from the horoscope engine's response — rejected because it would couple vimsottari to horoscope's internal struct and require serialization round-trips. Direct SE call is simpler, faster, and decoupled.

**Usage in vimsottari.rs**:
```rust
swe_wrappers::swe_set_ephe_path_safe();
unsafe { swe_wrappers::swe_set_sid_mode(swe_wrappers::SE_SIDM_TRUE_CITRA, 0.0, 0.0); }
let moon_lon = swe_wrappers::calc_planet(jd, swe_wrappers::SE_MOON)?;
```

---

## R-002: Nakshatra identification from Moon longitude

**Decision**: Use `utils::decompose_longitude(moon_lon)` to obtain the nakshatra number (1–27) and Moon's position within the nakshatra.
**Rationale**: `decompose_longitude` returns `(zodiac_num, deg_in_sign, minutes, seconds, nakshatra_num, pada)`. The nakshatra number maps directly to the Vimsottari lord table (FR-003). For the balance fraction (FR-004), the Moon's exact position within the nakshatra must be computed from the raw longitude:

```rust
let nakshatra_span = 13.0 + 1.0/3.0;  // 13.333... degrees
let nakshatra_start = (nakshatra_num as f64 - 1.0) * nakshatra_span;
let nakshatra_end = nakshatra_start + nakshatra_span;
let balance_fraction = (nakshatra_end - moon_lon) / nakshatra_span;
```

`decompose_longitude` provides the nakshatra number; the balance fraction is computed from the raw `moon_lon` f64 to preserve full precision (no truncation from the integer deg/min/sec fields).
**Alternatives considered**: Computing nakshatra number manually from `floor(moon_lon / 13.333...)` — functionally equivalent but redundant since `decompose_longitude` already does this and is tested.

---

## R-003: Nakshatra-to-lord mapping

**Decision**: Use a compile-time lookup array indexed by `(nakshatra_num - 1) % 9`.
**Rationale**: The 27 nakshatras map to 9 lords in a repeating cycle:

| Index (mod 9) | Lord | Nakshatras |
|---|---|---|
| 0 | Ketu | 1 (Ashwini), 10 (Magha), 19 (Mula) |
| 1 | Venus | 2 (Bharani), 11 (P.Phalguni), 20 (P.Ashadha) |
| 2 | Sun | 3 (Krittika), 12 (U.Phalguni), 21 (U.Ashadha) |
| 3 | Moon | 4 (Rohini), 13 (Hasta), 22 (Shravana) |
| 4 | Mars | 5 (Mrigashira), 14 (Chitra), 23 (Dhanishtha) |
| 5 | Rahu | 6 (Ardra), 15 (Swati), 24 (Shatabhisha) |
| 6 | Jupiter | 7 (Punarvasu), 16 (Vishakha), 25 (P.Bhadrapada) |
| 7 | Saturn | 8 (Pushya), 17 (Anuradha), 26 (U.Bhadrapada) |
| 8 | Mercury | 9 (Ashlesha), 18 (Jyeshtha), 27 (Revati) |

```rust
const DASA_SEQUENCE: [DasaLord; 9] = [
    DasaLord::Ketu, DasaLord::Venus, DasaLord::Sun, DasaLord::Moon,
    DasaLord::Mars, DasaLord::Rahu, DasaLord::Jupiter, DasaLord::Saturn,
    DasaLord::Mercury,
];

fn nakshatra_to_lord_index(nakshatra_num: u8) -> usize {
    ((nakshatra_num - 1) % 9) as usize
}
```

**Alternatives considered**: HashMap — unnecessary overhead for a 9-element fixed mapping.

---

## R-004: Date arithmetic with chrono

**Decision**: Use `chrono::NaiveDateTime` (not `NaiveDate`) for all period boundary computation. The full birth datetime — including time-of-day — is the base point. Convert fractional days to `chrono::TimeDelta` for addition. Extract the date and round only at the final formatting step.
**Rationale**: `chrono` (already in `Cargo.toml`) handles leap years, month lengths, and century boundaries correctly. Truncating the birth time to date-only would shift every computed boundary by up to ±1 day (e.g., a birth at 20:34 is 0.857 of a day past midnight — ignoring that offset propagates through all cumulative additions). The approach:

1. Birth datetime → `NaiveDateTime` (from the full `localTime` string, e.g. `"1997-03-07T20:34:00"`)
2. Compute total days for each period as `f64` (fractional)
3. Accumulate boundaries as `f64` offsets from birth datetime
4. At formatting time: add the cumulative offset as fractional seconds via `TimeDelta`, then extract the `.date()` portion. If the resulting time-of-day is ≥ 12:00:00, advance to the next calendar day (rounding to nearest day).

This ensures contiguity (FR-010): each boundary is computed from the same base datetime + cumulative fractional offset. Because all boundaries share the same sub-day base, rounding to calendar dates cannot create gaps.

```rust
use chrono::{NaiveDateTime, NaiveDate, TimeDelta};

let birth_dt = NaiveDateTime::parse_from_str(local_time, "%Y-%m-%dT%H:%M:%S").unwrap();
// cumulative_days is f64, accumulated across all periods
let total_secs = (cumulative_days * 86_400.0).round() as i64;
let boundary_dt = birth_dt + TimeDelta::seconds(total_secs);
// Round to nearest calendar day: if time >= 12:00, advance to next day
let boundary_date = if boundary_dt.time() >= chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap() {
    boundary_dt.date() + TimeDelta::days(1)
} else {
    boundary_dt.date()
};
```

**Alternatives considered**:
- `NaiveDate` (date-only base) — rejected because truncating the time component loses up to 0.999 days of precision, causing ±1 day errors in all downstream boundaries.
- Julian Day arithmetic throughout — rejected because JD→calendar date conversion at output would require custom code; `chrono` does this correctly.
- Adding `Duration::days()` sequentially per period — rejected because sequential rounding could accumulate errors and break contiguity (the cumulative-offset approach avoids this).

---

## R-005: Solar year definition

**Decision**: `const SOLAR_YEAR_DAYS: f64 = 365.2425;`
**Rationale**: FR-006 specifies exactly 365.2425 days per solar year (Gregorian mean year). The total cycle is `120 × 365.2425 = 43829.1` days. All dasa durations are multiples of this constant.

| Lord | Years | Days (exact) |
|---|---|---|
| Ketu | 7 | 2556.6975 |
| Venus | 20 | 7304.85 |
| Sun | 6 | 2191.455 |
| Moon | 10 | 3652.425 |
| Mars | 7 | 2556.6975 |
| Rahu | 18 | 6573.165 |
| Jupiter | 16 | 5843.88 |
| Saturn | 19 | 6939.0075 |
| Mercury | 17 | 6191.4225 |
| **Total** | **120** | **43829.1** |

---

## R-006: First Mahadasa partial Antardasa handling

**Decision**: Compute all 9 Antardasas for the first Mahadasa starting from the Mahadasa lord, then skip those whose cumulative end falls before or at the balance start.
**Rationale**: Per clarification, the first (partial) Mahadasa includes only the Antardasas that fall after the birth moment. Within a Mahadasa, the Antardasas follow the Vimsottari sequence starting from the Mahadasa lord. For the partial first Mahadasa:

1. Compute the birth lord's position in the Vimsottari sequence
2. Compute 9 Antardasa durations (proportional: `mahadasa_days × lord_years / 120`)
3. The elapsed portion before birth spans some initial Antardasas. Identify which Antardasa the birth falls within.
4. That Antardasa starts on the birth date (with its remaining portion). Subsequent ones follow normally.

Implementation: cumulate Antardasa durations from the Mahadasa start. The birth falls at `(1.0 - balance_fraction) × full_mahadasa_days` from the Mahadasa start. All Antardasas whose cumulative end ≤ elapsed portion are skipped. The first included Antardasa starts on the birth date.

---

## R-007: Localization keys needed

**Decision**: Add 14 new keys to each locale file.
**Rationale**: FR-017 requires dasa label templates and 12 month names. The planet name portion of labels (e.g., "Venus") is already in the locale tables (`planet.Venus`). The new keys provide the label suffixes and month names:

| Key | English | Telugu |
|---|---|---|
| `dasa.maha` | `Mahadasa` | `మహాదశ` |
| `dasa.antar` | `Antardasa` | `అంతర్దశ` |
| `month.1` | `January` | `జనవరి` |
| `month.2` | `February` | `ఫిబ్రవరి` |
| `month.3` | `March` | `మార్చి` |
| `month.4` | `April` | `ఏప్రిల్` |
| `month.5` | `May` | `మే` |
| `month.6` | `June` | `జూన్` |
| `month.7` | `July` | `జూలై` |
| `month.8` | `August` | `ఆగస్టు` |
| `month.9` | `September` | `సెప్టెంబర్` |
| `month.10` | `October` | `అక్టోబర్` |
| `month.11` | `November` | `నవంబర్` |
| `month.12` | `December` | `డిసెంబర్` |

Locale array size: 71 → 85. Compile-time assertion updated accordingly.

---

## R-008: Adding the bridge dispatch arm safely

**Decision**: Add `"vimsottari_dasa" => { ... engines::vimsottari::execute(input) ... }` as a new arm in the `match op` block in `bridge.rs`, before the `_ =>` fallback.
**Rationale**: The bridge uses exhaustive string matching. Adding a new arm is purely additive — no existing arm is modified, reordered, or removed. The pattern is identical to how `"horoscope_positions"` was added in feature 006.

```rust
match op {
    "stub_op" => { /* existing */ },
    "list_cities" => { /* existing */ },
    "horoscope_positions" => { /* existing */ },
    "vimsottari_dasa" => {
        let json = engines::vimsottari::execute(input);
        write_json(&json, output_ptr, output_max_len)
    },
    _ => write_error(&format!("unknown operation: {op}"), -1, output_ptr, output_max_len),
}
```

**Risk assessment**: Zero risk to existing operations. The match is on exact string equality; no regex, no prefix matching. Compile-time verification ensures the new arm is syntactically valid.
