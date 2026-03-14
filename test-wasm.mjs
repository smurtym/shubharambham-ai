// test-wasm.mjs — headless smoke test for the bridge() function.
// Polyfills browser APIs so Emscripten's web target loads under Node.js.
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import assert from 'assert';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DIST = join(__dirname, 'dist');

// ---------------------------------------------------------------------------
// Minimal browser globals required by the Emscripten web target
// ---------------------------------------------------------------------------

global.window = global;
global.document = { currentScript: null };
global.location = { href: `file://${DIST}/`, pathname: `${DIST}/` };

// fetch polyfill — Emscripten uses this to load the .wasm binary and .data bundle.
// Must match the Headers/ReadableStream API Emscripten accesses.
global.fetch = async (url) => {
  const cleanUrl = String(url).replace(/\?.*$/, '').replace(/^file:\/\//, '');
  const absPath = cleanUrl.startsWith('/') ? cleanUrl : join(DIST, cleanUrl);
  const data = readFileSync(absPath);
  const buf = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
  const u8 = new Uint8Array(buf);
  return {
    ok: true,
    status: 200,
    url: cleanUrl,
    headers: {
      get: (name) => name.toLowerCase() === 'content-length' ? String(u8.length) : null,
    },
    body: {
      getReader: () => {
        let sent = false;
        return {
          read: async () => {
            if (sent) return { done: true, value: undefined };
            sent = true;
            return { done: false, value: u8 };
          },
        };
      },
    },
    arrayBuffer: () => Promise.resolve(buf),
    text: () => Promise.resolve(data.toString('utf8')),
  };
};

// XMLHttpRequest polyfill — Emscripten uses this for the .data preload bundle.
global.XMLHttpRequest = class XMLHttpRequest {
  constructor() {
    this.responseType = '';
    this.response = null;
    this.status = 0;
    this.onload = null;
    this.onerror = null;
  }
  open(method, url) {
    this._url = String(url);
  }
  send() {
    const cleanUrl = this._url.replace(/\?.*$/, '').replace(/^file:\/\//, '');
    const absPath = cleanUrl.startsWith('/') ? cleanUrl : join(DIST, cleanUrl);
    try {
      const data = readFileSync(absPath);
      this.status = 200;
      if (this.responseType === 'arraybuffer') {
        this.response = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
      } else {
        this.response = data.toString('utf8');
      }
      if (this.onload) this.onload({ target: this });
    } catch (e) {
      this.status = 404;
      if (this.onerror) this.onerror({ target: this });
    }
  }
};

// ---------------------------------------------------------------------------
// Bridge test helper
// ---------------------------------------------------------------------------

const INITIAL_OUTPUT_SIZE = 4096;

function callBridge(Module, op, inputJson) {
  const enc = new TextEncoder();
  const opBytes    = enc.encode(op + '\0');
  const inputBytes = enc.encode(inputJson + '\0');

  const opPtr  = Module._malloc(opBytes.length);
  const inPtr  = Module._malloc(inputBytes.length);
  Module.HEAPU8.set(opBytes,    opPtr);
  Module.HEAPU8.set(inputBytes, inPtr);

  let outPtr  = Module._malloc(INITIAL_OUTPUT_SIZE);
  let outSize = INITIAL_OUTPUT_SIZE;

  try {
    let written = Module._bridge(opPtr, inPtr, outPtr, outSize);
    if (written > outSize) {
      Module._free(outPtr);
      outPtr  = Module._malloc(written);
      outSize = written;
      written = Module._bridge(opPtr, inPtr, outPtr, outSize);
    }
    if (written < 0) throw new Error('bridge error code: ' + written);
    const bytes = Module.HEAPU8.subarray(outPtr, outPtr + written);
    return JSON.parse(new TextDecoder().decode(bytes));
  } finally {
    Module._free(opPtr);
    Module._free(inPtr);
    Module._free(outPtr);
  }
}

// ---------------------------------------------------------------------------
// Run with the module's onRuntimeInitialized callback
// ---------------------------------------------------------------------------

global.Module = {
  onRuntimeInitialized: function () {
    try {
      // J2000.0 = 2000-01-01 12:00 UTC → expected Sun apparent ecliptic longitude ≈ 280.37°
      // (Swiss Ephemeris returns apparent longitude; mean longitude ~280.466° differs by ~0.09°)
      const result = callBridge(global.Module, 'sun_longitude', JSON.stringify({ tjd: 2451545.0 }));
      console.log(`Sun longitude (J2000): ${result.longitude.toFixed(6)}°`);

      const diff = Math.abs(result.longitude - 280.37);
      assert(diff < 0.1, `Longitude ${result.longitude} is not within 0.1° of 280.37°`);
      console.log('✓ SC-003 PASS: Longitude within 0.1° of reference value');
    } catch (e) {
      console.error('FAIL:', e.message);
      process.exit(1);
    }
  }
};

const src = readFileSync(join(DIST, 'astro.js'), 'utf8');
// Indirect eval runs in global scope so Emscripten's
// `var Module = typeof Module !== "undefined" ? Module : {}`
// finds our global.Module (with onRuntimeInitialized) instead of creating a new one.
// eslint-disable-next-line no-eval
(0, eval)(src);
