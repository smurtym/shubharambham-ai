# Quickstart: Infrastructure and Tooling Setup

**Feature**: 001-infrastructure-tooling-setup  
**Target audience**: Developer setting up the project for the first time  
**Estimated time**: ~25 minutes (toolchain pre-installed)

---

## Prerequisites

Verify these are installed and on your PATH before proceeding:

```bash
emcc --version    # Must show 4.0.x or later
rustc --version   # Must show 1.88.0 or later
rustup target list --installed | grep wasm32-unknown-emscripten
# If not listed: rustup target add wasm32-unknown-emscripten
```

---

## Step 1 — Clone with submodules

```bash
git clone --recurse-submodules https://github.com/smurtym/shubharambham-ai.git
cd shubharambham-ai

# If you already cloned without --recurse-submodules:
git submodule update --init
```

Verify the submodule is present:

```bash
ls vendor/swisseph/sweph.c   # Must exist
```

---

## Step 2 — Download ephemeris data files

The ephemeris data files are not committed to Git. Download the two required files from
the Swiss Ephemeris FTP server:

```bash
mkdir -p ephe
curl -o ephe/semo_18.se1 "https://www.astro.com/ftp/swisseph/ephe/semo_18.se1"
curl -o ephe/sepl_18.se1 "https://www.astro.com/ftp/swisseph/ephe/sepl_18.se1"
```

Verify:

```bash
ls -lh ephe/
# semo_18.se1  ~5 MB
# sepl_18.se1  ~10 MB
```

---

## Step 3 — Build

```bash
./build.sh
```

The script will:
1. Check all prerequisites (Emscripten, Rust target, `vendor/swisseph/`, `ephe/` files)
2. Compile `vendor/swisseph/*.c` → `build/libswe.a` using `emcc`
3. Build `astro-wasm/` Rust crate → WASM via `cargo build --target wasm32-unknown-emscripten`
4. Re-link with `--preload-file ephe/@/ephe/` and `-sEXPORTED_FUNCTIONS=_bridge`
5. Copy `dist/astro.js`, `dist/astro.wasm`, `dist/astro.data`, `dist/index.html`, `dist/style.css`

Expected final output:

```
[build.sh] ✓ Prerequisites OK
[build.sh] ✓ libswe.a compiled
[build.sh] ✓ astro-wasm built
[build.sh] ✓ dist/ populated
Build complete. Serve dist/ with any HTTP server.
```

---

## Step 4 — Serve and verify

You cannot open `dist/index.html` directly as a `file://` URL because browsers block WASM
loading from `file://`. Use a local HTTP server:

```bash
python3 -m http.server 8080 --directory dist/
```

Open `http://localhost:8080` in Chrome, Firefox, or Safari.

**Expected browser behaviour**:
- Page displays the text "Work in Progress"
- Open the browser developer console (F12 → Console). You should see:

```
Sun longitude (J2000): 280.459500°
```

This is printed by `onRuntimeInitialized` calling `bridge('sun_longitude', JSON.stringify({ tjd: 2451545.0 }))`.  
The value should be within 0.001° of 280.459° (J2000 reference).

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| `build.sh: emcc not found` | Emscripten not on PATH | Source the Emscripten environment: `source $EMSDK/emsdk_env.sh` |
| `build.sh: vendor/swisseph is empty` | Submodule not initialized | `git submodule update --init` |
| `build.sh: ephe/sepl_18.se1 not found` | Ephemeris files not downloaded | Repeat Step 2 |
| Console shows `bridge error code: -3` | Ephemeris files not found at `/ephe/` in WASM | Ensure `build.sh` ran the `--preload-file` link step; check `dist/astro.data` exists |
| Browser shows nothing / blank console | WASM not supported | Upgrade browser to Chrome 57+, Firefox 53+, or Safari 11+ |
| `rustup target add` fails | Rust toolchain too old | `rustup update stable` |
