# Tasks: Horoscope UI Polish

**Input**: Design documents from `specs/008-horoscope-ui-polish/`
**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md) | **Contract**: [contracts/ts-wasm-bridge-v1.md](contracts/ts-wasm-bridge-v1.md)
**Data Model**: [data-model.md](data-model.md) | **Quickstart**: [quickstart.md](quickstart.md)

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no unresolved dependencies)
- **[Story]**: Which user story this task belongs to (US1–US5)
- Exact file paths are given in each task description
- **No test tasks**: Tests not explicitly requested in spec

---

## Phase 1: Setup

**Purpose**: Install runtime dependencies; configure Vite multi-page and TypeScript JSX; create the `web/horoscope/` HTML shell and React mount point.

- [ ] T001 Install React, MUI, Emotion, and plugin dependencies: `npm install --save-dev @vitejs/plugin-react@^4.2.0 @types/react@^18.2.0 @types/react-dom@^18.2.0 && npm install react@^18.3.0 react-dom@^18.3.0 @mui/material@^6.0.0 @emotion/react@^11.11.0 @emotion/styled@^11.11.0`
- [ ] T002 [P] Update `tsconfig.json`: add `"jsx": "react-jsx"` and `"moduleResolution": "bundler"` to `compilerOptions`; extend `"lib"` to include `"DOM"` and `"DOM.Iterable"`; add `"web/**/*.ts"` and `"web/**/*.tsx"` to the `include` array
- [ ] T003 [P] Update `vite.config.ts`: import `react` from `'@vitejs/plugin-react'` and prepend it to the plugins array; add `build.rollupOptions.input` with `main: resolve(__dirname, 'web/index.html')` and `horoscope: resolve(__dirname, 'web/horoscope/index.html')`; add `import { resolve } from 'path'`
- [ ] T004 [P] Create `web/horoscope/index.html`: HTML5 shell with Noto Sans + Noto Sans Telugu Google Fonts `<link>` tags, inline `<script>` declaring `window.Module = { onRuntimeInitialized: () => document.dispatchEvent(new Event('wasm-ready')) }` placed BEFORE `<script src="/astro.js">`, `<div id="root"></div>` body, and `<script type="module" src="./main.tsx">` Vite entry tag
- [ ] T005 [P] Create `web/horoscope/main.tsx`: import `React`, `ReactDOM`; call `ReactDOM.createRoot(document.getElementById('root')!).render(<React.StrictMode><App /></React.StrictMode>)`; import `App` from `'./App'`

**Checkpoint**: `npm run build` must succeed with no errors before Phase 2 begins (horoscope entry compiles even as a stub).

---

## Phase 2: Foundational

**Purpose**: Core TypeScript contracts, WASM bridge, and the App shell with WASM-ready gate. All user story components depend on these.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T006 [P] Create `web/horoscope/emscripten.d.ts`: ambient `declare global` block extending `Window` with `Module: { _malloc, _bridge, _free, HEAPU8, onRuntimeInitialized? }` typed exactly as in `contracts/ts-wasm-bridge-v1.md`; export `{}`
- [ ] T007 [P] Create `web/horoscope/types.ts`: export `Lang`, `CityRecord`, `PlanetaryPosition`, `HoroscopeResponse`, `WasmError`, `AppState`, `BirthFormState`, `CityModalState`, `ChakraGridProps`, `LongitudeTableProps`, `EchoPanelProps`, `CityModalProps`, `LanguageSelectorProps` — all fields from `data-model.md`; export `SIGN_CELL` constant with the 12-entry mapping from `data-model.md` (zodiacNumber 1–12 → `{row, col}`)
- [ ] T008 Create `web/horoscope/astro-glue.ts`: implement `bridge(op, input)` using `window.Module._malloc` / `HEAPU8.set` / `_bridge` / `_free` with a `finally` block; implement `listCities(lang)` and `getHoroscopePositions(cityId, localTime, lang)` wrapping `bridge()`; throw `WasmError` on negative return codes; buffer sizes 512 KiB for `list_cities`, 64 KiB for `horoscope_positions` — as specified in `contracts/ts-wasm-bridge-v1.md`
- [ ] T009 Create `web/horoscope/App.tsx`: read `lang` from `?lang=` URL param (default `'en'`); manage `wasmReady` state via `document.addEventListener('wasm-ready', ...)` with `{ once: true }` in `useEffect`; render `<LanguageSelector current={lang} />`; render a loading indicator (`CircularProgress`) while `wasmReady === false`; hold `result: HoroscopeResponse | null` and `error: string | null` state; conditionally render result area when `result` is not null
- [ ] T010 [P] Create `web/horoscope/components/LanguageSelector.tsx`: MUI `Select` (or `FormControl`+`Select`) with two `MenuItem` options — value `'en'` labelled `"English"`, value `'te'` labelled `"తెలుగు"` (native script only); on `onChange`, navigate to `window.location.href` with the updated `?lang=` param (full page reload); render prominently at the top of the page; touch target ≥ 44 px height (FR-016)

