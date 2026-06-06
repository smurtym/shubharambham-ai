# Data Model: 008 — Horoscope UI Polish

**Phase**: 1 — Design  
**Feature**: `008-horoscope-ui-polish`

All types are TypeScript, located in `web/horoscope/types.ts`.  
Field names match the JSON field names returned by the WASM bridge exactly.

---

## Core Types

### `Lang`
```typescript
type Lang = 'en' | 'te';
```
Derived from the `?lang=` URL query parameter. Defaults to `'en'` if absent or unrecognised.

---

### `CityRecord`
Represents one city returned by the `list_cities` WASM operation.

```typescript
interface CityRecord {
  lang: string;           // BCP-47 language code of returned strings, e.g. "en" | "te"
  cityId: number;         // Unique numeric ID (u32 in Rust); base-4 zoom-15 quadkey encoded as decimal
  timeZone: string;       // IANA timezone string, e.g. "Asia/Kolkata"
  canonicalName: string;  // Canonical (English) city name, always ASCII
  cityName: string;       // Localised city name (may equal canonicalName for "en")
  region1: string;        // State/province (localised)
  region2: string;        // Country (localised)
  lat: number;            // Latitude (f64), tile-centre derived from cityId
  lng: number;            // Longitude (f64), tile-centre derived from cityId
}
```

---

### `PlanetaryPosition`
Shape of one entry in `HoroscopeResponse.planets`.  
Field names match `horoscope-api-v1.md` exactly.

```typescript
interface PlanetaryPosition {
  name: string;               // Translated full planet name
  abbrev: string;             // Translated 2-character abbreviation
  longitude: number;          // Absolute sidereal longitude [0, 360)
  zodiacNumber: number;       // 1–12, Aries = 1
  zodiacSign: string;         // Translated zodiac sign name
  zodiacAbbrev: string;       // Translated 2-character sign abbreviation
  degreesInSign: number;      // 0–29 (truncated)
  minutes: number;            // 0–59
  seconds: number;            // 0–59
  nakshatra: number;          // 1–27, Ashwini = 1
  nakshatraName: string;      // Translated nakshatra name
  pada: number;               // 1–4
  navamsaZodiacNumber: number;  // 1–12
  navamsaZodiacSign: string;    // Translated navamsa sign name
  navamsaZodiacAbbrev: string;  // Translated 2-character navamsa abbreviation
}
```

---

### `HoroscopeResponse`
Full response from the `horoscope_positions` WASM operation.

```typescript
interface HoroscopeResponse {
  lang: string;
  cityId: number;
  cityName: string;
  region1: string;
  region2: string;
  lat: number;
  lng: number;
  timezone: string;
  planets: Record<string, PlanetaryPosition>;
  // Keys: "Ascendant" | "Sun" | "Moon" | "Mars" | "Mercury"
  //       | "Jupiter" | "Venus" | "Saturn" | "Rahu" | "Ketu"
}
```

---

### `WasmError`
Thrown by `bridge()` when the WASM bridge returns a negative value or an error JSON.

```typescript
interface WasmError {
  code: number;   // WASM return code: -1 unknown op, -2 bad JSON, -3 calc error
  message: string;
}
```

---

## UI State Models

### `AppState`
Top-level state managed by `App.tsx`.

```typescript
interface AppState {
  lang: Lang;            // Read from ?lang=, never mutated at runtime
  wasmReady: boolean;    // true after 'wasm-ready' event fires
  result: HoroscopeResponse | null;  // null = no chart yet
  error: string | null;  // non-null = WASM error displayed to user
}
```

---

### `BirthFormState`
State owned by `App.tsx`; passed as props to `BirthForm.tsx`.

```typescript
interface BirthFormState {
  cityId: number | null;       // null = no city selected
  cityDisplayName: string;     // shown in Location field after selection
  localTime: string;           // ISO-8601 without TZ: YYYY-MM-DDTHH:MM:SS
  modalOpen: boolean;
  loading: boolean;            // true = currently calling WASM
}
```

---

### `CityModalState`
Local state in `CityModal.tsx`.

```typescript
interface CityModalState {
  query: string;
  cities: CityRecord[];
  loading: boolean;
  noResults: boolean;   // true when query non-empty but cities.length === 0
}
```

---

## Constants

### `SIGN_CELL` — South Indian Chakra Grid Placement
Maps `zodiacNumber` (1–12) to a `{ row, col }` pair for a 4×4 CSS Grid.  
Rows and cols are **0-indexed**. The 2×2 centre (`rows 1–2, cols 1–2`) is the chart label.

```typescript
const SIGN_CELL: Record<number, { row: number; col: number }> = {
  1:  { row: 0, col: 1 },  // Aries
  2:  { row: 0, col: 2 },  // Taurus
  3:  { row: 0, col: 3 },  // Gemini
  4:  { row: 1, col: 3 },  // Cancer
  5:  { row: 2, col: 3 },  // Leo
  6:  { row: 3, col: 3 },  // Virgo
  7:  { row: 3, col: 2 },  // Libra
  8:  { row: 3, col: 1 },  // Scorpio
  9:  { row: 3, col: 0 },  // Sagittarius
  10: { row: 2, col: 0 },  // Capricorn
  11: { row: 1, col: 0 },  // Aquarius
  12: { row: 0, col: 0 },  // Pisces
};
```

---

## Component Props

### `ChakraGrid`
```typescript
interface ChakraGridProps {
  planets: Record<string, PlanetaryPosition>;
  title: string;       // "Rasi" or "Navamsa" (localised)
  navamsa?: boolean;   // false/absent = use zodiacNumber; true = use navamsaZodiacNumber
}
```

### `LongitudeTable`
```typescript
interface LongitudeTableProps {
  planets: Record<string, PlanetaryPosition>;
}
```

### `EchoPanel`
```typescript
interface EchoPanelProps {
  response: HoroscopeResponse;
  dateOfBirth: string;   // YYYY-MM-DD from <input type="date">
  timeOfBirth: string;   // HH:MM from <input type="time">
}
```

### `CityModal`
```typescript
interface CityModalProps {
  open: boolean;
  lang: Lang;
  onSelect: (city: CityRecord) => void;
  onClose: () => void;
}
```

### `LanguageSelector`
```typescript
interface LanguageSelectorProps {
  current: Lang;
  // Navigation to ?lang=en or ?lang=te is done internally via window.location
}
```
