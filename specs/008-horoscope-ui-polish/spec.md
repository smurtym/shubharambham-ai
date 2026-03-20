# Feature Specification: Horoscope UI Polish

**Feature Branch**: `008-horoscope-ui-polish`  
**Created**: 2026-03-15  
**Status**: Draft  
**Input**: User description: "create a new polished ui page feature with name 'horoscope-ui-polish' — 3 inputs: Date of birth, Time of birth, Location (auto search and modal window to select it). Shows Horoscope, Navamsa chakra in south indian style. Echoes given data along with lat, long, tz. Show longitudes of all bodies."

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Enter birth details and select a city via searchable modal (Priority: P1)

A user arrives at the horoscope page and sees three clearly labelled input fields: Date of Birth, Time of Birth, and Location. They click the Location field and a modal window opens with a live search box. They type a city name (in any supported language), the list filters as they type, and they click a result to select the city. The modal closes and the selected city name is shown in the Location field. The user then clicks "Calculate" — from that moment, the page has everything it needs to generate the chart.

**Why this priority**: Without this input step there is nothing to calculate. It is the minimum viable interaction and must work stand-alone before any chart rendering is attempted.

**Independent Test**: Can be fully tested by opening the page, interacting with the modal, selecting a city, filling in date and time, and confirming the Calculate button becomes active — without any chart rendering needed.

**Acceptance Scenarios**:

1. **Given** the horoscope page is open, **When** the user clicks the Location field, **Then** a modal window appears containing a search box and a scrollable list of all cities in the current display language.
2. **Given** the modal is open, **When** the user types a partial city name, **Then** the city list filters in real time to show only cities whose name, region, or country contains the typed text (case-insensitive).
3. **Given** the filtered list is showing, **When** the user clicks a city entry, **Then** the modal closes and the city name is displayed in the Location field; the city's internal identifier is stored for the calculation.
4. **Given** all three inputs are filled (date, time, city), **When** the user clicks "Calculate", **Then** the calculate action is triggered and no validation errors are shown.
5. **Given** the modal is open, **When** the user presses Escape or clicks outside the modal, **Then** the modal closes without changing the previously selected city.
6. **Given** the user has not yet selected a city and clicks "Calculate", **Then** the Location field is highlighted as required and no calculation is triggered.

---

### User Story 2 — View South Indian Rasi (birth) chakra (Priority: P2)

After submitting valid birth details, the user sees a South Indian style Rasi horoscope chart rendered on the page. The grid is a 4×4 layout with the centre 2×2 area used for the chart label. Each of the 12 zodiac signs occupies one of the 12 border cells in the fixed South Indian arrangement. Planet abbreviations appear in the cell of the sign they occupy. The Ascendant's cell is marked distinctly.

**Why this priority**: The Rasi chakra is the core deliverable of a horoscope page. Once the input story works, rendering this chart is the next most valuable step and constitutes a usable product on its own.

**Independent Test**: Can be fully tested by submitting a known reference birth chart and verifying that each planet abbreviation appears in the correct house cell.

**Acceptance Scenarios**:

1. **Given** a successful calculation, **When** the Rasi chakra is rendered, **Then** a 4×4 South Indian grid is displayed with 12 border cells and a 2×2 centre area labelled "Rasi".
2. **Given** the chart is rendered, **When** the user inspects the 12 cells, **Then** each cell is assigned to a fixed zodiac sign in the standard South Indian layout (Sagittarius top-left, Capricorn top-second, Aquarius top-third, Pisces top-right; left column downward: Scorpio, Libra; right column downward: Aries, Taurus; bottom row: Virgo, Leo, Cancer, Gemini).
3. **Given** the chart is rendered, **When** the user looks at each planet, **Then** the planet's two-letter abbreviation appears in the cell of the zodiac sign it occupies, matching its computed sidereal sign.
4. **Given** the Ascendant sign, **When** the cell for that sign is rendered, **Then** it is visually distinguished (e.g., a diagonal corner mark or distinct border style) to indicate it is the Lagna.

---

### User Story 3 — View South Indian Navamsa (D9) chakra (Priority: P3)

Alongside the Rasi chart, the user sees a second identically structured South Indian grid showing the Navamsa (D9) chart. Each planet is placed in its Navamsa sign cell. The user can visually compare both charts side by side.

**Why this priority**: The Navamsa is the second most important chart in Vedic astrology. Adding it immediately after the Rasi chart doubles the interpretive value of the page with minimal additional effort.

**Independent Test**: Can be fully tested by verifying that each planet abbreviation in the Navamsa grid matches the navamsa sign number returned by the calculation engine for the same reference birth chart.

**Acceptance Scenarios**:

