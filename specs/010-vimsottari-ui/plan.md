# Implementation Plan: Vimsottari Dasa Display

**Branch**: `010-vimsottari-ui` | **Date**: 2026-03-21 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `specs/010-vimsottari-ui/spec.md`

## Summary

Add a Vimsottari Dasa section to the existing horoscope page. After a birth chart is calculated, the app automatically calls the `vimsottari_dasa` WASM operation and renders the result as an accordion list below the longitude table. Each Mahadasa is a collapsible accordion row; expanding reveals its Antardasa sub-periods as compact single-line rows. The Mahadasa with `isCurrent = true` in the engine response is pre-expanded on load, and the active Antardasa within it is highlighted. The Rust `vimsottari_dasa` engine (Feature 009) is extended with `is_current: bool` fields on `AntardasaEntry` and `MahadasaEntry`, computed at call time via `chrono::Local::now()` — eliminating all client-side date parsing.

## Technical Context

**Language/Version**: TypeScript (strict mode), React 18  
**Primary Dependencies**: Material UI (MUI) v6+ — `Accordion`, `AccordionSummary`, `AccordionDetails`, `Typography`, `Box`, `Stack`, `Alert`, `CircularProgress`  
**Storage**: N/A — all data is fetched from the WASM engine per calculation; no persistence  
**Testing**: Playwright (browser e2e) + one Rust unit test for the `is_current` date-range computation (TR003)  
**Target Platform**: Mobile-first web browser (Chrome, Firefox, Safari on mobile); WASM engine already compiled  
**Project Type**: Web application with Rust/WASM backend extension — React frontend + one Rust engine change  
**Performance Goals**: Vimsottari Panel must render within the same user interaction cycle as the chakra grids — no perceptible additional delay beyond the existing horoscope fetch  
**Constraints**: Mobile-first; 44×44 px minimum touch targets on accordion headers; accordion labels must wrap gracefully on narrow viewports; no layout overflow  
**Scale/Scope**: One Rust engine change (`vimsottari.rs`); one new component (`VimsottariPanel`); changes to four existing frontend files (`App.tsx`, `astro-glue.ts`, `types.ts`, `i18n.ts`); one contract update

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design: PASS.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Rust-First Computation | ✅ COMPLIANT | Rust engine extended: `is_current: bool` computed in Rust via `chrono::Local::now().date_naive()` and set on `AntardasaEntry`/`MahadasaEntry`. Active-period detection remains in Rust — no logic moves to the frontend. |
| II. React + TypeScript Presentation Layer | ✅ COMPLIANT | New `VimsottariPanel.tsx` is a React component in strict TypeScript. MUI Accordion used for interaction. No plain JS files added. No calculation or i18n logic in the frontend. |
| III. Contract-Driven WASM API | ✅ COMPLIANT | `vimsottari_dasa` contract extended: `is_current: bool` added to `MahadasaEntry` and `AntardasaEntry` in `specs/009-vimsottari-dasa/contracts/vimsottari-dasa-api-v1.md`. No new WASM operations; contract update included in Phase 0 task TR002. |
| IV. Mobile-First Design | ✅ COMPLIANT | Accordion headers designed with adequate touch targets (min 44px height via MUI defaults + padding). Labels use `flexWrap` to prevent horizontal overflow on narrow screens. |
| V. Correctness of Vedic Calculations | ✅ N/A | No calculation logic. UI renders pre-computed engine output verbatim. |
| VI. Localization in Rust | ✅ COMPLIANT | Mahadasa/Antardasa labels and date strings arrive pre-translated from WASM. Only chrome UI strings (section heading) are added to `i18n.ts`, consistent with existing pattern. |

**No violations. No Complexity Tracking table required.**

## Project Structure

### Documentation (this feature)

```text
specs/010-vimsottari-ui/
├── plan.md           ← this file
├── research.md       ← Phase 0 output
├── data-model.md     ← Phase 1 output
├── quickstart.md     ← Phase 1 output
└── tasks.md          ← Phase 2 output (created by /speckit.tasks)
```

No new contracts directory — this feature consumes the existing contract from Feature 009.

### Source Code (files touched by this feature)

```text
astro-wasm/src/engines/
└── vimsottari.rs                  # Add is_current: bool to AntardasaEntry and MahadasaEntry;
                                   # compute in build_antardasas(); derive in build_periods()

specs/009-vimsottari-dasa/contracts/
└── vimsottari-dasa-api-v1.md     # Update: add is_current field to MahadasaEntry/AntardasaEntry

web/horoscope/
├── App.tsx                        # Add dasa state; call getVimsottariDasa() after
│                                  # horoscope success; render <VimsottariPanel>
├── astro-glue.ts                  # Add BUF_VIMSOTTARI constant; add getVimsottariDasa()
├── types.ts                       # Add VimsottariResponse, MahadasaEntry, AntardasaEntry
│                                  # (with is_current: boolean fields)
├── i18n.ts                        # Add vimsottariDasa heading key (en + te)
└── components/
    └── VimsottariPanel.tsx        # NEW: accordion list + is_current-based highlighting
```

**Structure Decision**: Two-layer change — Rust engine extension (`vimsottari.rs`) + React frontend component (`web/horoscope/`). The new component is placed alongside existing peer components (`ChakraGrid`, `LongitudeTable`, etc.).
