# Data Model: Horoscope Positions

**Branch**: `006-horoscope-positions` | **Date**: 2026-03-15

---

## Entities

### HoroscopeRequest

Input to the `"horoscope_positions"` bridge operation.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `operation` | string | must equal `"horoscope_positions"` | Bridge operation name (required by bridge contract) |
| `cityId` | u16 | must exist in CITIES store | 16-bit zoom-7 quadkey identifying the city |
| `localTime` | string | `YYYY-MM-DDTHH:MM:SS`, no tz suffix | Date and time in the city's local timezone |
| `lang` | string | BCP-47; unknown → falls back to `"en"` | Display language for all translated strings |

---

### HoroscopeResponse

Top-level JSON output object.

| Field | Type | Description |
|-------|------|-------------|
| `lang` | string | Echoed input lang |
| `cityId` | u16 | Echoed input city ID |
| `cityName` | string | Translated city name |
| `region1` | string | Translated state/province |
| `region2` | string | Translated country |
| `lat` | f64 | City latitude (tile-centre, decimal degrees) |
| `lng` | f64 | City longitude (tile-centre, decimal degrees) |
| `timezone` | string | IANA timezone string (e.g. `"Asia/Kolkata"`) |
| `planets` | object | Keyed map of 10 `PlanetaryPosition` entries |

`planets` key set (fixed, English canonical names):
`"Ascendant"`, `"Sun"`, `"Moon"`, `"Mars"`, `"Mercury"`, `"Jupiter"`, `"Venus"`, `"Saturn"`, `"Rahu"`, `"Ketu"`

---

### PlanetaryPosition

Value in the `planets` map. One entry per celestial body.

| Field | JSON key | Type | Description |
|-------|----------|------|-------------|
| Planet name | `name` | string | Translated full name (e.g. "Sun" / "సూర్యుడు") |
| Planet abbreviation | `abbrev` | string | Translated 2-letter abbrev (e.g. "Su" / "సూ") |
| Sidereal longitude | `longitude` | f64 | Absolute sidereal degrees in [0, 360) |
| Zodiac sign number | `zodiacNumber` | u8 | 1–12 (Aries = 1) |
| Zodiac sign name | `zodiacSign` | string | Translated (e.g. "Capricorn" / "మకరం") |
| Zodiac abbreviation | `zodiacAbbrev` | string | Translated 2-letter (e.g. "Cp" / "మ") |
| Degrees in sign | `degreesInSign` | u8 | 0–29 (truncated) |
| Minutes | `minutes` | u8 | 0–59 (truncated) |
| Seconds | `seconds` | u8 | 0–59 (truncated) |
| Nakshatra number | `nakshatra` | u8 | 1–27 (Ashwini = 1) |
| Nakshatra name | `nakshatraName` | string | Translated (e.g. "Sravana" / "శ్రవణం") |
| Pada | `pada` | u8 | 1–4 |
| Navamsa sign number | `navamsaZodiacNumber` | u8 | 1–12 |
| Navamsa sign name | `navamsaZodiacSign` | string | Translated |
| Navamsa abbreviation | `navamsaZodiacAbbrev` | string | Translated 2-letter |

---

### CelestialBody (Rust enum — internal)

Maps canonical body names to Swiss Ephemeris integer constants.

| Variant | SE constant | SE value | Notes |
|---------|-------------|----------|-------|
| Sun | SE_SUN | 0 | — |
| Moon | SE_MOON | 1 | — |
| Mercury | SE_MERCURY | 2 | — |
| Venus | SE_VENUS | 3 | — |
| Mars | SE_MARS | 4 | — |
| Jupiter | SE_JUPITER | 5 | — |
| Saturn | SE_SATURN | 6 | — |
| Rahu | SE_TRUE_NODE | 11 | True Node |
| Ketu | — | derived | Rahu longitude + 180° mod 360 |
| Ascendant | — | derived | `swe_houses_ex` `ascmc[0]` |

---

## State Transitions / Processing Pipeline

```
HoroscopeRequest (JSON)
  │
  ├─ Validate operation == "horoscope_positions"
  ├─ Lookup cityId in CITIES → CityRecord (error if not found)
  ├─ Resolve lat/lng via decode_city_id()
  ├─ local_to_jd(localTime, city.timezone) → jd (error if malformed)
  │
  ├─ swe_set_sid_mode(SE_SIDM_TRUE_CITRA=27, 0.0, 0.0)
  ├─ swe_set_ephe_path("/ephe")
  │
  ├─ For each of: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu
  │     calc_planet(jd, body) → tropical longitude
  │     Apply SEFLG_SIDEREAL (or use auto-subtraction via swe_set_sid_mode)
  │     decompose_longitude(sidereal_lon) → PlanetaryPosition fields
  │
  ├─ Ketu: sidereal_lon = normalize(Rahu.longitude + 180.0)
  │     decompose_longitude(ketu_lon) → PlanetaryPosition fields
  │
  ├─ Ascendant: calc_ascendant(jd, lat, lon) → sidereal ascendant degree
  │     decompose_longitude(asc_lon) → PlanetaryPosition fields
  │
  └─ Assemble HoroscopeResponse → serialize to JSON
```

---

## Validation Rules