1. **Given** a successful calculation, **When** the page is rendered, **Then** both the Rasi and Navamsa chakras are visible simultaneously, each labelled with its chart type ("Rasi" / "Navamsa").
2. **Given** the Navamsa chart is rendered, **When** the user inspects it, **Then** each planet's abbreviation is in the cell corresponding to its Navamsa sign number as returned by the engine.
3. **Given** the same reference birth, **When** a planet occupies a different sign in the Navamsa than in the Rasi, **Then** the planet's abbreviation appears in different cells in the two charts.

---

### User Story 4 — View echoed inputs and resolved coordinates (Priority: P4)

Below the charts, the user sees a summary panel confirming what was submitted: the date of birth, time of birth, selected city name, and the resolved coordinates (latitude, longitude, timezone) for that city. This confirms to the user which location was used for the calculation.

**Why this priority**: The echo panel is a quality-assurance aid — it helps the user confirm that the engine used the intended location. It is non-critical for the core charting experience but builds trust in the output.

**Independent Test**: Can be fully tested without chart rendering: submit a known city, confirm the echo panel shows that city's coordinates and timezone to the expected precision.

**Acceptance Scenarios**:

1. **Given** a successful calculation, **When** the echo panel is visible, **Then** it shows the entered date of birth, time of birth, selected city name, resolved latitude (3 decimal places), resolved longitude (3 decimal places), and the IANA timezone string.
2. **Given** a city like Hyderabad is selected, **When** the echo panel is displayed, **Then** the latitude reads approximately 17.3xx°N, longitude approximately 78.4xx°E, and timezone reads `Asia/Kolkata`.

---

### User Story 5 — View a table of all body longitudes (Priority: P5)

Below the echo panel, the user sees a table listing all ten celestial bodies (Ascendant, Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu) with each body's translated name and its absolute sidereal longitude in decimal degrees to 5 decimal places. The table is clearly labelled and readable on both desktop and mobile widths.

**Why this priority**: The longitude table is a secondary reference view for more advanced users or developers validating the calculation. It is the last layer of information — useful but not essential for the charting experience.

**Independent Test**: Can be tested by comparing each row's longitude value against the raw engine response, confirming values match to 5 decimal places.

**Acceptance Scenarios**:

1. **Given** a successful calculation, **When** the longitude table is rendered, **Then** it contains exactly 10 rows — one for each body — with columns for body name and decimal degree longitude.
2. **Given** the table is rendered, **When** the user checks a planet's longitude, **Then** the displayed value matches the engine's `longitude` field for that body to 5 decimal places.
3. **Given** `lang=te` is active, **When** the table is rendered, **Then** body names in the table are in Telugu script.

---

### Edge Cases

- **No city selected before Calculate**: The form must prevent submission and highlight the Location field — partial input (date + time without location) must not be sent to the engine.
- **WASM not yet ready**: If the user clicks Calculate before the WASM module has finished loading, the button must be disabled or show a loading indicator; the action must not silently fail.
- **Multiple planets in the same sign**: All planets sharing a sign appear in the same chart cell; the cell must accommodate up to 10 abbreviations without clipping.
- **Ascendant and a planet in the same sign**: Both the Ascendant marker and planet abbreviations coexist in the same Rasi cell without critically overlapping.
- **Long city name in Location field**: When the selected city name plus region is too long for the field width, the text is truncated with an ellipsis; the full name is accessible via the modal.
- **Mobile viewport**: Both South Indian chakra grids must remain legible on viewports as narrow as 320px; cells must not clip planet abbreviations.
- **Language switch after calculation**: If the user changes the display language after a chart is rendered, all translated strings (planet names, sign names, city name) in the chakras, echo panel, and longitude table update without requiring a new calculation.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The page MUST present three clearly labelled input fields: Date of Birth (calendar date), Time of Birth (time of day), and Location (displays the selected city name and region).
- **FR-002**: The Location field MUST open a modal window when activated. The modal MUST contain a search text box and a scrollable, filtered list of cities. The list MUST update in real time as the user types — filtering by city name, region, or country — in a case-insensitive manner.
- **FR-003**: Selecting a city from the modal MUST close the modal, display the city's name and region in the Location field, and store the city's identifier for the subsequent calculation. Only one city may be selected at a time.
- **FR-004**: The modal MUST be dismissible without selecting a city by pressing Escape or clicking/tapping outside the modal boundary. Dismissing without selection MUST leave any previously selected city unchanged.
- **FR-005**: A "Calculate" button MUST be present. Clicking it with all three inputs filled MUST invoke the horoscope calculation. Clicking it with any required input missing MUST highlight the missing field(s) and prevent the engine call.
- **FR-006**: On successful calculation, the page MUST render a South Indian style Rasi chakra — a 4×4 grid with 12 border cells fixed to the standard South Indian sign layout and a 2×2 centre area labelled "Rasi". Each planet's translated two-letter abbreviation MUST appear in the cell corresponding to the zodiac sign it occupies. The Ascendant's cell MUST carry a distinct visual marker to identify it as the Lagna.
- **FR-007**: On successful calculation, the page MUST render a South Indian style Navamsa (D9) chakra using the same 4×4 grid layout, labelled "Navamsa". Each planet's abbreviation MUST appear in the cell corresponding to its Navamsa sign number. Both charts MUST be visible simultaneously.
- **FR-008**: On successful calculation, the page MUST display an echo panel showing: the entered date of birth, the entered time of birth, the selected city name with region, the resolved latitude (3 decimal places), the resolved longitude (3 decimal places), and the IANA timezone string for the city.
- **FR-009**: On successful calculation, the page MUST display a table of all ten body longitudes. The table MUST contain one row per body (Ascendant, Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu), with the body's translated name and its absolute sidereal longitude in decimal degrees to 5 decimal places.
- **FR-010**: All translated strings on the page (planet names, zodiac sign names, city name) MUST reflect the currently active display language. Changing the display language after a chart is shown MUST update all translated strings without requiring a new calculation.
- **FR-011**: The Calculate button MUST be disabled or display a loading indicator while the WASM module is initialising. Once the module signals readiness, the button MUST become interactive.
- **FR-012**: While a calculation is in-flight, the page MUST show a visual loading state and MUST prevent duplicate submissions by disabling the Calculate button until the response is received.
- **FR-013**: On calculation error (e.g., engine returns `{"error":"..."}`), the page MUST display the error message to the user in a clearly visible area and MUST NOT render partial chart data.
- **FR-014**: Both chakra grids and the longitude table MUST be legible and fully functional on mobile viewports at 320px wide. No cell content (abbreviations, sign labels) may be obscured or clipped by the grid boundary.

