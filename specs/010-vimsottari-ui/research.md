# Research: Vimsottari Dasa Display (010-vimsottari-ui)

**Generated**: 2026-03-21  
**Source**: Existing codebase analysis — no external research needed. All unknowns resolved from code inspection.

---

## Q1: Do we need a larger WASM bridge buffer for the vimsottari_dasa response?

**Decision**: Reuse the existing `BUF_HOROSCOPE = 65536` (64 KiB) constant — but define a named alias for clarity.

**Rationale**: Maximum response size estimate: 9 Mahadasas × 9 Antardasas × ~120 bytes/entry ≈ ~10 KiB. City echo fields ≈ 300 bytes. Total worst case ≈ ~11 KiB, well under 64 KiB. No separate larger buffer is needed.

**Alternatives considered**: A dedicated 128 KiB constant was considered but rejected — the payload calculation shows 64 KiB has an 85% safety margin. A named constant `BUF_VIMSOTTARI = BUF_HOROSCOPE` is used anyway for readability, so the value can be changed independently later without hunting for magic numbers.

---

## Q2: How should MUI Accordion be configured for multiple-expand behaviour?

**Decision**: Render each `<Accordion>` as an independent uncontrolled component. Use React state (`Set<number>`) to track which accordion indices are expanded. Control each with `expanded` + `onChange` props.

**Rationale**: MUI `Accordion` used without a wrapping `AccordionGroup` is inherently independent — multiple can be open simultaneously by default when each manages its own state. Using a controlled pattern (explicit `expanded` + `onChange`) is preferred here because we need to pre-expand the active Mahadasa (FR-015) on first render. An uncontrolled accordion cannot be seeded with an initial expanded state programmatically after mount.

**Implementation pattern**:
```tsx
const [expanded, setExpanded] = useState<Set<number>>(() => {
  const active = periods.findIndex(p => isActive(p));
  return active >= 0 ? new Set([active]) : new Set();
});

<Accordion
  expanded={expanded.has(i)}
  onChange={() => setExpanded(prev => {
    const next = new Set(prev);
    next.has(i) ? next.delete(i) : next.add(i);
    return next;
  })}
>
```

**Alternatives considered**: `disableGutters` + uncontrolled — not used because pre-expand on mount requires controlled state. A single-expand accordion (`onChange` closes others) — rejected; spec FR-006 explicitly allows multiple open.

---

## Q3: Where should active-period detection (today's Mahadasa/Antardasa) be computed?

**Decision**: In the Rust engine. The `vimsottari_dasa` handler adds `is_current: bool` to `AntardasaEntry` and `MahadasaEntry`, computed via `chrono::Local::now().date_naive()` at call time. The frontend reads the flag directly — no date parsing on the client.

**Rationale**: Two options were evaluated:

- **Option A (client-side)**: Parse the engine's `"YYYY MonthName DD"` strings in TypeScript using `dayjs` + `customParseFormat` and compare against `Date.now()`. Rejected because: (1) date strings use localised month names (`"March"` in English, `"మార్చి"` in Telugu) — any mismatch silently produces `Invalid Date`, breaking active-period detection for Telugu users; (2) splits the "what is current" concern across two layers; (3) requires `customParseFormat` plugin wiring.

- **Option B (Rust-side flag)**: Rust engine sets `is_current` using `chrono::Local::now()` in the JSON response. Chosen because: (1) eliminates locale parsing fragility entirely; (2) the entity that produced the date ranges also determines currentness — correct ownership per Rust-first principle (Constitution I); (3) mutual exclusivity guaranteed (exactly one `AntardasaEntry` has `is_current = true`); (4) frontend is reduced to a boolean field read.

**Rust-side computation**:

```rust
// In build_antardasas() — after computing start_date and end_date:
let today = chrono::Local::now().date_naive();
let is_current = start_date <= today && today < end_date;
entries.push(AntardasaEntry { lord: ..., label: ...,
    start_date: format_date(start_date, lang),
    end_date:   format_date(end_date, lang),
    is_current,
});

// In build_periods() — after calling build_antardasas():
let is_current = antardasas.iter().any(|a| a.is_current);
periods.push(MahadasaEntry { lord: ..., label: ...,
    start_date: format_date(start_date, lang),
    end_date:   format_date(end_date, lang),
    is_current,
    antardasas,
});
```

