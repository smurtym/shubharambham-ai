# Feature Specification: Vimsottari Dasa Display

**Feature Branch**: `010-vimsottari-ui`  
**Created**: 2026-03-21  
**Status**: Draft  
**Input**: User description: "vimsottari-ui — Display all vimsottari dasa data that we get from vimsottari engine. This will be shown at the bottom on horoscope page that we currently have. All mahadasas are accordions, and when they expand, they show antaradasa."

## User Scenarios & Testing *(mandatory)*

### User Story 1 — View Mahadasa periods after birth chart calculation (Priority: P1)

After calculating a birth chart, an astrologer sees a "Vimsottari Dasa" section appear automatically at the bottom of the horoscope page. The section lists all Mahadasa periods (up to 9) as collapsed accordion rows. Each row shows the Mahadasa lord's translated name, its start date, and its end date — giving an immediate overview of the complete dasa timeline.

**Why this priority**: This is the primary deliverable. Without the list of Mahadasas rendered, the feature has no value. The accordion headers alone, even without expandability, constitute a usable read-only summary.

**Independent Test**: Fill in birth details for a known chart, press Calculate, then scroll to the bottom. Verify that a labeled "Vimsottari Dasa" section appears containing one row per Mahadasa (9 for a full chart, fewer for a partial first Mahadasa), each row displaying the Mahadasa name, start date, and end date.

**Acceptance Scenarios**:

1. **Given** a valid birth chart has been calculated, **When** the user scrolls to the bottom of the horoscope page, **Then** a "Vimsottari Dasa" section is visible containing a list of up to 9 Mahadasa accordion rows, each showing the Mahadasa label (e.g., "Mercury Mahadasa"), start date, and end date.
2. **Given** no birth chart has been calculated yet, **When** the user views the horoscope page, **Then** the Vimsottari Dasa section is not shown.
3. **Given** `lang=te` is active, **When** the section is rendered, **Then** all Mahadasa labels appear in Telugu script (e.g., "బుధ మహాదశ") and dates use Telugu month names.
4. **Given** `lang=en` is active, **When** the section is rendered, **Then** all Mahadasa labels are in English (e.g., "Mercury Mahadasa") and dates use English month names.
5. **Given** a valid birth chart has been calculated, **When** the Vimsottari Dasa section first renders, **Then** the Mahadasa whose date range contains today's date is automatically pre-expanded, and within it the Antardasa whose date range contains today's date is visually highlighted (e.g., with an accent colour or bold text).

---

### User Story 2 — Expand a Mahadasa to view its Antardasas (Priority: P1)

An astrologer taps or clicks a Mahadasa accordion row to expand it. The row opens to reveal all Antardasa sub-periods within that Mahadasa, each showing the Antardasa lord's translated name, start date, and end date. Only one (or more) accordions can be expanded at the same time — the user can collapse it by tapping again.

**Why this priority**: The accordion expand behavior is the core interaction of this feature. Together with User Story 1, it forms the full deliverable as described by the user.

**Independent Test**: Click any Mahadasa accordion row. Verify that it expands to show a list of Antardasa entries. Each entry must have a label (e.g., "Sun Antardasa"), a start date, and an end date. Click the row again; verify it collapses.

**Acceptance Scenarios**:

1. **Given** the Vimsottari Dasa section is visible, **When** the user clicks/taps a Mahadasa accordion row, **Then** the row expands to reveal all Antardasas within that Mahadasa, each showing an Antardasa label, start date, and end date.
2. **Given** a Mahadasa accordion is expanded, **When** the user clicks/taps it again, **Then** the row collapses and the Antardasas are hidden.
3. **Given** a Mahadasa accordion is expanded, **When** the user expands a second Mahadasa, **Then** both are visible simultaneously (multiple-expand allowed).
4. **Given** the first (partial) Mahadasa at birth, **When** expanded, **Then** only the remaining post-birth Antardasas are shown; no pre-birth Antardasa entries appear.
5. **Given** `lang=te` is active and a Mahadasa is expanded, **When** Antardasas are displayed, **Then** all Antardasa labels appear in Telugu script (e.g., "సూర్య అంతర్దశ").

---

### User Story 3 — Graceful loading and error states for Dasa section (Priority: P2)

While the Vimsottari Dasa data is being fetched from the WASM engine, the section shows a loading indicator. If the fetch fails, a descriptive error message is shown inside the section without affecting the rest of the horoscope page (chakra grids, longitude table, echo panel remain unaffected).

**Why this priority**: Robustness is secondary but important. The horoscope page already works; this story ensures the dasa section degrades gracefully rather than breaking the page.

**Independent Test**: Simulate a failed dasa fetch (e.g., invalid city) and verify the rest of the horoscope page remains usable with an error shown only in the dasa section.

