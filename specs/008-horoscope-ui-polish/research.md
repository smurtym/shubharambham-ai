# Research: 008 — Horoscope UI Polish

**Phase**: 0 — Outline & Research  
**Feature**: `008-horoscope-ui-polish`  
**Status**: Complete

---

## 1. Emscripten WASM + Vite Integration

### Decision
Load `public/astro.js` via a plain `<script src="/astro.js">` tag in `web/horoscope/index.html`, **not** via ESM `import`. Declare `window.Module` with `onRuntimeInitialized` in an inline `<script>` placed before the `astro.js` script tag.

### Rationale
Emscripten's output is a non-standard IIFE that expects a global `Module` object at load time. Vite's ESM pipeline cannot import it; attempting to do so causes `Module is not a function` or silent breakage. The existing `web/astro-glue.js` uses this same inline-global pattern — the new TypeScript layer replicates it exactly.

### Implementation Pattern
```html
<!-- web/horoscope/index.html — BEFORE <script src="/astro.js"> -->
<script>
  window.Module = {
    onRuntimeInitialized: function () {
      document.dispatchEvent(new Event('wasm-ready'));
    }
  };
</script>
<script src="/astro.js"></script>
```

React code listens for `'wasm-ready'` on `document` before enabling the UI.

### TypeScript Ambient Declaration
```typescript
// web/horoscope/emscripten.d.ts
declare global {
  interface Window {
    Module: {
      _malloc: (size: number) => number;
      _bridge: (opPtr: number, inPtr: number, outPtr: number, outSize: number) => number;
      _free: (ptr: number) => void;
      HEAPU8: Uint8Array;
      onRuntimeInitialized?: () => void;
    };
  }
}
export {};
```

### Alternatives Considered
- **Vite `assetsInclude` + dynamic import**: Rejected — Emscripten output has mutable global state; no clean ESM interface.
- **Web Worker**: Rejected — over-engineering; WASM loads fast enough (~200ms) for the feature budget.

---

## 2. Vite Multi-Page App

### Decision
Add `web/horoscope/index.html` as a second entry in `vite.config.ts` via `rollupOptions.input`.

### Rationale
Vite natively supports multi-page apps by listing HTML files as additional Rollup inputs. This requires zero extra plugins, keeps the existing `web/index.html` build intact, and produces a clean `dist/horoscope/index.html` output served at `/horoscope`.

### Implementation
```typescript
// vite.config.ts additions
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

// In defineConfig:
plugins: [react(), ...existingPlugins],
build: {
  rollupOptions: {
    input: {
      main: resolve(__dirname, 'web/index.html'),
      horoscope: resolve(__dirname, 'web/horoscope/index.html'),
    },
  },
},
```

### Alternatives Considered
- **Separate Vite project in `web/horoscope/`**: Rejected — breaks the shared `public/astro.js` path; adds a second build system.
- **React Router within existing page**: Rejected — violates spec decision to keep `/horoscope` as a separate page at `/horoscope`; existing vanilla page must be untouched.

---

## 3. React + TypeScript + MUI v6 Setup

### Decision
Install the following packages:

```bash
# devDependencies
npm install --save-dev \
  @vitejs/plugin-react@^4.2.0 \
  @types/react@^18.2.0 \
  @types/react-dom@^18.2.0

# dependencies
npm install \
  react@^18.3.0 \
  react-dom@^18.3.0 \
  @mui/material@^6.0.0 \
  @emotion/react@^11.11.0 \
  @emotion/styled@^11.11.0
```

### Rationale
Constitution v2.0.0 mandates React 18+, MUI v6+, and TypeScript strict. MUI v6 requires Emotion as its CSS-in-JS engine. `@vitejs/plugin-react` provides Fast Refresh and JSX transform.

### tsconfig.json Additions
```json
{
  "compilerOptions": {
    "jsx": "react-jsx",
    "moduleResolution": "bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"]
  },
  "include": [
    "**/*.config.ts",
    "tests/**/*.ts",
    "web/**/*.ts",
    "web/**/*.tsx"
  ]
}
```