**Frontend usage** (no date parsing needed):

```typescript
// Find active Mahadasa index:
const activeIdx = dasa.periods.findIndex(p => p.isCurrent);

// Highlight active Antardasa:
const antarActive = antar.isCurrent;
```

**Trade-off**: The `is_current` flag makes the response time-dependent (same inputs → different output on different days). This is intentional and expected — the response is not designed to be cached.

---

## Q4: Where exactly should the VimsottariPanel be triggered in App.tsx?

**Decision**: Trigger the dasa fetch **sequentially after** the horoscope positions fetch inside `handleCalculate()` — not in parallel.

**Rationale**: `bridge()` in `astro-glue.ts` is **synchronous** (uses `_malloc`/`_bridge`/`_free` — all synchronous WASM calls). `getHoroscopePositions` is declared `async` but contains no real `await`; it returns an already-resolved Promise. Using `Promise.all` would *appear* to run them in parallel but actually executes them sequentially in the same JS event loop tick, which is misleading.

More importantly, the WASM module shares a single heap and the underlying Swiss Ephemeris C library has **global mutable state** (`swe_set_ephe_path`, `swe_set_sid_mode`, internal planet tables). While the JS+WASM single-threaded model prevents true concurrent execution today, using `Promise.all` with synchronous WASM calls obscures this dependency and would become unsafe if the calls were ever made truly async (e.g., moved to a WebWorker). Sequential awaits correctly express the intent and are safe regardless of execution model.

If the horoscope fetch fails, the dasa fetch is skipped entirely. The dasa section's loading/error state is independent from the main error display.

**Implementation pattern**:
```tsx
async function handleCalculate() {
  setLoading(true);
  setError(null);
  setResult(null);
  setDasa(null);
  setDasaError(null);
  try {
    const res = await getHoroscopePositions(cityId!, localTime, lang);
    setResult(res);
    // Dasa fetch is separate — its error does not affect the horoscope result
    setDasaLoading(true);
    try {
      const dasa = await getVimsottariDasa(cityId!, localTime, lang);
      setDasa(dasa);
    } catch (e: unknown) {
      setDasaError((e as WasmError)?.message ?? String(e));
    } finally {
      setDasaLoading(false);
    }
  } catch (e: unknown) {
    setError((e as WasmError)?.message ?? String(e));
  } finally {
    setLoading(false);
  }
}
```

**Alternative considered**: `Promise.all` — rejected because (1) it falsely implies concurrency, (2) both bridges are synchronous so there is no latency benefit, and (3) shared WASM heap + Swiss Ephemeris global state make true parallelism unsafe if the architecture ever changes.

---

## Q5: What i18n strings need to be added to i18n.ts?

**Decision**: Add one new key: `vimsottariDasa` (section heading).

| Key | English | Telugu |
|-----|---------|--------|
| `vimsottariDasa` | `'Vimsottari Dasa'` | `'వింశోత్తరి దశ'` |

All Mahadasa labels, Antardasa labels, and date strings come pre-translated from the WASM engine — they are rendered verbatim. No additional i18n keys needed.

---

## Summary: All NEEDS CLARIFICATION resolved

| # | Unknown | Resolution |
|---|---------|-----------|
| Q1 | Buffer size for vimsottari_dasa | 64 KiB sufficient; use `BUF_VIMSOTTARI` alias |
| Q2 | MUI multi-expand accordion pattern | Controlled with `Set<number>` state; seed active index on mount |
| Q3 | Active-period detection placement | Rust engine sets `is_current: bool` on `AntardasaEntry`/`MahadasaEntry`; frontend reads flag directly |
| Q4 | App.tsx trigger point | Sequential `await` inside `handleCalculate()` (after horoscope positions succeed) |
| Q5 | i18n additions | One key only: `vimsottariDasa` heading |