**Acceptance Scenarios**:

1. **Given** a birth chart has just been calculated, **When** the vimsottari dasa fetch is in progress, **Then** the Vimsottari Dasa section shows a loading indicator and no partial data.
2. **Given** the vimsottari dasa fetch completes successfully, **Then** the loading indicator disappears and the Mahadasa accordion list is rendered.
3. **Given** the vimsottari dasa fetch returns an error, **Then** an error message is shown inside the Vimsottari Dasa section; the rest of the horoscope page (grids, table, echo panel) remains fully usable.

---

### Edge Cases

- What happens when the birth chart calculation succeeds but the subsequent dasa fetch fails? The dasa section shows an error independently; the rest of the page is unaffected.
- What happens when the first Mahadasa is nearly fully elapsed at birth (very few days remain)? The accordion for that Mahadasa may show only one or two Antardasas — this is expected and must not cause a layout error.
- How does the section behave when the user changes input and re-calculates? The previous dasa data must be cleared (or the section hidden) before the new fetch begins, to avoid stale data being shown while loading.
- What happens on a very small screen where accordion labels are long? Labels must wrap or truncate — the accordion must not overflow horizontally.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: After a successful birth chart calculation (horoscope positions fetch), the application MUST automatically initiate a second fetch for Vimsottari Dasa data using the same `cityId`, `localTime`, and `lang` inputs — without requiring a separate user action.
- **FR-002**: The application MUST call the `vimsottari_dasa` WASM operation through the same bridge already used for horoscope positions. The Rust engine MUST set `is_current: bool` on each `AntardasaEntry` (`true` if today's date falls within that Antardasa's date range) and on each `MahadasaEntry` (derived as `antardasas.iter().any(|a| a.is_current)`). The frontend MUST use these flags directly — no date-comparison or date-parsing logic is performed client-side.
- **FR-003**: The Vimsottari Dasa section MUST be rendered below all existing horoscope components (echo panel, chakra grids, longitude table) on the horoscope page.
- **FR-004**: The section MUST display a translated heading (e.g., "Vimsottari Dasa" in English, "వింశోత్తరి దశ" in Telugu).
- **FR-005**: The section MUST render one accordion item per Mahadasa period returned by the engine. Each accordion header MUST display: the Mahadasa label (already translated by the engine), the start date, and the end date.
- **FR-006**: Each accordion MUST be independently expandable and collapsible. Multiple accordions MAY be open simultaneously.
- **FR-007**: When an accordion is expanded, it MUST display all Antardasa entries returned for that Mahadasa. Each Antardasa row MUST display: the Antardasa label (already translated by the engine), the start date, and the end date.
- **FR-008**: The section MUST NOT be rendered before a birth chart is calculated. It becomes visible only after a successful horoscope positions response is received.
- **FR-009**: While the vimsottari dasa fetch is in progress, the section MUST display a loading indicator. No partial or stale data must be shown.
- **FR-010**: If the vimsottari dasa fetch returns an error, the section MUST display a descriptive error message. The existing horoscope components (chakra grids, longitude table, echo panel) MUST remain fully functional.
- **FR-011**: When the user changes any birth input (date, time, or city) and recalculates, the previous dasa data MUST be cleared before the new fetch is initiated. No stale data from a previous calculation must persist.
- **FR-012**: The UI labels for this section (section heading, any chrome text) MUST be added to the existing `i18n.ts` string table for both English and Telugu. Planet names and Mahadasa/Antardasa period labels (already translated by the WASM engine) must NOT be duplicated in `i18n.ts`.
- **FR-013**: New TypeScript types for the Vimsottari response (`VimsottariResponse`, `MahadasaEntry`, `AntardasaEntry`) MUST be added to `types.ts`. A new `getVimsottariDasa()` function MUST be added to `astro-glue.ts` following the same pattern as `getHoroscopePositions()`.
- **FR-014**: The Vimsottari Dasa section MUST be implemented as a dedicated React component (`VimsottariPanel`) to keep `App.tsx` maintainable.
- **FR-015**: When the section first renders, the `MahadasaEntry` with `isCurrent = true` in the engine response MUST be automatically pre-expanded. Within that Mahadasa, the `AntardasaEntry` with `isCurrent = true` MUST be visually highlighted (e.g., accent colour, bold text, or a distinct background). If no `MahadasaEntry` has `isCurrent = true` (e.g., the chart is for a future birth), all accordions remain collapsed and no Antardasa is highlighted. The `isCurrent` flags are set by the Rust engine — the frontend MUST NOT recompute them from date strings.
- **FR-016**: Each Antardasa entry inside an expanded accordion MUST be rendered as a single compact row: the Antardasa label on the left and the start date and end date on the right, all on one line. Multi-line card or table layouts are not used for Antardasa rows.
- **FR-017**: Every Mahadasa accordion header MUST display only the Mahadasa label and its start–end dates, regardless of whether the Mahadasa is active or not. No Antardasa information is shown inside a collapsed accordion header. (Antardasa detail is already visible via FR-015 pre-expansion of the active Mahadasa.)

