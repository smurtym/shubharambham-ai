# Quickstart: 008 — Horoscope UI Polish

**Purpose**: Developer onboarding for the React+TypeScript horoscope SPA  
**Feature**: `008-horoscope-ui-polish`  
**Branch**: `008-horoscope-ui-polish`

---

## Prerequisites

- Node.js 18+ and npm 9+
- Rust/WASM toolchain already set up (WASM is pre-built; no Rust changes in this feature)
- Repository cloned at `/astro/shubharambham-ai`

---

## Step 1 — Install React + MUI Dependencies

From the repository root:

```bash
# devDependencies
npm install --save-dev \
  @vitejs/plugin-react@^4.2.0 \
  @types/react@^18.2.0 \
  @types/react-dom@^18.2.0

# runtime dependencies
npm install \
  react@^18.3.0 \
  react-dom@^18.3.0 \
  @mui/material@^6.0.0 \
  @emotion/react@^11.11.0 \
  @emotion/styled@^11.11.0
```

---

## Step 2 — Update `tsconfig.json`

Add the following to `compilerOptions` and `include`:

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

---

## Step 3 — Update `vite.config.ts`

Add the React plugin and second entry point:

```typescript
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

// In plugins array, prepend:
react()

// In build.rollupOptions.input:
{
  main: resolve(__dirname, 'web/index.html'),
  horoscope: resolve(__dirname, 'web/horoscope/index.html'),
}
```

---

## Step 4 — Create `web/horoscope/` Directory Structure

```
web/horoscope/
  index.html             # App shell: Noto Fonts link, window.Module init, astro.js script
  main.tsx               # React mount: createRoot(document.getElementById('root'))
  emscripten.d.ts        # Ambient Window.Module type declaration
  types.ts               # CityRecord, PlanetaryPosition, HoroscopeResponse, Lang, etc.
  astro-glue.ts          # TypeScript bridge: bridge(), listCities(), getHoroscopePositions()
  App.tsx                # Root component: reads ?lang=, manages wasmReady + result state
  components/
    LanguageSelector.tsx  # MUI Select with options "English" / "తెలుగు"
    BirthForm.tsx         # Date/time input + Location button + Calculate button
    CityModal.tsx         # Full-screen modal: search input + scrollable city list
    ChakraGrid.tsx        # 4×4 South Indian chakra grid; reused for Rasi + Navamsa
    EchoPanel.tsx         # Displays city/timezone confirmation text
    LongitudeTable.tsx    # MUI Table of all 10 planets with longitude detail
```

> **Critical**: Do NOT modify `web/index.html`, `web/astro-glue.js`, `web/data.js`,  
> `web/components.js`, `web/style.css`, or anything under `astro-wasm/`.

---

## Step 5 — Key Implementation Notes

### `web/horoscope/index.html`
Must declare `window.Module` inline BEFORE the `astro.js` script:
```html
<script>
  window.Module = {
    onRuntimeInitialized: function () {
      document.dispatchEvent(new Event('wasm-ready'));
    }
  };
</script>
<script src="/astro.js"></script>
```

### WASM Ready Gate in React
```typescript
// In App.tsx
const [wasmReady, setWasmReady] = useState(false);
useEffect(() => {
  document.addEventListener('wasm-ready', () => setWasmReady(true), { once: true });
}, []);
```

Location field and Calculate button must be `disabled` until `wasmReady === true`.

### Language from URL
```typescript
const lang = (new URLSearchParams(window.location.search).get('lang') === 'te'
  ? 'te' : 'en') as Lang;
```

### South Indian Chakra Grid
Use the `SIGN_CELL` constant in `types.ts` (see `data-model.md`) to map `zodiacNumber` → `{row, col}` in a 4×4 CSS Grid.

---

## Step 6 — Build and Verify

```bash
npm run build
```

Expected output contains:
- `dist/index.html` — existing vanilla page (unchanged)
- `dist/horoscope/index.html` — new React SPA
- `dist/assets/*.js` — React + MUI bundle

---

## Step 7 — Local Development Server

```bash
npm run dev
```

Navigate to:
- `http://localhost:4173/horoscope` — new React SPA
- `http://localhost:4173/` — existing vanilla page (must still work)

---

## Step 8 — Playwright Tests

```bash
npx playwright test
```

Tests are in `tests/astro.spec.ts`. The new feature adds test cases using the `/horoscope` route.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Location field stays disabled forever | `window.Module` declaration missing or after `astro.js` | Move inline `<script>` before `<script src="/astro.js">` |
| `Module._bridge is not a function` | `astro.js` imported as ESM | Change to plain `<script src="/astro.js">`, not `import` |
| Telugu text shows boxes/tofu | Noto Sans Telugu not loaded | Check Google Fonts link tags in `index.html` |
| Vite build fails on `.tsx` | `jsx` not in `tsconfig.json` | Add `"jsx": "react-jsx"` to `compilerOptions` |
| Existing vanilla page broken | `rollupOptions.input` overrides default | Ensure `main` key points to `web/index.html` |
