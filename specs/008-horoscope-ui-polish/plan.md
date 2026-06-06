# Implementation Plan: 008 — Horoscope UI Polish

**Branch**: `008-horoscope-ui-polish` | **Date**: 2026-03 | **Spec**: [specs/008-horoscope-ui-polish/spec.md](spec.md)
**Input**: Feature specification from `specs/008-horoscope-ui-polish/spec.md`

---

## Summary

Build a React 18 + TypeScript + MUI v6 horoscope SPA at `/horoscope` as a second Vite entry point alongside the existing vanilla JS page at `/`. The SPA provides a birth-details form (date/time + city selection), displays the South Indian chakra grid (Rasi + Navamsa) and a longitude table for all 10 planets, and supports English and Telugu via a `?lang=` URL parameter.

All computation stays in the existing Emscripten WASM module (`public/astro.js`). The new code calls the same two WASM bridge operations (`list_cities`, `horoscope_positions`) through a typed TypeScript wrapper (`astro-glue.ts`). **No changes to `astro-wasm/` or `public/astro.js`.**

---

## Technical Context

**Language/Version**: TypeScript 5.x strict; React 18.3; MUI 6.0
**Primary Dependencies**: React 18, MUI v6, Emotion, `@vitejs/plugin-react`, Vite 5
**Storage**: N/A — all state is in-memory; city data served by WASM
**Testing**: Playwright (`tests/astro.spec.ts`), targeting `/horoscope` route
**Target Platform**: Mobile browsers (Chrome/Firefox/Safari Android); 320px minimum viewport
**Project Type**: SPA sub-page within an existing static Vite multi-page build
**Performance Goals**: WASM loads within 3 s on mobile; chart renders within 500 ms of Calculate press
**Constraints**: WASM binary (`astro-wasm/`, `public/astro.js`) MUST NOT be modified; existing `web/index.html` and vanilla JS files MUST remain untouched
**Scale/Scope**: Single-screen SPA; 10 planets, 2 charts, 1 table

---

## Constitution Check

*GATE: All principles satisfied. No violations.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I — WASM Core (NON-NEGOTIABLE) | ✅ PASS | No Rust/WASM changes. All computation stays in WASM. |
| II — React + TypeScript Presentation (NON-NEGOTIABLE) | ✅ PASS | React 18, MUI v6, Noto Sans Telugu, TypeScript strict, Vite 5. |
| III — Contract-First API Design | ✅ PASS | `contracts/ts-wasm-bridge-v1.md` documents the TypeScript typed wrapper. Underlying WASM API (`horoscope-api-v1.md`) is unchanged. |
| IV — Mobile-First UX | ✅ PASS | Vertical stack ≤ 599 px, side-by-side ≥ 600 px. 44×44 px touch targets. Touch-primary design. |
| V — Calculation Integrity | ✅ PASS | No calculation logic in the React layer. All output is directly from WASM response; UI only renders returned strings. |
| VI — Localisation in WASM | ✅ PASS | All planet names, sign names, nakshatra names returned pre-localised by WASM. React only passes `lang` to WASM and renders the response. |

**Post-design re-check**: Principles I, III, V, VI verified — no new WASM operations, bridge contract exposes only existing ops.

---

## Project Structure

### Documentation (this feature)

```text
specs/008-horoscope-ui-polish/
├── plan.md                         # This file
├── research.md                     # Phase 0: Emscripten+Vite, MUI, fonts, grid layout
├── data-model.md                   # Phase 1: TypeScript types + SIGN_CELL constant
├── quickstart.md                   # Phase 1: Developer onboarding
├── contracts/
│   └── ts-wasm-bridge-v1.md        # Phase 1: TypeScript bridge interface contract
└── tasks.md                        # Phase 2 (speckit.tasks — not yet created)
```

### Source Code (repository root)

```text
# Vite configuration (update — NOT new files)
vite.config.ts          # Add @vitejs/plugin-react + horoscope entry in rollupOptions.input
tsconfig.json           # Add jsx, moduleResolution, DOM lib, include web/**/*.tsx

# Existing vanilla JS (UNTOUCHED)
web/
├── index.html          # UNTOUCHED
├── astro-glue.js       # UNTOUCHED
├── data.js             # UNTOUCHED
├── components.js       # UNTOUCHED
└── style.css           # UNTOUCHED

# New React SPA (all new files)
web/horoscope/
├── index.html                  # Shell: Noto Fonts, window.Module init, /astro.js script tag
├── main.tsx                    # createRoot mount
├── emscripten.d.ts             # Ambient Window.Module declaration
├── types.ts                    # Lang, CityRecord, PlanetaryPosition, HoroscopeResponse, SIGN_CELL
├── astro-glue.ts               # TypeScript bridge: bridge(), listCities(), getHoroscopePositions()
├── App.tsx                     # Root: ?lang= parsing, wasmReady state, result/error state
└── components/
    ├── LanguageSelector.tsx    # MUI Select; labels "English" / "తెలుగు"
    ├── BirthForm.tsx           # Date/time + Location button + Calculate button
    ├── CityModal.tsx           # Full-screen modal: search + city list
    ├── ChakraGrid.tsx          # 4×4 South Indian grid (reused for Rasi + Navamsa)
    ├── EchoPanel.tsx           # City/timezone echo line
    └── LongitudeTable.tsx      # MUI Table: planet detail rows

# Tests (add test cases)
tests/
└── astro.spec.ts               # Add /horoscope route test cases
```

**Structure Decision**: Vite multi-page app. Single Vite config, two HTML entries. The new React SPA lives entirely under `web/horoscope/` and is self-contained. Existing `web/*.{html,js,css}` files are untouched.

---

## Complexity Tracking

*No Constitution Check violations. Section not applicable.*