### Alternatives Considered
- **MUI v5**: Rejected — constitution mandates v6+.
- **Styled-components instead of Emotion**: Rejected — MUI v6 default CSS-in-JS is Emotion; mixing would add bundle weight.
- **Preact**: Rejected — constitution mandates React 18+.

---

## 4. Fonts — Noto Sans + Noto Sans Telugu

### Decision
Load both fonts from Google Fonts CDN via `<link>` tags in `web/horoscope/index.html`.

### Implementation
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Noto+Sans:wght@400;500;600;700&family=Noto+Sans+Telugu:wght@400;500;600;700&display=swap" rel="stylesheet">
```

MUI theme baseline:
```typescript
typography: { fontFamily: '"Noto Sans", "Noto Sans Telugu", sans-serif' }
```

### Rationale
Constitution v2.0.0 mandates Noto Sans Telugu for Telugu script rendering. Google Fonts CDN is the lightest delivery mechanism — no self-hosting needed for this feature scope.

### Alternatives Considered
- **Self-hosted fonts**: Rejected — adds build complexity with no benefit for a static site.
- **System default font**: Rejected — no reliable system font supports Devanagari/Telugu Unicode ranges on Android Chrome.

---

## 5. South Indian Chakra Grid Layout

### Decision
Use a CSS Grid `4×4` with explicit `grid-row` / `grid-column` placement per cell. A constant `SIGN_CELL` map translates `zodiacNumber` (1–12) to `{row, col}` grid positions. The 2×2 centre (`rows 1-2, cols 1-2`) is a single merged cell with the chart label.

### Grid Cell Coordinate Map (0-indexed, row then col)

| Sign # | Name | Row | Col |
|--------|------|-----|-----|
| 1 | Aries | 0 | 1 |
| 2 | Taurus | 0 | 2 |
| 3 | Gemini | 0 | 3 |
| 4 | Cancer | 1 | 3 |
| 5 | Leo | 2 | 3 |
| 6 | Virgo | 3 | 3 |
| 7 | Libra | 3 | 2 |
| 8 | Scorpio | 3 | 1 |
| 9 | Sagittarius | 3 | 0 |
| 10 | Capricorn | 2 | 0 |
| 11 | Aquarius | 1 | 0 |
| 12 | Pisces | 0 | 0 |

Centre label spans `row 1–2, col 1–2` (CSS 1-indexed: `grid-row: 2/4; grid-column: 2/4`).

### Rationale
Explicit placement is deterministic and testable. MUI `Box` with `sx={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)' }}` handles the responsive sizing; cells scale naturally with container width.

### Alternatives Considered
- **Absolute positioning with percentages**: Rejected — fragile on different screen widths.
- **SVG overlay**: Rejected — over-engineering; not needed for text-only cells.

---

## 6. WASM Bridge — TypeScript Rewrite Strategy

### Decision
Implement `web/horoscope/astro-glue.ts` as a typed TypeScript rewrite of the existing `web/astro-glue.js`. The underlying `window.Module._bridge` call pattern is identical; only types and async helper are added.

### Key Exported Functions
```typescript
function bridge(op: string, input: object): object  // sync, throws on WASM error
async function listCities(lang: Lang): Promise<{ cities: CityRecord[] }>
async function getHoroscopePositions(cityId: number, localTime: string, lang: Lang): Promise<HoroscopeResponse>
```

### Rationale
The existing `bridge()` implementation is correct — `_malloc` → encode UTF-8 → `_bridge` → decode → `_free`. A direct TypeScript port minimizes risk and respects the WASM-library freeze constraint. No new WASM operations are added.

### Alternatives Considered
- **Import `web/data.js` from React app**: Rejected — path aliasing with non-TS files is fragile; TypeScript types would be lost.
- **Fetch-based JSON endpoint**: Rejected — WASM runs client-side; there is no server.
