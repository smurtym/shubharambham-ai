#!/usr/bin/env bash
# build.sh — single entry point for the shubharambham-ai build pipeline
# Usage: ./build.sh
#
# Phases:
#   1. Prerequisite checks (all must pass before any compilation)
#   2. Swiss Ephemeris C compilation → lib/libswe.a                     (added in T011)
#   3. Rust WASM build + dist/ assembly                                  (added in T014)
#   4. Static web assets copy                                            (added in T009)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EMCC="${EMCC:-emcc}"
EMAR="${EMAR:-emar}"

log()  { echo "[build.sh] $*"; }
fail() { echo "[build.sh] ERROR: $*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Phase 1 — Prerequisite checks
# ---------------------------------------------------------------------------

log "Checking prerequisites..."

# 1a. emcc on PATH and Emscripten >= 4.0.0
if ! command -v "$EMCC" > /dev/null 2>&1; then
  fail "emcc not found on PATH. Source the Emscripten environment first:\n  source \$EMSDK/emsdk_env.sh"
fi

EMCC_VERSION="$("$EMCC" --version 2>&1 | head -1 | grep -oP '\d+\.\d+\.\d+' | head -1)"
EMCC_MAJOR="$(echo "$EMCC_VERSION" | cut -d. -f1)"
if [[ "$EMCC_MAJOR" -lt 4 ]]; then
  fail "Emscripten >= 4.0.0 required, found $EMCC_VERSION"
fi

# 1b. vendor/swisseph submodule populated
if [[ ! -f "$REPO_ROOT/vendor/swisseph/sweph.c" ]]; then
  fail "vendor/swisseph submodule is empty. Run:\n  git submodule update --init"
fi

# 1c. Ephemeris data files present
for EPHE_FILE in semo_18.se1 sepl_18.se1; do
  if [[ ! -s "$REPO_ROOT/ephe/$EPHE_FILE" ]]; then
    fail "ephe/$EPHE_FILE not found or empty.\nDownload from: https://www.astro.com/ftp/swisseph/ephe/$EPHE_FILE"
  fi
done

# 1d. wasm32-unknown-emscripten Rust target installed
if ! rustup target list --installed 2>/dev/null | grep -q "wasm32-unknown-emscripten"; then
  fail "Rust target wasm32-unknown-emscripten not installed. Run:\n  rustup target add wasm32-unknown-emscripten"
fi

log "✓ Prerequisites OK (Emscripten $EMCC_VERSION, swisseph submodule, ephe/ files, Rust target)"

# ---------------------------------------------------------------------------
# Phase 2 — Compile Swiss Ephemeris C sources → lib/libswe.a
# ---------------------------------------------------------------------------

SWEDIR="$REPO_ROOT/vendor/swisseph"
OBJ_DIR="$REPO_ROOT/build/swe_objs"
LIB_DIR="$REPO_ROOT/lib"

mkdir -p "$OBJ_DIR" "$LIB_DIR"

SWE_SRCS="swedate.c swehouse.c swejpl.c swemmoon.c swemplan.c sweph.c swephlib.c"

log "Compiling Swiss Ephemeris C sources..."
for src in $SWE_SRCS; do
  obj="$OBJ_DIR/${src%.c}.o"
  "$EMCC" -O2 -c "$SWEDIR/$src" -o "$obj" \
    -I"$SWEDIR" \
    -DNOT_WINDOWS \
    -DEMSCRIPTEN
done

"$EMAR" rcs "$LIB_DIR/libswe.a" "$OBJ_DIR"/*.o
log "✓ libswe.a compiled → $LIB_DIR/libswe.a"

# ---------------------------------------------------------------------------
# Phase 3 — Rust WASM build; emcc writes output directly to dist/
# ---------------------------------------------------------------------------

export LIBSWE_DIR="$LIB_DIR"

# dist/ must exist before cargo runs — build.rs directs emcc output there.
mkdir -p "$REPO_ROOT/dist"

log "Building Rust WASM crate (astro-wasm)..."
cd "$REPO_ROOT/astro-wasm"
cargo build --target wasm32-unknown-emscripten --release 2>&1
cd "$REPO_ROOT"

if [[ ! -f "$REPO_ROOT/dist/astro.js" ]]; then
  fail "dist/astro.js was not produced. Check the emcc link step above."
fi
log "✓ WASM artifacts written to dist/ (astro.js, astro.wasm, astro.data)"

# ---------------------------------------------------------------------------
# Phase 4 — Static web assets copy
# ---------------------------------------------------------------------------

mkdir -p "$REPO_ROOT/dist"
cp "$REPO_ROOT/web/index.html" "$REPO_ROOT/web/style.css" "$REPO_ROOT/dist/"
log "✓ Web assets copied to dist/"

log "Build complete. Serve dist/ with: python3 -m http.server 8080 --directory dist/"
