import type { Lang, CityRecord, HoroscopeResponse, WasmError } from './types';

const BUF_LIST_CITIES = 524288;   // 512 KiB
const BUF_HOROSCOPE   = 65536;    //  64 KiB

function bridge(op: string, input: object): object {
  const enc = new TextEncoder();
  const opBytes    = enc.encode(op + '\0');
  const inputBytes = enc.encode(JSON.stringify(input) + '\0');

  const opPtr  = window.Module._malloc(opBytes.length);
  const inPtr  = window.Module._malloc(inputBytes.length);
  window.Module.HEAPU8.set(opBytes,    opPtr);
  window.Module.HEAPU8.set(inputBytes, inPtr);

  const bufSize = op === 'list_cities' ? BUF_LIST_CITIES : BUF_HOROSCOPE;
  const outPtr  = window.Module._malloc(bufSize);

  try {
    const written = window.Module._bridge(opPtr, inPtr, outPtr, bufSize);

    if (written < 0) {
      const raw = window.Module.HEAPU8.subarray(outPtr, outPtr + bufSize);
      const text = new TextDecoder().decode(raw).replace(/\0.*/, '');
      let errMsg = `bridge error ${written}`;
      try {
        const parsed = JSON.parse(text) as { error?: string };
        if (parsed.error) errMsg = parsed.error;
      } catch { /* ignore */ }
      const wasmError: WasmError = { code: written, message: errMsg };
      throw wasmError;
    }

    const bytes = window.Module.HEAPU8.subarray(outPtr, outPtr + written);
    return JSON.parse(new TextDecoder().decode(bytes)) as object;
  } finally {
    window.Module._free(opPtr);
    window.Module._free(inPtr);
    window.Module._free(outPtr);
  }
}

export async function listCities(lang: Lang): Promise<{ cities: CityRecord[] }> {
  return bridge('list_cities', { operation: 'list_cities', lang }) as { cities: CityRecord[] };
}

export async function getHoroscopePositions(
  cityId: number,
  localTime: string,
  lang: Lang
): Promise<HoroscopeResponse> {
  return bridge('horoscope_positions', {
    operation: 'horoscope_positions',
    cityId,
    localTime,
    lang,
  }) as HoroscopeResponse;
}