**Checkpoint**: `web/horoscope/App.tsx` compiles, shows a loading indicator on page load and switches to ready state once `wasm-ready` fires (verifiable in browser dev tools).

---

## Phase 3: User Story 1 — Birth details input + city search modal (Priority: P1) 🎯 MVP

**Goal**: A user can open the page, search for a city in a modal, fill date and time, and trigger a calculation — without any chart needing to render.

**Independent Test**: Open `/horoscope`, click Location (disabled until WASM ready), type a city name, select a result, fill Date and Time, click Calculate — confirm the Calculate button fires the engine call (visible in network/console) and no validation errors block it.

### Implementation for User Story 1

- [ ] T011 [P] [US1] Create `web/horoscope/components/CityModal.tsx`: MUI `Dialog` (full-screen on mobile); contains a `TextField` search input; on mount, calls `listCities(lang)` to populate the city list; filters list client-side as the user types (case-insensitive match on `cityName`, `region1`, `region2`); renders a `List` of `ListItem`+`ListItemButton` rows (≥44 px height per FR-016); shows `"No cities found"` `Typography` when query is non-empty and no cities pass the filter; closes on row click (calls `props.onSelect(city)`), on Escape, and on backdrop click (`props.onClose`); accepts `CityModalProps` from `types.ts`
- [ ] T012 [P] [US1] Create `web/horoscope/components/BirthForm.tsx`: renders three MUI inputs — `TextField` for Date of Birth (type `date`, FR-001), `TextField` for Time of Birth (type `time`, FR-001), `TextField` for Location (read-only, shows `cityDisplayName`, onClick opens CityModal); the Location field and Calculate `Button` are `disabled` when `props.wasmReady === false` (FR-011); Calculate `Button` is also `disabled` when `props.loading === true` (FR-012); clicking Calculate with any input empty highlights the missing field(s) with MUI `error` prop and calls nothing (FR-005); on valid Calculate click calls `props.onCalculate(cityId, localTime)`; touch targets ≥ 44 px for all controls (FR-016); accepts `BirthFormState`-derived props
- [ ] T013 [US1] Wire User Story 1 state into `web/horoscope/App.tsx`: add `cityId`, `cityDisplayName`, `localTime`, `modalOpen`, `loading` state; pass `wasmReady`, `loading`, city/date/time values, and callbacks (`onCitySelect`, `onDateChange`, `onTimeChange`, `onCalculate`) to `<BirthForm>`; in each `onChange` handler clear `result` and `error` to `null` immediately (FR-015); in `onCalculate` assemble `localTime = dateStr + 'T' + timeStr + ':00'` (seconds default to `:00` per spec Assumptions), set `loading=true`, call `getHoroscopePositions(cityId, localTime, lang)` from `astro-glue.ts`, on success set `result`, on `WasmError` set `error` message (FR-013), in `finally` set `loading=false`; render a non-null `error` in a visible MUI `Alert` component

**Checkpoint [US1]**: The three-input form is fully functional. A city can be selected, date and time entered, Calculate pressed. The engine call is made. All edge cases work: WASM-loading gate, in-flight guard, missing-field validation, "No cities found", input-clears-result.