### Key Entities

- **BirthInputs**: The three user-supplied values — date of birth (calendar date), time of birth (time of day), and selected city identifier — that together form a complete horoscope request.
- **CitySearchModal**: The modal UI component with a search text box and a filtered, scrollable city list. State: open/closed, current search query, filtered city list for the active language.
- **RasiChakra**: A South Indian 4×4 grid rendering of the birth (D1) chart. Each of the 12 border cells is permanently assigned to a zodiac sign in the South Indian layout; planets are placed by their sidereal sign number.
- **NavamsaChakra**: A South Indian 4×4 grid rendering of the Navamsa (D9) chart. Same layout as RasiChakra; planets are placed by their Navamsa sign number.
- **EchoPanel**: A read-only summary of the submitted inputs and the resolved geographic values (lat, lng, timezone) for the selected city.
- **LongitudeTable**: A read-only table of all ten celestial bodies and their absolute sidereal longitudes in decimal degrees.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can go from an empty page to a rendered Rasi chakra in 6 or fewer interactions: click Location → type city → select city → enter date → enter time → click Calculate.
- **SC-002**: Given a published reference birth chart, every planet abbreviation in the Rasi chakra appears in the correct sign cell and every abbreviation in the Navamsa chakra appears in the correct Navamsa sign cell — zero misplacements across all ten bodies.
- **SC-003**: The echo panel shows the selected city's resolved latitude and longitude to 3 decimal places, and the correct IANA timezone string — values exactly match what the engine returned.
- **SC-004**: The longitude table shows all 10 body longitudes with no absent rows; each displayed value matches the engine's `longitude` field to 5 decimal places.
- **SC-005**: On a 320px wide mobile viewport, both chakra grids are fully visible and all planet abbreviations within each cell are readable without horizontal scrolling or clipping.
- **SC-006**: Switching the display language after rendering a chart updates all translated strings (planet names, sign names in chakras, city name in echo panel, body names in longitude table) within the same page view, with no `[missing]` placeholder appearing.

## Assumptions

- The existing `listCities(lang)` and `horoscopePositions(lang, cityId, localTime)` WASM bridge functions are used as-is; no new engine capabilities are required for this feature.
- The South Indian 4×4 cell layout uses the fixed sign-to-cell assignment: top row (columns 1–4) = Sagittarius, Capricorn, Aquarius, Pisces; left column (rows 2–3) = Scorpio, Libra; right column (rows 2–3) = Aries, Taurus; bottom row (columns 1–4) = Virgo, Leo, Cancer, Gemini; centre 2×2 = chart label area.
- Birth time is entered at minute precision (`HH:MM`). Seconds default to `:00` when passed to the engine.
- The page is a standalone HTML page that loads the existing `astro.js` and `astro-glue.js` assets — no new server-side component or separate API is introduced.
- City search filtering is performed client-side over the pre-loaded city list; no per-keystroke network request is made.
- Only one language is active at a time; all visible text switches to that language when the user changes the language selector.
