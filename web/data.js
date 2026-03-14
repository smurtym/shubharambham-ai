// data.js — web-side registry of WASM features.
// Depends on astro-glue.js being loaded first (bridge() must be defined).

/**
 * Compute the Sun's ecliptic longitude for a given UTC datetime and locale.
 *
 * @param {string} isoDatetime - Full ISO 8601 UTC string, e.g. "2000-01-01T12:00:00Z"
 * @param {string} lang        - Locale identifier: "en" or "te"
 * @returns {{ label: string, longitude: number } | { error: string }}
 *   On success: { label: <localized planet name>, longitude: <decimal degrees> }
 *   On error:   { error: <human-readable message> }
 */
function getSunLongitude(isoDatetime, lang) {
  return bridge(
    'sun_longitude',
    JSON.stringify({ operation: 'sun_longitude', datetime: isoDatetime, lang: lang })
  );
}