---

## Phase 4: User Story 2 — South Indian Rasi chakra (Priority: P2)

**Goal**: After a successful calculation, the Rasi (D1) chart renders as a 4×4 South Indian grid with planets in the correct sign cells.

**Independent Test**: Submit the reference birth chart — `cityId=466083476` (Hyderabad, Telangana), `localTime='2026-03-20T15:05:00'`, `lang='en'`. Verify Rasi placements: Lagna marker on Cancer cell {row:1,col:3}; Sun/Moon/Venus/Saturn in Pisces {row:0,col:0}; Mars/Mercury/Rahu in Aquarius {row:1,col:0}; Jupiter in Gemini {row:0,col:3}; Ketu in Leo {row:2,col:3}. Verify Navamsa placements: Lagna in Sagittarius {row:3,col:0}; Sun/Ketu in Leo {row:2,col:3}; Moon/Venus in Capricorn {row:2,col:0}; Mars in Pisces {row:0,col:0}; Mercury/Rahu in Aquarius {row:1,col:0}; Jupiter in Aries {row:0,col:1}; Saturn in Virgo {row:3,col:3}. Confirm the Ascendant cell (Cancer) carries a distinct visual marker.

### Implementation for User Story 2

- [ ] T014 [P] [US2] Create `web/horoscope/components/ChakraGrid.tsx`: render a 4×4 MUI `Box` CSS Grid (`gridTemplateColumns: 'repeat(4, 1fr)'`, `gridTemplateRows: 'repeat(4, 1fr)'`); place a centre cell spanning `gridRow: '2/4', gridColumn: '2/4'` with the chart `title` label; for each of the 12 sign positions (using `SIGN_CELL` from `types.ts`), render a `Box` cell placed with `gridRow` and `gridColumn` (1-indexed); when `props.navamsa === false`, group planets by `zodiacNumber`; when `props.navamsa === true`, group by `navamsaZodiacNumber`; render planet `abbrev` strings inside each cell — multiple planets displayed stacked vertically; for Rasi mode apply a distinct visual marker (e.g., `borderLeft: '3px solid'` in a contrasting colour) to the cell whose sign number matches the Ascendant's `zodiacNumber`; each cell must accommodate up to 10 abbreviations without clipping (overflow scroll or wrap); accepts `ChakraGridProps` from `types.ts`
- [ ] T015 [US2] Wire Rasi chakra into `web/horoscope/App.tsx` result area: when `result` is not null render `<ChakraGrid planets={result.planets} title="Rasi" navamsa={false} />`; place the chakra below `<BirthForm>` in document flow

**Checkpoint [US2]**: Rasi chakra renders correctly for a known reference chart. All 10 abbreviations are in their expected sign cells with zero misplacements (SC-002).

---

## Phase 5: User Story 3 — South Indian Navamsa (D9) chakra (Priority: P3)

**Goal**: The Navamsa chart appears alongside the Rasi chart; both charts are visible simultaneously and use the same grid component.

**Independent Test**: Submit the same reference birth chart. Verify each planet abbreviation in the Navamsa grid matches its `navamsaZodiacNumber`. Verify the layout is side-by-side at 768px viewport and stacked at 375px viewport.

### Implementation for User Story 3

- [ ] T016 [US3] Add Navamsa chakra and responsive two-chart layout to `web/horoscope/App.tsx`: wrap both chakra grids in an MUI `Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}`; render `<ChakraGrid planets={result.planets} title="Navamsa" navamsa={true} />` as the second chart; each grid takes equal width in row mode (`flex: 1`) and full width in column mode; ensure no horizontal scrolling appears at 320px (SC-005, FR-007)

**Checkpoint [US3]**: Both Rasi and Navamsa charts render side-by-side on ≥600px and stack vertically on <600px. Navamsa placements match engine `navamsaZodiacNumber` for all 10 bodies.

---

## Phase 6: User Story 4 — Echo panel (Priority: P4)