| Field | Rule |
|-------|------|
| `cityId` | Must match a `city_id` in the compiled CITIES static array |
| `localTime` | Must parse as `%Y-%m-%dT%H:%M:%S`; DST gaps resolved by advancing 1 hour |
| `lang` | Unknown values fall back silently to `"en"`; no error |
| `operation` | Must equal `"horoscope_positions"` (bridge mismatch → -2) |

---

## Locale Key Table (49 new keys, both languages)

### Planet names and abbreviations (20 keys)

| Key | English | Telugu |
|-----|---------|--------|
| `planet.Ascendant` | Ascendant | లగ్నం |
| `planet.abbrev.Ascendant` | As | ల |
| `planet.Sun` | Sun | సూర్యుడు |
| `planet.abbrev.Sun` | Su | సూ |
| `planet.Moon` | Moon | చంద్రుడు |
| `planet.abbrev.Moon` | Mo | చం |
| `planet.Mars` | Mars | కుజుడు |
| `planet.abbrev.Mars` | Ma | కు |
| `planet.Mercury` | Mercury | బుధుడు |
| `planet.abbrev.Mercury` | Me | బు |
| `planet.Jupiter` | Jupiter | గురువు |
| `planet.abbrev.Jupiter` | Ju | గు |
| `planet.Venus` | Venus | శుక్రుడు |
| `planet.abbrev.Venus` | Ve | శు |
| `planet.Saturn` | Saturn | శని |
| `planet.abbrev.Saturn` | Sa | శ |
| `planet.Rahu` | Rahu | రాహువు |
| `planet.abbrev.Rahu` | Ra | రా |
| `planet.Ketu` | Ketu | కేతువు |
| `planet.abbrev.Ketu` | Ke | కే |

### Zodiac sign names and abbreviations (24 keys)

| Key | English | Telugu |
|-----|---------|--------|
| `sign.Aries` | Aries | మేషం |
| `sign.abbrev.Aries` | Ar | మేషం |
| `sign.Taurus` | Taurus | వృషభం |
| `sign.abbrev.Taurus` | Ta | వృషభం |
| `sign.Gemini` | Gemini | మిథునం |
| `sign.abbrev.Gemini` | Ge | మిథునం |
| `sign.Cancer` | Cancer | కర్కాటకం |
| `sign.abbrev.Cancer` | Cn | కర్కాటకం |
| `sign.Leo` | Leo | సింహం |
| `sign.abbrev.Leo` | Le | సింహం |
| `sign.Virgo` | Virgo | కన్య |
| `sign.abbrev.Virgo` | Vi | కన్య |
| `sign.Libra` | Libra | తుల |
| `sign.abbrev.Libra` | Li | తుల |
| `sign.Scorpio` | Scorpio | వృశ్చికం |
| `sign.abbrev.Scorpio` | Sc | వృశ్చికం |
| `sign.Sagittarius` | Sagittarius | ధనుస్సు |
| `sign.abbrev.Sagittarius` | Sg | ధనుస్సు |
| `sign.Capricorn` | Capricorn | మకరం |
| `sign.abbrev.Capricorn` | Cp | మకరం |
| `sign.Aquarius` | Aquarius | కుంభం |
| `sign.abbrev.Aquarius` | Aq | కుంభం |
| `sign.Pisces` | Pisces | మీనం |
| `sign.abbrev.Pisces` | Pi | మీనం |

### Nakshatra names (27 keys)

| Key | English | Telugu |
|-----|---------|--------|
| `nakshatra.1` | Ashwini | అశ్విని |
| `nakshatra.2` | Bharani | భరణి |
| `nakshatra.3` | Krittika | కృత్తిక |
| `nakshatra.4` | Rohini | రోహిణి |
| `nakshatra.5` | Mrigashira | మృగశిర |
| `nakshatra.6` | Ardra | ఆర్ద్ర |
| `nakshatra.7` | Punarvasu | పునర్వసు |
| `nakshatra.8` | Pushya | పుష్యమి |
| `nakshatra.9` | Ashlesha | ఆశ్లేష |
| `nakshatra.10` | Magha | మఘ |
| `nakshatra.11` | Purva Phalguni | పుబ్బ |
| `nakshatra.12` | Uttara Phalguni | ఉత్తర |
| `nakshatra.13` | Hasta | హస్త |
| `nakshatra.14` | Chitra | చిత్ర |
| `nakshatra.15` | Swati | స్వాతి |
| `nakshatra.16` | Vishakha | విశాఖ |
| `nakshatra.17` | Anuradha | అనురాధ |
| `nakshatra.18` | Jyeshtha | జ్యేష్ట |
| `nakshatra.19` | Mula | మూల |
| `nakshatra.20` | Purva Ashadha | పూర్వాషాఢ |
| `nakshatra.21` | Uttara Ashadha | ఉత్తరాషాఢ |
| `nakshatra.22` | Shravana | శ్రవణం |
| `nakshatra.23` | Dhanishtha | ధనిష్ట |
| `nakshatra.24` | Shatabhisha | శతభిషం |
| `nakshatra.25` | Purva Bhadrapada | పూర్వభాద్ర |
| `nakshatra.26` | Uttara Bhadrapada | ఉత్తరభాద్ర |
| `nakshatra.27` | Revati | రేవతి |
