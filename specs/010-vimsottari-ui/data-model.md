# Data Model: Vimsottari Dasa Display (010-vimsottari-ui)

**Generated**: 2026-03-21  
**Source**: Existing WASM API contract (`specs/009-vimsottari-dasa/contracts/vimsottari-dasa-api-v1.md`)

This feature is purely frontend. The data model consists of TypeScript types that mirror the WASM engine's JSON response, plus derived UI state types used internally by React components.

---

## TypeScript Types (additions to `types.ts`)

### `AntardasaEntry`

Maps 1:1 to the WASM response field `periods[n].antardasas[m]`.

```typescript
export interface AntardasaEntry {
  lord:      string;   // canonical English planet key (e.g. "Venus") — language-independent
  label:     string;   // translated: "{planet_name} {dasa.antar}" (e.g. "Venus Antardasa")
  startDate: string;   // "YYYY MonthName DD" — month in requested lang
  endDate:   string;   // "YYYY MonthName DD" — equals next antardasa's startDate
  isCurrent: boolean;  // true if today falls within startDate..endDate; set by Rust engine
}
```

### `MahadasaEntry`

Maps 1:1 to the WASM response field `periods[n]`.

```typescript
export interface MahadasaEntry {
  lord:       string;           // canonical English planet key (e.g. "Mars")
  label:      string;           // translated: "{planet_name} {dasa.maha}" (e.g. "Mars Mahadasa")
  startDate:  string;           // "YYYY MonthName DD"
  endDate:    string;           // "YYYY MonthName DD" — equals next mahadasa's startDate
  isCurrent:  boolean;          // true if any child AntardasaEntry.isCurrent is true; set by Rust
  antardasas: AntardasaEntry[]; // ordered; up to 9 (fewer for first partial Mahadasa)
}
```

### `VimsottariResponse`

Top-level WASM response for `vimsottari_dasa` operation.

```typescript
export interface VimsottariResponse {
  lang:     string;          // echoed input
  cityId:   number;          // echoed input
  cityName: string;          // translated city name
  region1:  string;          // translated state/province
  region2:  string;          // translated country
  lat:      number;          // decimal latitude, 3dp
  lng:      number;          // decimal longitude, 3dp
  timezone: string;          // IANA timezone string
  periods:  MahadasaEntry[]; // ordered Mahadasa periods; exactly 9 in a full chart
}
```

---

## UI State (internal to `App.tsx` and `VimsottariPanel`)

### State fields added to `App.tsx`

| State variable | Type | Description |
|---|---|---|
| `dasa` | `VimsottariResponse \| null` | Fetched dasa data; `null` until fetched or after input change |
| `dasaLoading` | `boolean` | `true` while `getVimsottariDasa()` is in progress |
| `dasaError` | `string \| null` | Error message from dasa fetch; `null` on success |

### `VimsottariPanel` props (component contract)

```typescript
export interface VimsottariPanelProps {
  dasa:    VimsottariResponse | null;  // null while loading or after a fetch error
  loading: boolean;
  error:   string | null;
  lang:    Lang;
}
```

### Internal state in `VimsottariPanel`

| State variable | Type | Description |
|---|---|---|
| `expanded` | `Set<number>` | Indices of currently expanded accordion rows; initialised with the active Mahadasa index |

---

## Data Relationships

```
App.tsx
 ├── result: HoroscopeResponse           ← from getHoroscopePositions()
 └── dasa:   VimsottariResponse          ← from getVimsottariDasa() (parallel)

VimsottariResponse
 └── periods: MahadasaEntry[]            (0..9)
      └── antardasas: AntardasaEntry[]   (0..9 per Mahadasa)

Active-period detection (Rust-side, read-only on frontend):
  AntardasaEntry.isCurrent = true  →  highlight Antardasa row
  MahadasaEntry.isCurrent  = true  →  pre-expand accordion (derived from child AntardasaEntries)
  AntardasaEntry.startDate ≤ today < AntardasaEntry.endDate → highlight row
```

---

## Field Provenance

| Field | Source | Transformation |
|-------|--------|----------------|
| `MahadasaEntry.label` | WASM engine (pre-translated) | Rendered verbatim — no frontend transformation |
| `AntardasaEntry.label` | WASM engine (pre-translated) | Rendered verbatim |
| `MahadasaEntry.startDate` / `endDate` | WASM engine (formatted, localized month) | Rendered verbatim — active-period detection performed in Rust via `isCurrent` flag |
| `AntardasaEntry.startDate` / `endDate` | WASM engine | Rendered verbatim — same as above |
| `expanded` Set | Derived from `periods.findIndex(p => p.isCurrent)` on component mount | Client-side only; not persisted |
