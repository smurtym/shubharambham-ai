# Tasks: Vimsottari Dasa Display

**Input**: Design documents from `specs/010-vimsottari-ui/`
**Prerequisites**: plan.md ✅ spec.md ✅ research.md ✅ data-model.md ✅ quickstart.md ✅

**Tests**: Not requested — no test tasks generated.

**Scope reminder**: Rust engine extension (Phase 0) + React frontend (Phases 1–5). No new npm packages.

---

## Phase 0: Rust Engine — Add `is_current` Flag (Blocking)

**Purpose**: The Rust `vimsottari_dasa` engine must set `is_current: true` on the currently active `AntardasaEntry` and its parent `MahadasaEntry`, computed at WASM call time via `chrono::Local::now()`. This eliminates all client-side date parsing.

**Blocks**: All subsequent phases — the TypeScript types (Phase 1) and component logic (Phases 3–4) depend on this field being present in the WASM response.

- [ ] TR001 In `astro-wasm/src/engines/vimsottari.rs`: (a) add `is_current: bool` field to the `AntardasaEntry` struct and the `MahadasaEntry` struct; (b) in `build_antardasas()`, after computing `start_date` and `end_date`, compute `let today = chrono::Local::now().date_naive(); let is_current = start_date <= today && today < end_date;` and include the field in the `AntardasaEntry` push; (c) in `build_periods()`, after calling `build_antardasas()`, set `let is_current = antardasas.iter().any(|a| a.is_current);` and include it in the `MahadasaEntry` push.

- [ ] TR002 Update the API contract at `specs/009-vimsottari-dasa/contracts/vimsottari-dasa-api-v1.md`: add `is_current` row to the Mahadasa and Antardasa field tables; add `"isCurrent": true` on the active entry in the sample response JSON. Then run `./build.sh` from the repo root to rebuild the WASM binary. Verify by running `npm run dev`, calculating the reference chart (Hyderabad 1997-03-07T20:34:00), and opening the browser console — the `vimsottari_dasa` response must contain `"isCurrent": true` on exactly one Antardasa and its parent Mahadasa; all other entries must have `"isCurrent": false`.

- [ ] TR003 Add a Rust unit test for the `is_current` date-range computation in `astro-wasm/src/engines/vimsottari.rs`. Extract a pure helper `fn is_period_current(start: NaiveDate, end: NaiveDate, today: NaiveDate) -> bool` (or test the inline logic directly). Test cases: (a) `today` in range → `true`; (b) `today == start_date` (inclusive boundary) → `true`; (c) `today == end_date` (exclusive boundary) → `false`; (d) `today` before range → `false`; (e) `today` after range → `false`. Run `cargo test -p astro-wasm` to confirm all pass.

**Checkpoint**: `./build.sh` succeeds. `cargo test` is green. The `vimsottari_dasa` response contains `"isCurrent": true` on exactly one Antardasa and its parent Mahadasa; all other entries have `"isCurrent": false`. Phase 0 fully complete.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish type foundations that every subsequent task depends on.

- [ ] T001 Add `AntardasaEntry`, `MahadasaEntry`, `VimsottariResponse`, `VimsottariPanelProps` interfaces to `web/horoscope/types.ts`. Include `isCurrent: boolean` in both `AntardasaEntry` and `MahadasaEntry` to mirror the Rust struct fields serialised as `isCurrent` via `#[serde(rename_all = "camelCase")]`.
- [ ] T002 [P] Add `vimsottariDasa` key to `UIStrings` interface, `en` object (`'Vimsottari Dasa'`), and `te` object (`'వింశోత్తరి దశ'`) in `web/horoscope/i18n.ts`

**Checkpoint**: Types and i18n strings are in place — all downstream tasks can reference them without compilation errors.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: WASM bridge function that all UI tasks depend on. Must exist before `App.tsx` or `VimsottariPanel` can be wired up.

- [ ] T003 Add `BUF_VIMSOTTARI = BUF_HOROSCOPE` constant and `getVimsottariDasa(cityId, localTime, lang)` function to `web/horoscope/astro-glue.ts`, following the same pattern as `getHoroscopePositions()`. Extend `bridge()` with an optional third parameter `bufSize: number = BUF_HOROSCOPE` and pass `BUF_VIMSOTTARI` in the `getVimsottariDasa()` call. The existing `getHoroscopePositions()` and `listCities()` callers require no changes — they use the default.

**Checkpoint**: `getVimsottariDasa()` is callable. Foundation ready — `App.tsx` and `VimsottariPanel` can now be implemented.

---

## Phase 3: User Story 1 — View Mahadasa Periods (Priority: P1) 🎯 MVP

**Goal**: After calculating a birth chart, the Vimsottari Dasa section appears at the bottom of the page showing all Mahadasa accordion rows (collapsed by default except the active one).

