# WASM API Contract: Horoscope Positions v1

**Version**: 1.0.0  
**Operation**: `horoscope_positions`  
**Status**: Active  
**Feature**: `006-horoscope-positions`  
**Coordinate System**: Sidereal — True Chitrapaksha Ayanamsa (`SE_SIDM_TRUE_CITRA`, mode 27)  
**House System**: Whole Sign (`'W'`)

---

## Bridge Signature

```c
// Existing bridge export — unchanged
int bridge(
    const char* op_ptr,         // "horoscope_positions" (null-terminated)
    const char* input_ptr,      // JSON request (null-terminated)
    char*       output_ptr,     // JSON response written here
    int         output_max_len  // size of output buffer
);
```

**Return values** (unchanged from existing bridge contract):

| Return value | Meaning |
|---|---|
| `> 0` and `<= output_max_len` | Bytes written to `output_ptr` (success) |
| `> output_max_len` | Buffer too small; value is required size |
| `-1` | Unknown operation name |
| `-2` | JSON parse error, invalid field, or `op_ptr` / JSON `"operation"` field mismatch |
| `-3` | Calculation error (Swiss Ephemeris returned error) |

---

## Request

**JSON schema** — all fields required:

```json
{
  "operation": "horoscope_positions",
  "cityId": 11705,
  "localTime": "1961-10-28T07:30:00",
  "lang": "te"
}
```

| Field | Type | Constraints |
|-------|------|-------------|
| `operation` | string | must equal `"horoscope_positions"` |
| `cityId` | integer (u32) | must exist in compiled city store |
| `localTime` | string | `YYYY-MM-DDTHH:MM:SS`, no timezone suffix |
| `lang` | string | BCP-47; unknown values fall back to `"en"` |

---

## Response (success)

```json
{
  "lang": "en",
  "cityId": 11705,
  "cityName": "Hyderabad",
  "region1": "Telangana",
  "region2": "India",
  "lat": 17.97,
  "lng": 78.75,
  "timezone": "Asia/Kolkata",
  "planets": {
    "Ascendant": {
      "name": "Ascendant",
      "abbrev": "As",
      "longitude": 193.41,
      "zodiacNumber": 7,
      "zodiacSign": "Libra",
      "zodiacAbbrev": "Li",
      "degreesInSign": 13,
      "minutes": 24,
      "seconds": 36,
      "nakshatra": 15,
      "nakshatraName": "Swati",
      "pada": 1,
      "navamsaZodiacNumber": 7,
      "navamsaZodiacSign": "Libra",
      "navamsaZodiacAbbrev": "Li"
    },
    "Sun": { "...same shape..." },
    "Moon": { "...same shape..." },
    "Mars": { "...same shape..." },
    "Mercury": { "...same shape..." },
    "Jupiter": { "...same shape..." },
    "Venus": { "...same shape..." },
    "Saturn": { "...same shape..." },
    "Rahu": { "...same shape..." },
    "Ketu": { "...same shape..." }
  }
}
```

### PlanetaryPosition shape

| JSON key | Type | Description |
|----------|------|-------------|
| `name` | string | Translated full planet name |
| `abbrev` | string | Translated 2-character abbreviation |
| `longitude` | number (f64) | Absolute sidereal longitude, [0, 360) |
| `zodiacNumber` | integer (u8) | 1–12 (Aries = 1) |
| `zodiacSign` | string | Translated zodiac sign name |
| `zodiacAbbrev` | string | Translated 2-character sign abbreviation |
| `degreesInSign` | integer (u8) | 0–29, truncated |
| `minutes` | integer (u8) | 0–59, truncated |
| `seconds` | integer (u8) | 0–59, truncated |
| `nakshatra` | integer (u8) | 1–27 (Ashwini = 1) |
| `nakshatraName` | string | Translated nakshatra name |
| `pada` | integer (u8) | 1–4 |
| `navamsaZodiacNumber` | integer (u8) | 1–12 |
| `navamsaZodiacSign` | string | Translated navamsa sign name |
| `navamsaZodiacAbbrev` | string | Translated 2-character navamsa abbreviation |

---

## Error Response

```json
{ "error": "city not found: cityId=99999" }
```

Returned whenever the bridge function returns a negative value. The `error` field is always a non-empty, human-readable string.

---

## Localization Keys

All keys are resolved via the existing `localization::get_string(key, lang)` function.

**Planet names** (canonical key = `planet.<CanonicalName>`):  
`planet.Ascendant`, `planet.Sun`, `planet.Moon`, `planet.Mars`, `planet.Mercury`, `planet.Jupiter`, `planet.Venus`, `planet.Saturn`, `planet.Rahu`, `planet.Ketu`

**Planet abbreviations** (key = `planet.abbrev.<CanonicalName>`):  
`planet.abbrev.Ascendant`, `planet.abbrev.Sun`, ... `planet.abbrev.Ketu`

**Zodiac sign names** (key = `sign.<EnglishName>`):  
`sign.Aries`, `sign.Taurus`, `sign.Gemini`, `sign.Cancer`, `sign.Leo`, `sign.Virgo`, `sign.Libra`, `sign.Scorpio`, `sign.Sagittarius`, `sign.Capricorn`, `sign.Aquarius`, `sign.Pisces`

**Zodiac abbreviations** (key = `sign.abbrev.<EnglishName>`):  
`sign.abbrev.Aries`, ..., `sign.abbrev.Pisces`

**Nakshatra names** (key = `nakshatra.<number>`, 1–27):  
`nakshatra.1` (Ashwini) → `nakshatra.27` (Revati)

---

## Calculation Notes

- **Ayanamsa**: `swe_set_sid_mode(27, 0.0, 0.0)` called on every request. `SEFLG_SIDEREAL` added to iflag for `swe_calc_ut` and `swe_houses_ex`.
- **Rahu**: Swiss Ephemeris `SE_TRUE_NODE` (body 11). True node, not mean node.
- **Ketu**: Not a separate SE lookup. `ketu_longitude = normalize(rahu_longitude + 180.0)`.
- **Ascendant**: `swe_houses_ex` with `hsys = 'W'`; `ascmc[0]` is the sidereal Ascendant degree.
- **Coordinates**: Zoom-7 tile-centre formula — `lat/lng` from `decode_city_id(cityId)`.
- **Timezone**: `localTime` converted to UTC via `chrono-tz` using city's IANA timezone.

---

## Versioning

Breaking changes (parameter rename, field removal, type change) increment `MAJOR`.  
Additive changes (new optional response field) increment `MINOR`.  
Clarifications increment `PATCH`.