**Goal**: A summary panel below the charts confirms the date, time, city name, latitude, longitude, and timezone that were used in the calculation.

**Independent Test**: Submit Hyderabad (`cityId=466083476`). Confirm echo panel shows lat ≈ 17.xxx°, lng ≈ 78.xxx° to 3 decimal places and timezone `Asia/Kolkata`.

### Implementation for User Story 4

- [ ] T017 [P] [US4] Create `web/horoscope/components/EchoPanel.tsx`: accepts `EchoPanelProps` (`response: HoroscopeResponse`, `dateOfBirth: string`, `timeOfBirth: string`); renders an MUI `Paper` or `Card` containing: date of birth, time of birth, `response.cityName` + `response.region1`, `response.lat.toFixed(3)` latitude, `response.lng.toFixed(3)` longitude, and `response.timezone` IANA string — matching FR-008 and SC-003 exactly
- [ ] T018 [US4] Render `<EchoPanel>` in `web/horoscope/App.tsx` result area: pass `response={result}`, `dateOfBirth` and `timeOfBirth` from form state; place below the chakra grids

**Checkpoint [US4]**: Echo panel displays all six fields with the correct precision. Values match engine response exactly (SC-003).

---

## Phase 7: User Story 5 — Longitude table (Priority: P5)

**Goal**: A table below the echo panel lists all 10 bodies with their absolute sidereal longitudes to 5 decimal places.

**Independent Test**: Submit reference chart. Confirm 10 rows are present, body names are translated per active `lang`, and each longitude value matches `planet.longitude.toFixed(5)` from the engine response.

### Implementation for User Story 5

- [ ] T019 [P] [US5] Create `web/horoscope/components/LongitudeTable.tsx`: accepts `LongitudeTableProps` (`planets: Record<string, PlanetaryPosition>`); renders an MUI `Table` with `TableHead` (columns: "Body", "Longitude") and `TableBody` with one `TableRow` per planet in canonical order (Ascendant, Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu); each row shows `planet.name` (translated by WASM) in the first cell and `planet.longitude.toFixed(5)` in the second cell; table is responsive — no horizontal scrolling at 320px (FR-014); matching FR-009 and SC-004
- [ ] T020 [US5] Render `<LongitudeTable>` in `web/horoscope/App.tsx` result area: pass `planets={result.planets}`; place below `<EchoPanel>`

**Checkpoint [US5]**: Longitude table shows exactly 10 rows. Each longitude matches engine `longitude` field to 5 decimal places. Telugu body names render correctly when `?lang=te` (SC-006, SC-004).

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Verify touch targets, mobile layout, build correctness, and end-to-end dev flow.

- [ ] T021 [P] Audit all interactive elements in `web/horoscope/components/` for 44×44 px minimum touch targets (FR-016): Apply MUI `sx={{ minHeight: 44, minWidth: 44 }}` or `py` padding where needed to Location field trigger, CityModal list rows, Calculate button, language selector dropdown, and modal close areas
- [ ] T022 [P] Validate 320px mobile viewport (SC-005): Open `/horoscope` at 320px width in browser; confirm Rasi and Navamsa chakras stack vertically, each spans full width, no abbreviation is clipped inside cells, no horizontal scrollbar appears; adjust `ChakraGrid.tsx` cell sizing if needed
- [ ] T023 Run `npm run build` and verify both outputs: `dist/index.html` (existing vanilla page, unchanged) and `dist/horoscope/index.html` (new React SPA); fix any TypeScript strict-mode errors (`noImplicitAny`, `strictNullChecks`) in `web/horoscope/` files; verify `dist/` produces a separate JS chunk for the horoscope entry — confirm no monolithic initial bundle spanning both entry points (constitution II code-splitting mandate)
- [ ] T024 Run quickstart.md end-to-end validation: follow all 8 steps from `quickstart.md`; confirm `/horoscope` loads with Noto fonts, WASM ready state fires, full chart renders for a known birth date, and `?lang=te` renders Telugu labels without any `[missing]` placeholder (SC-006)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — **BLOCKS all user stories**
- **User Stories (Phases 3–7)**: All depend on Phase 2 completion
  - US1 (Phase 3) → US2 (Phase 4) → US3 (Phase 5) must be done in this order (each builds on App.tsx result area from the previous)
  - US4 (Phase 6) and US5 (Phase 7) depend on US1 but are independent of US2/US3
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Depends only on Phase 2 — no dependency on other stories
- **US2 (P2)**: Depends on US1 being complete (result state must exist in App.tsx)
- **US3 (P3)**: Depends on US2 being complete (ChakraGrid.tsx must exist)
- **US4 (P4)**: Depends on US1 being complete (result state must exist); independent of US2/US3
- **US5 (P5)**: Depends on US1 being complete (result state must exist); independent of US2/US3/US4

