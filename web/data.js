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

/**
 * Compute Vedic horoscope positions for a given city, local time, and language.
 *
 * @param {number} cityId     - 32-bit city identifier
 * @param {string} localTime  - Local datetime in "YYYY-MM-DDTHH:MM:SS" format
 * @param {string} lang       - Language code, e.g. "en" or "te"
 * @returns {Object} Full HoroscopeResponse including planets keyed by canonical name
 * @throws {Error} if the bridge returns an error response
 */
function getHoroscopePositions(cityId, localTime, lang) {
  const response = bridge(
    'horoscope_positions',
    JSON.stringify({ operation: 'horoscope_positions', cityId: cityId, localTime: localTime, lang: lang })
  );
  if (response.error) {
    throw new Error(response.error);
  }
  return response;
}
