# WASM API Contract: Vimsottari Dasa v1

**Version**: 1.0.0
**Operation**: `vimsottari_dasa`
**Status**: Draft
**Feature**: `009-vimsottari-dasa`
**Coordinate System**: Sidereal — True Chitrapaksha Ayanamsa (`SE_SIDM_TRUE_CITRA`, mode 27)

---

## Bridge Signature

```c
// Existing bridge export — unchanged
int bridge(
    const char* op_ptr,         // "vimsottari_dasa" (null-terminated)
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
| `-2` | JSON parse error, invalid field, or mismatched operation |
| `-3` | Calculation error (Swiss Ephemeris returned error) |

---

## Request

**JSON schema** — all fields required:

```json
{
  "operation": "vimsottari_dasa",
  "cityId": 11705,
  "localTime": "1997-03-07T20:34:00",
  "lang": "en"
}
```

| Field | Type | Constraints |
|-------|------|-------------|
| `operation` | string | must equal `"vimsottari_dasa"` |
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
  "periods": [
    {
      "lord": "Mars",
      "label": "Mars Mahadasa",
      "startDate": "1997 March 07",
      "endDate": "1999 January 18",
      "antardasas": [
        {
          "lord": "Venus",
          "label": "Venus Antardasa",
          "startDate": "1997 March 07",
          "endDate": "1998 February 11"
        },
        {
          "lord": "Sun",
          "label": "Sun Antardasa",
          "startDate": "1998 February 11",
          "endDate": "1998 June 19"
        },
        {
          "lord": "Moon",
          "label": "Moon Antardasa",
          "startDate": "1998 June 19",
          "endDate": "1999 January 18"
        }
      ]
    },
    {
      "lord": "Rahu",
      "label": "Rahu Mahadasa",
      "startDate": "1999 January 18",
      "endDate": "2017 January 18",
      "antardasas": [
        {
          "lord": "Rahu",
          "label": "Rahu Antardasa",
          "startDate": "1999 January 18",
          "endDate": "2001 October 01"
        },
        {
          "lord": "Jupiter",
          "label": "Jupiter Antardasa",
          "startDate": "2001 October 01",
          "endDate": "2004 February 23"
        }
      ]
    }
  ]
}
```

*Note: Response abbreviated. Full response contains 9 Mahadasa entries; each full Mahadasa contains 9 Antardasa entries. The first (partial) Mahadasa may have fewer Antardasas.*

---

## Response fields

### Top-level

| Field | Type | Description |
|-------|------|-------------|
| `lang` | string | Echoed input |
| `cityId` | integer | Echoed input |
| `cityName` | string | Translated city name for the requested `lang` |
| `region1` | string | Translated state/province |
| `region2` | string | Translated country |
| `lat` | number | Decimal latitude, 3 decimal places |
| `lng` | number | Decimal longitude, 3 decimal places |
| `timezone` | string | IANA timezone (e.g. `"Asia/Kolkata"`) |
| `periods` | array | Ordered array of Mahadasa objects (exactly 9) |

### Mahadasa object

| Field | Type | Description |
|-------|------|-------------|
| `lord` | string | Canonical English planet key (e.g. `"Venus"`) — language-independent |
| `label` | string | Translated: `"{planet_name} {dasa.maha}"` |
| `startDate` | string | `"YYYY MonthName DD"` with localized month name, zero-padded day |
| `endDate` | string | Same format; equals next Mahadasa's `startDate` |
| `antardasas` | array | Ordered Antardasa objects (9 for full Mahadasas; ≤9 for first) |

### Antardasa object

| Field | Type | Description |
|-------|------|-------------|
| `lord` | string | Canonical English planet key — language-independent |
| `label` | string | Translated: `"{planet_name} {dasa.antar}"` |
| `startDate` | string | Same date format as Mahadasa |
| `endDate` | string | Same format; equals next Antardasa's `startDate` within Mahadasa |

---

## Date format

Dates follow the pattern: `YYYY MonthName DD`

- `YYYY`: 4-digit year
- `MonthName`: Full translated month name via `get_string("month.{1-12}", lang)`
- `DD`: Zero-padded day (01–31)

Examples:
- English: `"1997 March 07"`, `"2017 January 18"`
- Telugu: `"1997 మార్చి 07"`, `"2017 జనవరి 18"`

---

## Contiguity invariants

1. `periods[0].startDate` = birth date
2. `periods[i].endDate` = `periods[i+1].startDate` for all `i` in `0..8`
3. `periods[i].antardasas[j].endDate` = `periods[i].antardasas[j+1].startDate` for all valid `j`
4. `periods[i].antardasas[0].startDate` = `periods[i].startDate`
5. `periods[i].antardasas[last].endDate` = `periods[i].endDate`

---

## Error response

```json
{
  "error": "city not found: cityId=99999"
}
```

| Condition | Error message pattern |
|-----------|----------------------|
| Unknown `cityId` | `"city not found: cityId={id}"` |
| Malformed `localTime` | `"invalid localTime '{value}': {detail}"` |
| JSON parse failure | `"JSON parse error: {detail}"` |
| SE calculation failure | `"calc_planet(Moon) failed: {detail}"` |

---

## Localization keys (new)

14 keys added to each locale (`en.rs`, `te.rs`):

| Key | English | Telugu |
|-----|---------|--------|
| `dasa.maha` | `Mahadasa` | `మహాదశ` |
| `dasa.antar` | `Antardasa` | `అంతర్దశ` |
| `month.1` | `January` | `జనవరి` |
| `month.2` | `February` | `ఫిబ్రవరి` |
| `month.3` | `March` | `మార్చి` |
| `month.4` | `April` | `ఏప్రిల్` |
| `month.5` | `May` | `మే` |
| `month.6` | `June` | `జూన్` |
| `month.7` | `July` | `జూలై` |
| `month.8` | `August` | `ఆగస్టు` |
| `month.9` | `September` | `సెప్టెంబర్` |
| `month.10` | `October` | `అక్టోబర్` |
| `month.11` | `November` | `నవంబర్` |
| `month.12` | `December` | `డిసెంబర్` |

Existing `planet.*` keys are reused for planet names in labels.
