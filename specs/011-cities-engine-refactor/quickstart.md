# Quickstart Validation Guide: Cities Engine Refactor

**Feature**: `011-cities-engine-refactor` | **Date**: 2026-06-27

## Prerequisites

- Rust stable toolchain installed
- Working directory: repo root (`/astro/shubharambham-ai`)
- No Emscripten or ephemeris files required for this change

## Validation Steps

### Step 1 — Run the full bridge test suite

```bash
cd astro-wasm && cargo test
```

**Expected outcome**: All tests pass, including:

| Test | What it validates |
|------|-------------------|
| `test_bridge_list_cities_en` | `list_cities` returns correct English city list |
| `test_bridge_list_cities_te_filters` | Telugu-only cities are filtered correctly |
| `test_bridge_canonical_name_invariant` | `canonicalName` is always ASCII regardless of `lang` |
| `test_bridge_list_cities_missing_lang` | Missing `lang` field returns `-2` + error JSON |
| `test_bridge_malformed_json_returns_minus2` | Malformed JSON returns `-2` |
| `test_bridge_operation_mismatch_returns_minus2` | Operation mismatch returns `-2` |
| `test_bridge_unknown_op_returns_minus1` | Unknown op still returns `-1` |

### Step 2 — Verify structural changes (code inspection)

**`bridge.rs`** must satisfy all of the following:
- No `CitiesRequest` struct defined
- No `dispatch_list_cities` function defined
- `list_cities` arm is a match expression delegating to `engines::cities::execute(input)`

**`engines/mod.rs`** must contain `pub mod cities;`

**`engines/cities.rs`** must exist and contain:
- `CitiesRequest` struct (private, derives `Deserialize`)
- `pub fn execute(input: &str) -> Result<String, String>`

### Step 3 — Confirm no TypeScript or contract changes

```bash
git diff --name-only
```

**Expected**: Only `astro-wasm/src/bridge.rs`, `astro-wasm/src/engines/mod.rs`, and `astro-wasm/src/engines/cities.rs` appear in the diff. No changes to:
- `web/horoscope/astro-glue.ts`
- `specs/004-city-data/contracts/city-data-v1.md`
- Any test file under `astro-wasm/src/bridge.rs` `#[cfg(test)]` block

## Pass Criteria

The feature is complete when:
1. `cargo test` exits with zero failures
2. Structural inspection confirms items in Step 2
3. `git diff` confirms no out-of-scope files changed