### Within Each User Story

- Files marked [P] within a story can be written simultaneously (different files)
- Wire-up tasks are sequential — they depend on the component files existing
- App.tsx changes (T013, T015, T016, T018, T020) must be done in order as each builds on the previous result area

---

## Parallel Example: Phase 2 (Foundational)

```bash
# These 4 tasks can start simultaneously (all different files):
T006: Create web/horoscope/emscripten.d.ts
T007: Create web/horoscope/types.ts
T008: Create web/horoscope/astro-glue.ts   ← reads types.ts, can be parallel if types.ts written first
T010: Create web/horoscope/components/LanguageSelector.tsx

# Then sequentially:
T009: Create web/horoscope/App.tsx          ← imports LanguageSelector + uses types
```

## Parallel Example: User Story 1 (Phase 3)

```bash
# These two can be written simultaneously (different files):
T011: Create web/horoscope/components/CityModal.tsx
T012: Create web/horoscope/components/BirthForm.tsx

# Then sequentially (depends on both above):
T013: Wire US1 state into web/horoscope/App.tsx
```

## Parallel Example: User Stories 4 & 5 (Phases 6–7)

```bash
# US4 and US5 independent component files can overlap with Phase 4/5 work:
T017: Create web/horoscope/components/EchoPanel.tsx     [P] with T019
T019: Create web/horoscope/components/LongitudeTable.tsx [P] with T017

# Then App.tsx wire-up sequentially:
T018: Render EchoPanel in App.tsx
T020: Render LongitudeTable in App.tsx
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T005)
2. Complete Phase 2: Foundational (T006–T010) — CRITICAL blocker
3. Complete Phase 3: User Story 1 (T011–T013)
4. **STOP and VALIDATE**: The city search modal, form validation, WASM gate, and engine call all work. No chart needed.
5. Demo: Open `/horoscope`, search a city, fill date/time, click Calculate, confirm engine call succeeds.

### Incremental Delivery

1. Setup + Foundational → App shell compiles and WASM loads
2. US1 → Interactive form + engine call works (MVP!)
3. US2 → Rasi chakra renders correctly
4. US3 → Navamsa chakra added, responsive layout works
5. US4 → Echo panel confirms inputs
6. US5 → Longitude table completes the detail view
7. Polish → Mobile touch, 320px verification, build check

### Suggested MVP Scope

Implement **US1 only** (T001–T013) for a fully demonstrable MVP:
- User can select a city, enter birth details, trigger a calculation
- WASM loading gate + in-flight guard + validation all function
- Delivers a working engine integration with visible result (even if chart is not yet rendered)

---

## Task Summary

| Phase | Tasks | Count | Parallelisable |
|-------|-------|-------|---------------|
| Phase 1: Setup | T001–T005 | 5 | T002, T003, T004, T005 |
| Phase 2: Foundational | T006–T010 | 5 | T006, T007, T010 |
| Phase 3: US1 | T011–T013 | 3 | T011, T012 |
| Phase 4: US2 | T014–T015 | 2 | T014 |
| Phase 5: US3 | T016 | 1 | — |
| Phase 6: US4 | T017–T018 | 2 | T017 |
| Phase 7: US5 | T019–T020 | 2 | T019 |
| Phase 8: Polish | T021–T024 | 4 | T021, T022 |
| **Total** | | **24** | **11 parallelisable** |
