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

## Step 2 — Obtain ephemeris data files

The ephemeris data files are not committed to Git. Obtain the two required files
(`semo_18.se1`, `sepl_18.se1`) and place them in the `ephe/` directory:

```bash
mkdir -p ephe
```

> **Note**: The Swiss Ephemeris FTP download URL
> (`https://www.astro.com/ftp/swisseph/ephe/`) may be unavailable. If so, copy
> the files from another local Swiss Ephemeris installation:
> ```bash
> cp /path/to/existing/swisseph/ephe/semo_18.se1 ephe/
> cp /path/to/existing/swisseph/ephe/sepl_18.se1 ephe/
> ```

Verify:

```bash
ls -lh ephe/
# semo_18.se1  ~1.3 MB
# sepl_18.se1  ~450 KB
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
[build.sh] ✓ Prerequisites OK (Emscripten 4.x.y, swisseph submodule, ephe/ files, Rust target)
[build.sh] ✓ libswe.a compiled → lib/libswe.a
[build.sh] ✓ WASM artifacts written to dist/ (astro.js, astro.wasm, astro.data)
[build.sh] ✓ Web assets copied to dist/
[build.sh] Build complete. Serve dist/ with: python3 -m http.server 8080 --directory dist/
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
Sun longitude (J2000): 280.368919°
```

This is printed by `onRuntimeInitialized` calling `bridge('sun_longitude', JSON.stringify({ tjd: 2451545.0 }))`.  
The value is the Sun's **apparent geocentric ecliptic longitude** at J2000.0 as computed by
Swiss Ephemeris. It differs from the mean longitude (~280.466°) due to the equation of center
and other corrections. Any value in the range 280.3°–280.5° is correct.

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
