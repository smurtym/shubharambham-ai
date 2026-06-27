# Quickstart Validation Guide: Runtime City CSV

**Feature**: `012-runtime-city-csv` | **Date**: 2026-06-27

## Prerequisites

- Rust stable toolchain on PATH (`~/.cargo/bin/cargo`)
- `ephe/cities.csv` present (moved from `data/cities.csv`)
- Working directory: repo root (`/astro/shubharambham-ai`)

---

## Step 1 — Verify build.rs has no city codegen

```bash
grep -n "generate_cities\|escape_str\|cities\.csv" astro-wasm/build.rs
```

**Expected**: No output — these identifiers must not appear in `build.rs`.

---

## Step 2 — Verify cities.rs is gone and CSV is in ephe/

```bash
ls astro-wasm/src/data/cities.rs 2>&1   # must say: No such file or directory
ls ephe/cities.csv                       # must succeed
ls data/cities.csv 2>&1                 # must say: No such file or directory
```

---

## Step 3 — Run the full test suite

```bash
cd astro-wasm && /home/ubuntu/.cargo/bin/cargo test 2>&1
```

**Expected**: All tests pass (including all `test_bridge_list_cities_*`, `test_list_cities_*`, `test_city_ids_unique`, `test_timezones_valid`).

---

## Step 4 — Verify SC-004: Add a city without recompiling

```bash
# Append a test city row
echo "466083478,TestCity,Asia/Kolkata,en,TestCity,Telangana,India,99,1" >> ephe/cities.csv

# Run only the city listing test (no recompile needed if binary is already built)
cd astro-wasm && /home/ubuntu/.cargo/bin/cargo test test_list_cities_en_returns_all_five -- --nocapture 2>&1

# Confirm TestCity now appears in en results without a full build
```

**Expected**: Test passes with one more city than before.

```bash
# Clean up the test row
# Remove the last line from ephe/cities.csv
head -n -1 ephe/cities.csv > /tmp/cities_tmp.csv && mv /tmp/cities_tmp.csv ephe/cities.csv
```

---

## Step 5 — Verify error handling: missing CSV

```bash
# Temporarily rename the CSV
mv ephe/cities.csv ephe/cities.csv.bak

# Run a city test — expect it to fail gracefully (error, not panic)
cd astro-wasm && /home/ubuntu/.cargo/bin/cargo test test_bridge_list_cities_en -- --nocapture 2>&1

# Restore
mv ephe/cities.csv.bak ephe/cities.csv
```

**Expected**: The test fails, but the process does not panic/abort; an error message is returned.

---

## Pass Criteria

| Check | Expected |
|-------|----------|
| `build.rs` grep | No matches |
| `data/cities.rs` | Does not exist |
| `ephe/cities.csv` | Exists |
| `cargo test` | 0 failures |
| Add city row, run test | New city appears without recompile |
| Missing CSV | Error returned, no panic |
