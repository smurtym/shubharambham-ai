# Data Model: Cities Engine Refactor

**Feature**: `011-cities-engine-refactor` | **Date**: 2026-06-27

## Overview

This refactor moves an existing struct from `bridge.rs` into `engines/cities.rs`. No new data structures are introduced; no data layer changes.

---

## Struct: `CitiesRequest` (moves from `bridge.rs` → `engines/cities.rs`)

**Visibility**: Private to `engines::cities`

**Purpose**: Deserialised from the JSON `input_ptr` payload when the `list_cities` operation is invoked.

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `operation` | `String` | yes | Must equal `"list_cities"`; mismatch → `Err("operation mismatch: ...")` |
| `lang` | `String` | yes | Missing field → serde parse error → `Err("JSON parse error: ...")` |

**Derives**: `serde::Deserialize`

**Notes**:
- `lang` is a language tag (`"en"`, `"te"`). An unknown but syntactically present tag is not an error at this layer — `data::list_cities` returns an empty cities array.
- The struct is consumed within `execute`; it is never returned or exposed outside the module.

---

## Public API: `engines::cities::execute`

```
pub fn execute(input: &str) -> Result<String, String>
```

| Case | Returns |
|------|---------|
| Valid JSON, operation matches, lang present | `Ok(json)` where `json` is the serialised city list from `data::list_cities` |
| Malformed JSON | `Err("JSON parse error: <serde error>")` |
| `operation` field ≠ `"list_cities"` | `Err("operation mismatch: op_ptr=list_cities body=<value>")` |

**Bridge arm mapping**:

```
Ok(json)  → write_json(&json, output_ptr, output_max_len)  → positive byte count
Err(msg)  → write_error(&msg, -2, output_ptr, output_max_len)  → -2
```

---

## Unchanged Data Layer

The following are explicitly out of scope and unchanged:

| Component | Location | Status |
|-----------|----------|--------|
| `CityRecord` | `data/mod.rs` | Unchanged |
| `CityResponse` | `data/mod.rs` | Unchanged |
| `data::list_cities()` | `data/mod.rs` | Unchanged |
| `data::cities::CITIES` | `data/cities.rs` | Unchanged |
| `data::decode_city_id()` | `data/mod.rs` | Unchanged |
