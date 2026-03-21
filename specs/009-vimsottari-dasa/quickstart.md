# Quickstart: Vimsottari Dasa

**Feature**: `009-vimsottari-dasa`
**WASM bridge operation**: `vimsottari_dasa`

---

## Get Vimsottari Dasa periods from JavaScript

The JS/TS layer calls the WASM bridge with a `cityId`, a local-time string, and a BCP-47 language tag:

```ts
// Invoke the bridge with operation "vimsottari_dasa"
const request = JSON.stringify({
  operation: "vimsottari_dasa",
  cityId: 11705,
  localTime: "1997-03-07T20:34:00",
  lang: "en"
});

const result = bridge("vimsottari_dasa", request);
console.log(result);
```

---

## Sample request JSON

```json
{
  "operation": "vimsottari_dasa",
  "cityId": 11705,
  "localTime": "1997-03-07T20:34:00",
  "lang": "en"
}
```

---

## Sample response JSON (abbreviated)

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
        { "lord": "Rahu", "label": "Rahu Antardasa", "startDate": "1999 January 18", "endDate": "2001 October 01" },
        { "lord": "Jupiter", "label": "Jupiter Antardasa", "startDate": "2001 October 01", "endDate": "2004 February 23" },
        "... (7 more Antardasas)"
      ]
    },
    "... (7 more Mahadasas)"
  ]
}
```

---

## Telugu response (same request with `lang=te`)

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
  "periods": [
    {
      "lord": "Mars",
      "label": "కుజ మహాదశ",
      "startDate": "1997 మార్చి 07",
      "endDate": "1999 జనవరి 18",
      "antardasas": [
        {
          "lord": "Venus",
          "label": "శుక్ర అంతర్దశ",
          "startDate": "1997 మార్చి 07",
          "endDate": "1998 ఫిబ్రవరి 11"
        }
      ]
    }
  ]
}
```

---

## Build and test

```bash
# Run Rust unit tests (includes vimsottari reference chart tests)
cd astro-wasm && cargo test

# Build WASM (from repo root)
./build.sh
```

---

## Key files

| File | Description |
|------|-------------|
| `astro-wasm/src/engines/vimsottari.rs` | Dasa engine — request parsing, Moon lookup, period computation, response assembly |
| `astro-wasm/src/engines/mod.rs` | Engine registry — `pub mod vimsottari;` |
| `astro-wasm/src/bridge.rs` | Bridge routing — `"vimsottari_dasa"` dispatch arm |
| `astro-wasm/src/locales/en.rs` | English locale — 14 new keys (dasa labels + month names) |
| `astro-wasm/src/locales/te.rs` | Telugu locale — 14 new keys |
| `specs/009-vimsottari-dasa/contracts/vimsottari-dasa-api-v1.md` | API contract |
