// astro-glue.js — WASM initialisation and bridge helper.
// Sourced by index.html before astro.js so Module is declared first.
// Do not add application logic here; see data.js.

// FR-008: Graceful fallback when WebAssembly is unavailable.
if (typeof WebAssembly === 'undefined') {
  console.error('WebAssembly is not supported in this browser. Calculations are unavailable.');
}

// Bridge JS helper — Option B retry pattern (contracts/wasm-api-v2.md).
// op:        string — operation name (e.g. "sun_longitude")
// inputJson: string — JSON-serialised wasm-api-v2 request body
// Returns:   parsed JS object (SunResult shape: { label?, longitude?, error? })
var INITIAL_OUTPUT_SIZE = 4096;

function bridge(op, inputJson) {
  var enc = new TextEncoder();
  var opBytes    = enc.encode(op + '\0');
  var inputBytes = enc.encode(inputJson + '\0');

  var opPtr  = Module._malloc(opBytes.length);
  var inPtr  = Module._malloc(inputBytes.length);
  Module.HEAPU8.set(opBytes,    opPtr);
  Module.HEAPU8.set(inputBytes, inPtr);

  var outPtr  = Module._malloc(INITIAL_OUTPUT_SIZE);
  var outSize = INITIAL_OUTPUT_SIZE;

  try {
    var written = Module._bridge(opPtr, inPtr, outPtr, outSize);

    if (written > outSize) {
      // Buffer too small — Rust returned required size; retry once.
      Module._free(outPtr);
      outPtr  = Module._malloc(written);
      outSize = written;
      written = Module._bridge(opPtr, inPtr, outPtr, outSize);
    }

    if (written < 0) {
      // Bridge returned negative code; try to parse best-effort ErrorResponse.
      var errBytes = Module.HEAPU8.subarray(outPtr, outPtr + outSize);
      try {
        return JSON.parse(new TextDecoder().decode(errBytes).replace(/\0.*/, ''));
      } catch (_) {
        throw new Error('bridge error code: ' + written);
      }
    }

    var bytes = Module.HEAPU8.subarray(outPtr, outPtr + written);
    return JSON.parse(new TextDecoder().decode(bytes));
  } finally {
    Module._free(opPtr);
    Module._free(inPtr);
    Module._free(outPtr);
  }
}

// Module must be declared before <script src="astro.js"> loads.
var Module = {
  onRuntimeInitialized: function () {
    // WASM is ready. UI scripts may now call bridge().
    document.dispatchEvent(new Event('wasm-ready'));
  }
};
