<!--
SYNC IMPACT REPORT
==================
Version change: 1.0.0 → 2.0.0 (MAJOR: backward-incompatible redefinition of
Principle II — "Lean Web Presentation" removed; replaced with
"React + TypeScript Presentation Layer")

Modified principles:
  - Principle II: "Lean Web Presentation (NON-NEGOTIABLE)"
    → "React + TypeScript Presentation Layer (NON-NEGOTIABLE)"
    Vanilla JS, no-framework, no-library mandate fully replaced with
    TypeScript + React 18 + Material UI + Noto Sans Telugu font mandate.

Added sections: None

Removed sections: None

Templates reviewed:
  ✅ .specify/templates/plan-template.md — No changes needed; Constitution Check
     section is populated dynamically per-feature; no vanilla JS references.
  ✅ .specify/templates/spec-template.md — No changes needed; generic template
     compatible with all six principles.
  ✅ .specify/templates/tasks-template.md — No changes needed; "[framework]"
     placeholder is generic and already maps to React/TypeScript.
  ✅ .github/prompts/*.md — Speckit workflow drivers; no project-specific
     technology references requiring update.

Follow-up TODOs:
  - Any existing feature plans (plan.md) that have a Constitution Check referencing
    the old Principle II ("no frameworks") MUST be re-reviewed against v2.0.0 and
    their Complexity Tracking updated. Feature specs themselves are unaffected.
  - The existing vanilla-JS web/ source files (index.html, astro-glue.js,
    components.js, data.js, style.css) are legacy artifacts. Feature 008 and all
    subsequent features MUST be implemented using the new React + TypeScript stack.
    Migration of earlier features is left to a dedicated migration task.
-->

# Shubharambham AI Constitution

## Core Principles

### I. Rust-First Computation (NON-NEGOTIABLE)

All business logic, astronomical calculations, and localization MUST be implemented
in Rust and compiled to WebAssembly (WASM). The JavaScript layer MUST NOT perform
any calculation, data transformation, or i18n logic.

- The Emscripten toolchain MUST be used for WASM compilation to support Swiss
  Ephemeris virtual filesystem embedding.
- Swiss Ephemeris (`swisseph`) is the sole permitted library for astronomical
  computation; no approximation or alternative ephemeris is acceptable.
- Rust crates added to the project MUST be justified; prefer `no_std`-compatible
  or low-footprint crates to keep the WASM binary small.

### II. React + TypeScript Presentation Layer (NON-NEGOTIABLE)

The presentation layer MUST be implemented in TypeScript using React 18 or later
as the sole UI component framework.

- **React 18+** is the ONLY permitted UI framework. Vue, Angular, Svelte, and all
  other frameworks are excluded.
- **Material UI (MUI) v6+** is the ONLY permitted component and styling library.
  No other CSS frameworks, component libraries, or inline third-party stylesheets
  are permitted.
- **Noto Sans Telugu** MUST be loaded as the primary font to ensure correct
  rendering of Telugu script across all platforms and browsers. Latin glyphs MUST
  also fall back to a Noto Sans variant for visual consistency.
- **TypeScript** (strict mode) MUST be used for all source files in the
  presentation layer. Plain `.js` files in new features are not permitted.
- **Vite** is the ONLY permitted build/bundler tool for the React frontend.
- The React/TypeScript layer MUST NOT perform any calculation, data
  transformation, or i18n logic — these remain exclusively in Rust/WASM.
- The React layer MUST only: invoke WASM exports, receive returned data
  structures, and render them into React components using MUI primitives.
- Bundle size MUST be managed consciously. Code-splitting MUST be applied at the
  route or feature boundary to avoid a monolithic initial load. The WASM payload
  is excluded from this constraint.

### III. Contract-Driven WASM API

Every function exported from the Rust WASM module to JavaScript constitutes a
documented contract and MUST be specified before implementation begins.

- Contracts MUST define: function name, parameter types, return type, error
  codes, and localization keys returned.
- Contracts MUST be versioned. Breaking changes to a contract increment the
  contract's MAJOR version and require a migration plan.
- A contract definition file MUST live under `specs/<feature>/contracts/` before
  any implementation task may be started.
- JS callers MUST NOT inspect or manipulate internal WASM memory layouts beyond
  the published contract surface.

### IV. Mobile-First Design

All pages and UI components MUST be designed and tested for mobile browsers first
(Chrome, Firefox, and Safari on mobile), then adapted for desktop viewports.

- Layouts MUST be responsive using CSS only (no JS-driven layout logic).
- Touch targets MUST meet a minimum 44 × 44 px interactive area.
- No desktop-only interaction patterns (hover-only tooltips, right-click menus)
  are permitted as primary UI affordances.
- The website is NOT a Progressive Web App and MUST NOT register service workers
  or prompt for installation.

### V. Correctness & Accuracy of Vedic Calculations

Vedic astrology calculations MUST produce results consistent with established
reference values derived from Swiss Ephemeris.

- Every calculation module in Rust MUST have unit tests that compare output
  against known reference birthcharts or published Panchangam data.
- Ayanamsha selection and coordinate systems MUST be explicitly documented in
  code and in the relevant contract file.
- No rounding or truncation of intermediate astronomical values is permitted
  without documented justification.

### VI. Localization in Rust

All internationalization and localization logic MUST reside entirely within the
Rust WASM module.

- Supported locales at launch: `en` (English) and `te` (Telugu).
- The JS layer MUST pass only a locale identifier to WASM; WASM returns
  pre-rendered, locale-correct strings ready for DOM insertion.
- String resources, numeral systems, and transliterations MUST be compiled into
  the WASM binary — no runtime fetch of locale files is permitted.
- Adding a new locale is a minor constitution-compatible change and requires
  only new Rust locale data and an updated contract.

## Technology Stack Constraints

- **Computation**: Rust (stable toolchain) compiled to WASM via Emscripten.
- **Ephemeris**: Swiss Ephemeris (`swisseph`) C library, wrapped in Rust via FFI,
  with ephemeris data files embedded in the Emscripten virtual filesystem.
- **Presentation**: TypeScript (strict mode), React 18+, Material UI (MUI) v6+.
  Noto Sans Telugu (and Noto Sans for Latin) MUST be loaded as the primary
  typeface — via self-hosting or Google Fonts CDN.
- **Build**: Emscripten (`emcc`) for WASM compilation; **Vite** for the React/TS
  frontend bundle. No other bundlers (webpack, rollup standalone, parcel) are
  permitted for the frontend.
- **Testing**: `cargo test` for Rust unit/integration tests; Playwright for
  browser-based end-to-end tests covering the React + WASM integration layer.
- **Versioning**: Semantic versioning (`MAJOR.MINOR.PATCH`) for the WASM module
  and for each WASM API contract independently.
- **Deployment target**: Static file hosting (Vite build output + WASM assets);
  no server-side runtime required.

## Development Workflow

1. **Spec first**: A feature specification (`spec.md`) and WASM contract definitions
   (`contracts/`) MUST exist and be reviewed before any implementation begins.
2. **Constitution check gate**: Every plan (`plan.md`) MUST include a Constitution
   Check section confirming compliance with all six principles before Phase 0.
3. **Test before ship**: Rust calculation modules MUST have passing reference-value
   tests. New tests MUST be written/approved before implementation code is merged.
4. **Payload audit**: Before any feature is marked complete, the combined WASM +
   React bundle payload size MUST be measured and compared against the baseline.
   Significant regressions MUST be justified. Code-splitting is the primary
   mitigation strategy.
5. **Contract review**: Any change to a published WASM API contract (parameter
   names, types, return shape) MUST be treated as a breaking change and follow the
   versioning policy in Principle III.
6. **Locale completeness**: A feature is not shippable if English strings are
   present but Telugu strings are absent (or vice versa).
7. **TypeScript strict**: All new `.tsx`/`.ts` files MUST pass TypeScript strict
   mode with zero type errors. `any` casts MUST be documented with a comment
   explaining why the type boundary cannot be expressed statically.

## Governance

This constitution supersedes all other project-level practices and conventions.
Any conflict between this document and a feature spec, task list, or plan is
resolved in favour of the constitution.

**Amendment procedure**:
- Amendments MUST be proposed as a pull request modifying this file.
- The version line MUST be incremented according to semantic versioning rules.
- Principle removals or redefinitions that are backward-incompatible increment
  MAJOR; new principles or sections increment MINOR; clarifications increment PATCH.
- The Sync Impact Report (HTML comment at the top of this file) MUST be updated
  with every amendment.

All feature plans and pull requests MUST verify compliance with each principle
before merge. Complexity or deviations MUST be explicitly justified in plan.md
under the Complexity Tracking section.

**Version**: 2.0.0 | **Ratified**: 2026-03-14 | **Last Amended**: 2026-03-20
