// data.js — web-side registry of WASM features.
// Depends on astro-glue.js being loaded first (bridge() must be defined).

/**
 * Fetch the sorted, language-filtered city list from the WASM bridge.
 *
 * All data is compiled into the WASM binary — no network request is made.
 *
 * @param {string} lang - Language code, e.g. "en" or "te"
 * @returns {Array<{lang: string, cityId: number, timeZone: string,
 *                  canonicalName: string, cityName: string,
 *                  region1: string, region2: string,
 *                  lat: number, lng: number}>}
 * @throws {Error} if the bridge returns an error response
 */
function listCities(lang) {
  const response = bridge(
    'list_cities',
    JSON.stringify({ operation: 'list_cities', lang: lang })
  );
  if (response.error) {
    throw new Error(response.error);
  }
  return response.cities;
}
