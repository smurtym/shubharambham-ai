# Quickstart: Horoscope Positions

**Feature**: `006-horoscope-positions`  
**WASM bridge operation**: `horoscope_positions`

---

## Get horoscope positions from JavaScript

The JS helper lives in `web/data.js`. Call it with a `cityId`, an ISO local-time string, and a BCP-47 language tag:

```js
import { getHoroscopePositions } from './data.js';

// 1961-10-28 07:30 local time in Hyderabad (cityId 11705)
const result = await getHoroscopePositions(11705, '1961-10-28T07:30:00', 'te');
console.log(result);
```

The function signature is:

```ts
getHoroscopePositions(
  cityId: number,       // integer city ID from the compiled store
  localTime: string,    // "YYYY-MM-DDTHH:MM:SS" — no timezone suffix
  lang: string          // BCP-47, e.g. "en" or "te"; unknown falls back to "en"
): Promise<HoroscopeResponse>
```

---

## Sample request JSON

The bridge receives this string (assembled by `getHoroscopePositions`):

```json
{
  "operation": "horoscope_positions",
  "cityId": 11705,
  "localTime": "1961-10-28T07:30:00",
  "lang": "te"
}
```

---

## Sample response JSON (abbreviated)

Language `"te"` returns all names in Telugu:

```json
{
  "lang": "te",
  "cityId": 11705,
  "cityName": "హైదరాబాద్",
  "region1": "తెలంగాణ",
  "region2": "భారతదేశం",
  "lat": 17.97,
  "lng": 78.75,
  "timezone": "Asia/Kolkata",
  "planets": {
    "Ascendant": {
      "name": "లగ్నం",
      "abbrev": "ల",
      "longitude": 193.41,
      "zodiacNumber": 7,
      "zodiacSign": "తుల",
      "zodiacAbbrev": "తు",
      "degreesInSign": 13,
      "minutes": 24,
      "seconds": 36,
      "nakshatra": 15,
      "nakshatraName": "స్వాతి",
      "pada": 1,
      "navamsaZodiacNumber": 7,
      "navamsaZodiacSign": "తుల",
      "navamsaZodiacAbbrev": "తు"
    },
    "Sun": {
      "name": "సూర్యుడు",
      "abbrev": "సూ",
      "longitude": 191.15,
      "zodiacNumber": 7,
      "zodiacSign": "తుల",
      "zodiacAbbrev": "తు",
      "degreesInSign": 11,
      "minutes": 9,
      "seconds": 0,
      "nakshatra": 15,
      "nakshatraName": "స్వాతి",
      "pada": 1,
      "navamsaZodiacNumber": 7,
      "navamsaZodiacSign": "తుల",
      "navamsaZodiacAbbrev": "తు"
    }
    // ... Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu
  }
}
```

The `planets` object always contains exactly 10 keys (fixed order):  
`Ascendant, Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu`

---

## Error response

When the bridge returns a negative value, `getHoroscopePositions` rejects with an error string from the `"error"` field:

```json
{ "error": "city not found: cityId=99999" }
```

---

## Add a new language

1. Create `astro-wasm/src/locales/<lang>.rs` following the pattern in `en.rs`.
2. Add entries for all 71 locale keys (20 planet keys, 24 zodiac-sign keys, 27 nakshatra keys).
3. Register the new module and match arm in `astro-wasm/src/locales/mod.rs`.
4. Rebuild: `bash build.sh`.
5. Call with `lang: "<new-lang-code>"` — the fallback to `"en"` is automatic for any key missing from the new locale.

---

## Rebuild (development)

```bash
cd /astro/shubharambham-ai
bash build.sh
```

The WASM binary and JS glue are emitted to `public/`.

---

## Run tests

```bash
cd astro-wasm
cargo test --target wasm32-unknown-emscripten   # WASM target
cargo test                                       # native fast tests
```