### Key Entities

- **VimsottariResponse**: The JSON object returned by the `vimsottari_dasa` operation. Contains echoed city fields (`lang`, `cityId`, `cityName`, `region1`, `region2`, `lat`, `lng`, `timezone`) and a `periods` array of `MahadasaEntry` objects.
- **MahadasaEntry**: One Mahadasa period. Fields: `lord` (canonical planet key, language-independent), `label` (translated string, e.g., "Mercury Mahadasa"), `startDate` (formatted string), `endDate` (formatted string), `isCurrent` (boolean — `true` if any child `AntardasaEntry` is current, derived in Rust as `antardasas.iter().any(|a| a.is_current)`), `antardasas` (array of `AntardasaEntry`).
- **AntardasaEntry**: One Antardasa sub-period within a Mahadasa. Fields: `lord` (canonical planet key), `label` (translated string, e.g., "Sun Antardasa"), `startDate` (formatted string), `endDate` (formatted string), `isCurrent` (boolean — `true` if today's date falls within this period's date range, set by the Rust engine via `chrono::Local::now()`).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After pressing Calculate for a valid birth chart, the Vimsottari Dasa section appears at the bottom of the page without requiring a page reload or a second button press. The section is visible within the same page load cycle as the chakra grids.
- **SC-002**: The accordion list renders one item per Mahadasa returned by the engine. For a standard full-cycle chart, exactly 9 accordion items are present, each with a non-empty label, start date, and end date.
- **SC-003**: Clicking any accordion expands it to show all Antardasa entries for that Mahadasa. Each Antardasa entry has a non-empty label, start date, and end date. Clicking again collapses it.
- **SC-004**: With `lang=te`, all Mahadasa and Antardasa labels in the section are in Telugu script. With `lang=en`, all labels are in English. No `[missing]` or untranslated placeholder text appears.
- **SC-005**: When a fetch error occurs for the dasa section, the error message is contained within the Vimsottari Dasa section. The chakra grids, longitude table, and echo panel continue to display the previously loaded horoscope data correctly.
- **SC-006**: After changing any birth input and pressing Calculate again, the dasa section clears and shows a fresh loading state before displaying the updated dasa periods — no data from the previous calculation remains visible.

## Assumptions

- The WASM bridge's `bridge()` function in `astro-glue.ts` can call `vimsottari_dasa` using the same mechanism as `horoscope_positions` — no bridge changes are needed.
- The Rust `vimsottari_dasa` engine sets `is_current: bool` on each `AntardasaEntry` and `MahadasaEntry` based on `chrono::Local::now().date_naive()` at the time of the WASM call. Since the engine runs in-browser via Emscripten, this reflects the user's local system clock — the same source JavaScript's `Date.now()` would use.
- All Mahadasa period labels (e.g., "Mercury Mahadasa") and Antardasa labels (e.g., "Sun Antardasa") are already translated by the WASM engine and arrive as ready-to-display strings. The frontend does not need to translate planet names itself.
- Date strings (e.g., "2026 March 21") are already formatted by the WASM engine in the requested language and are rendered verbatim by the frontend.
- The Material UI `Accordion` component is available via the installed MUI v6+ dependency and is the appropriate choice for this feature's interaction model. It is not currently used by any existing component but requires no additional installation.
- The vimsottari dasa fetch is initiated immediately after a successful horoscope positions fetch, in parallel or sequentially — the exact timing is an implementation detail, but from the user's perspective the section should appear without a second explicit action.
- Buffer size for the `vimsottari_dasa` bridge call may need to be larger than the `BUF_HOROSCOPE` constant (64 KiB) given the volume of period data (9 Mahadasas × 9 Antardasas × 4 fields each). A dedicated buffer constant should be defined in `astro-glue.ts`.

## Clarifications

### Session 2026-03-21

- Q: Should the active Mahadasa (containing today's date) be highlighted or pre-expanded on load? → A: Active Mahadasa pre-expanded AND its currently running Antardasa also highlighted.
- Q: How should each Antardasa row be laid out inside an expanded Mahadasa? → A: Compact single row — label on the left, start date and end date on the right, inline.
- Q: Should the collapsed Mahadasa accordion header show the current Antardasa name? → A: No — header shows only Mahadasa label and dates; Antardasa detail is already accessible via the pre-expanded active Mahadasa (FR-015).