**Independent Test**: Calculate a chart for Hyderabad 1997-03-07T20:34:00. Scroll to the bottom. Verify a "Vimsottari Dasa" / "వింశోత్తరి దశ" heading appears with 9 accordion rows, each showing a Mahadasa label, start date, and end date. The Mahadasa containing today's date must be pre-expanded.

### Implementation

- [ ] T004 [US1] Create `web/horoscope/components/VimsottariPanel.tsx` — implement the full component shell: accept `VimsottariPanelProps` (`dasa`, `loading`, `error`, `lang`), render the section heading using `t.vimsottariDasa` from `i18n`, render a `CircularProgress` when `loading=true`, render an `Alert` when `error` is set.

- [ ] T005 [US1] No active-period date-parsing helper is required. The Rust engine (Phase 0 / TR001) sets `isCurrent: true` directly on the active `AntardasaEntry` and its parent `MahadasaEntry`. In `VimsottariPanel.tsx`, find the active Mahadasa index with `dasa.periods.findIndex(p => p.isCurrent)` and check the active Antardasa with `antar.isCurrent`. Do not import `dayjs` or `customParseFormat` for this purpose.

- [ ] T006 [US1] Implement the Mahadasa accordion list in `web/horoscope/components/VimsottariPanel.tsx`: initialise `expanded: Set<number>` state seeded with the index of the active Mahadasa (the one where `maha.isCurrent === true`); render one MUI `Accordion` per `dasa.periods` entry with `disableGutters` set on each `Accordion` (prevents MUI's default margin-animation style conflicts); each `AccordionSummary` must show the Mahadasa `label` (bold + `primary.main` colour if `maha.isCurrent`) and `startDate – endDate` on the right; leave `AccordionDetails` empty for now (filled in Phase 4).

- [ ] T007 [US1] Wire `VimsottariPanel` into `web/horoscope/App.tsx`: add `dasa: VimsottariResponse | null`, `dasaLoading: boolean`, `dasaError: string | null` state variables; in `handleCalculate()` call `getVimsottariDasa()` sequentially after `getHoroscopePositions()` succeeds (sequential await — not `Promise.all`; see research.md Q4); clear dasa state at the start of each calculation; reset dasa state also in `clearResult()`; render `<VimsottariPanel dasa={dasa} loading={dasaLoading} error={dasaError} lang={lang} />` below `<LongitudeTable>` inside the `{result && ...}` block, only when `result` is set. **Do not use a non-null assertion (`dasa!`)** — `VimsottariPanelProps.dasa` is `VimsottariResponse | null`; the component guards internally and renders the accordion list only when `!loading && !error && dasa !== null`.

**Checkpoint**: After pressing Calculate, the Vimsottari Dasa section appears with all Mahadasa accordion rows visible and the active one pre-expanded. US1 is fully testable independently.

---

## Phase 4: User Story 2 — Expand Mahadasa to View Antardasas (Priority: P1)

**Goal**: Clicking a Mahadasa accordion reveals its Antardasa sub-periods as compact single-line rows; the active Antardasa is highlighted.

**Independent Test**: Click any collapsed accordion row. Verify it expands to show all Antardasa entries, each as one line with label on the left and `startDate – endDate` on the right. The current Antardasa (within the pre-expanded active Mahadasa) has an accent-coloured left border and `action.selected` background. Click again — collapses. Two accordions can be open at the same time.

### Implementation

- [ ] T008 [US2] Implement `AccordionDetails` content in `web/horoscope/components/VimsottariPanel.tsx`: for each Mahadasa, map over `maha.antardasas` and render a `Box` per entry; layout: `display: flex`, `justifyContent: space-between`, `px: 2`, `py: 0.75`, `flexWrap: wrap`; Antardasa `label` on the left (`Typography variant="body2"`), `startDate – endDate` on the right (`Typography variant="body2" color="text.secondary"`); active Antardasa (where `antar.is_current`) gets `bgcolor: 'action.selected'`, `borderLeft: '3px solid'`, `borderLeftColor: 'primary.main'`, `fontWeight: 600`.

- [ ] T009 [US2] Verify accordion expand/collapse toggle in `web/horoscope/components/VimsottariPanel.tsx`: the `onChange` handler on each `Accordion` must toggle the index in the `expanded` Set — add if absent, delete if present — allowing multiple accordions open simultaneously. Confirm `disableGutters` is set to prevent MUI's default margin animation conflicts.

**Checkpoint**: Full accordion interaction works — expand, collapse, multi-expand, active Antardasa highlighted. US2 independently testable. (US3 loading/error UI was implemented in T004.)

---

## Phase 5: Polish & Verification

**Purpose**: Verify US3 loading/error states (implemented in T004), mobile layout, accessibility, and edge-case correctness across all stories.

- [ ] T010 [US3] Verify loading state in `web/horoscope/components/VimsottariPanel.tsx`: confirm that when `loading=true`, `CircularProgress` (size 20) renders and no accordion list is shown. Confirm `dasaLoading` is set to `true` before `getVimsottariDasa()` is called and cleared in the `finally` block in `App.tsx`.

- [ ] T011 [US3] Verify error state in `web/horoscope/components/VimsottariPanel.tsx`: confirm that when `error` is a non-empty string, `Alert severity="error"` renders and no accordion list is shown. Confirm the surrounding `{result && ...}` block in `App.tsx` means the chakra grids and longitude table are unaffected.

- [ ] T012 [P] Verify mobile layout in `web/horoscope/components/VimsottariPanel.tsx`: confirm `AccordionSummary` min-height is ≥ 44px (MUI default is 48px — verify not overridden); confirm `flexWrap: 'wrap'` on the summary `Box` prevents horizontal overflow on 375px viewport; confirm long Telugu labels wrap correctly on narrow screens.

- [ ] T013 [P] Verify edge case — future birth (no active Mahadasa): if no `MahadasaEntry` has `isCurrent: true` (e.g. a birth date in a future year), `periods.findIndex(p => p.isCurrent)` returns `-1`, the `expanded` Set initialises empty, all accordions start collapsed, and no Antardasa row is highlighted. Confirm the Rust engine returns `"isCurrent": false` for all entries when the entire dasa chart lies in the future.

- [ ] T014 [P] Verify edge case — first partial Mahadasa: for a birth chart where the first Mahadasa is only partially elapsed, confirm the accordion renders only the post-birth Antardasa entries present and no layout errors occur. Test with the 1997-03-07 reference chart to confirm an incomplete Mahadasa renders cleanly.

- [ ] T015 Run the manual verification checklist from `specs/010-vimsottari-ui/quickstart.md` end-to-end in a browser (`npm run dev`): English + Telugu, mobile width 375px, expand/collapse, active period highlighting, error state.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 0 (Rust)**: No dependencies — run first. TR001 → TR002 → TR003 are sequential (TR002 requires TR001 to compile cleanly; TR003 adds unit tests; `./build.sh` in TR002 is the WASM rebuild step). Blocks all other phases.
- **Phase 1 (Setup)**: Depends on Phase 0 complete (TypeScript types must mirror the updated WASM response including `is_current`). T001 and T002 are independent [P].
- **Phase 2 (Foundational)**: Depends on T001 (types must exist for the function signature). Blocks Phases 3–4.
- **Phase 3 (US1)**: Depends on Phase 1 + Phase 2 complete. T004–T006 can proceed in order; T007 depends on T004–T006.
- **Phase 4 (US2)**: Depends on T006 (AccordionDetails is added to the component built in US1). T008–T009 are sequential within the same file.
- **Phase 5 (Polish)**: Depends on Phases 3–4 complete. T010–T011 (US3 verification, depend on T004) and T012–T014 are all independent [P]; T015 runs last.

### Parallel Opportunities

**Phase 1**: T001 and T002 are independent — different files.

**Phase 6**: T012, T013, T014 are all verification tasks on already-implemented code — independent of each other.

### Parallel Example: Phase 1

```
Start
  TR001 [vimsottari.rs — add is_current fields + compute]
  TR002 [contract update + ./build.sh rebuild]
  TR003 [unit tests for is_current logic — cargo test]
         ↓ WASM binary ready + tests green
├── T001 [types.ts — add TS interfaces with is_current]
└── T002 [i18n.ts — add heading strings]
         ↓ both complete
        T003 [astro-glue.ts — bridge function]
         ↓
        T004–T007 [Phase 3 in sequence]
          ↓
        T008–T009 [Phase 4]
          ↓
        T010, T011, T012, T013, T014 [Phase 5 Polish — parallel]
          ↓
        T015 [final end-to-end verification]
```

---

## Implementation Strategy

**MVP scope (Phases 1–3)**: Types + bridge function + VimsottariPanel shell with accordion headers + App.tsx wiring. After Phase 3 the Mahadasa list is visible and the active period is pre-expanded. This alone satisfies US1 and provides immediate user value.

**Increment 2 (Phase 4)**: Add Antardasa rows inside each accordion — completes the core feature interaction (US2).

**Increment 3 (Phase 5)**: Verification of US3 states, mobile layout, and edge-case hardening — completes production readiness.

18 tasks total (TR001–TR003 Rust + T001–T015 frontend) touch 7 files. No new npm packages. The Rust addition is localised to `astro-wasm/src/engines/vimsottari.rs` and requires one `./build.sh` rebuild.
