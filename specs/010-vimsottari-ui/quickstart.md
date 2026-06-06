# Quickstart: Vimsottari Dasa Display (010-vimsottari-ui)

**Generated**: 2026-03-21

This feature includes a **Rust engine extension (Phase 0)** that must be completed before the React frontend work. After editing the Rust source, run `./build.sh` to rebuild the WASM binary, then start the Vite dev server for frontend work.

---

## Prerequisites

Ensure the following from Feature 009 are already in place:
- WASM build output at `public/astro.js` (the `vimsottari_dasa` operation is registered)
- `specs/009-vimsottari-dasa/contracts/vimsottari-dasa-api-v1.md` — **updated by TR002** (add `is_current` field to Mahadasa and Antardasa field tables; update sample JSON)

**Phase 0 (Rust) must complete before frontend work begins.** After TR001 + TR002, run `./build.sh` from the repo root to rebuild `public/astro.js`.

---

## Development workflow

```bash
# From repo root — start the Vite dev server (hot reload)
cd /astro/shubharambham-ai
npm run dev

# The app will be served at http://localhost:5173
# To test Telugu: http://localhost:5173?lang=te
```

For frontend-only changes (Phases 1–5), hot reload applies immediately. Phase 0 Rust changes require a full `./build.sh` rebuild before dev server changes are visible.

---

## Files to create / modify

| File | Action |
|------|--------|
| `astro-wasm/src/engines/vimsottari.rs` | Add `is_current: bool` to `AntardasaEntry` and `MahadasaEntry`; compute in `build_antardasas()`; derive in `build_periods()`. Run `./build.sh` after. |
| `specs/009-vimsottari-dasa/contracts/vimsottari-dasa-api-v1.md` | Update: document `is_current` field; update sample JSON |
| `web/horoscope/types.ts` | Add `AntardasaEntry`, `MahadasaEntry`, `VimsottariResponse`, `VimsottariPanelProps` (with `is_current: boolean`) |
| `web/horoscope/astro-glue.ts` | Add `BUF_VIMSOTTARI` constant; add `getVimsottariDasa()` function |
| `web/horoscope/i18n.ts` | Add `vimsottariDasa` key to `UIStrings` interface, `en`, and `te` |
| `web/horoscope/App.tsx` | Add `dasa`/`dasaLoading`/`dasaError` state; sequential dasa fetch after horoscope; render `<VimsottariPanel>` |
| `web/horoscope/components/VimsottariPanel.tsx` | **Create new** — accordion list; `is_current`-based pre-expansion and highlighting |

---

## Key implementation snippets

### 1 — `getVimsottariDasa()` in `astro-glue.ts`

```typescript
const BUF_VIMSOTTARI = BUF_HOROSCOPE; // payload ~11 KiB max; alias allows independent tuning later

export async function getVimsottariDasa(
  cityId: number,
  localTime: string,
  lang: Lang
): Promise<VimsottariResponse> {
  return bridge('vimsottari_dasa', {
    operation: 'vimsottari_dasa',
    cityId,
    localTime,
    lang,
  }, BUF_VIMSOTTARI) as VimsottariResponse;
}
```

*(The existing `bridge()` function in `astro-glue.ts` uses `BUF_HOROSCOPE` for all non-`list_cities` calls. Add a new overload or update `bridge()` to accept a buffer size parameter, then pass `BUF_VIMSOTTARI`.)*

### 2 — `is_current` in Rust (`astro-wasm/src/engines/vimsottari.rs`)

The Rust engine sets `is_current` — no date parsing needed in the frontend.

**Extend structs** (`AntardasaEntry` and `MahadasaEntry`):

```rust
struct AntardasaEntry {
    lord:       String,
    label:      String,
    start_date: String,
    end_date:   String,
    is_current: bool,  // true if today falls within this Antardasa's date range
}

struct MahadasaEntry {
    lord:       String,
    label:      String,
    start_date: String,
    end_date:   String,
    is_current: bool,  // true if any child Antardasa is current
    antardasas: Vec<AntardasaEntry>,
}
```

**Compute in `build_antardasas()`** (after computing `start_date` and `end_date`):

```rust
let today = chrono::Local::now().date_naive();
let is_current = start_date <= today && today < end_date;
entries.push(AntardasaEntry { lord: ..., label: ...,
    start_date: format_date(start_date, lang),
    end_date:   format_date(end_date, lang),
    is_current,
});
```

**Derive in `build_periods()`** (after calling `build_antardasas()`):

```rust
let is_current = antardasas.iter().any(|a| a.is_current);
periods.push(MahadasaEntry { lord: ..., label: ...,
    start_date: format_date(start_date, lang),
    end_date:   format_date(end_date, lang),
    is_current,
    antardasas,
});
```

After editing, run `./build.sh` from the repo root to rebuild the WASM binary.

### 3 — Initial expanded state (in `VimsottariPanel.tsx`)

```typescript
const [expanded, setExpanded] = useState<Set<number>>(() => {
  const idx = dasa.periods.findIndex(p => p.is_current);
  return idx >= 0 ? new Set([idx]) : new Set();
});
```

### 4 — Fetch sequencing in `App.tsx`

**Important — do NOT use `Promise.all`**: `bridge()` is synchronous WASM. "Parallel" promises are illusory (JS is single-threaded) and the Swiss Ephemeris C library has shared global state that would be unsafe under true parallelism. Use sequential awaits:

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
    // Dasa fetch is independent — its error does not affect the horoscope result
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

### 5 — Accordion render sketch (in `VimsottariPanel.tsx`)

```tsx
{dasa.periods.map((maha, i) => {
  const mahaActive = maha.is_current;
  return (
    <Accordion
      key={maha.lord}
      expanded={expanded.has(i)}
      onChange={() => setExpanded(prev => {
        const next = new Set(prev);
        next.has(i) ? next.delete(i) : next.add(i);
        return next;
      })}
      disableGutters
    >
      <AccordionSummary expandIcon={<ExpandMoreIcon />}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', width: '100%', flexWrap: 'wrap', gap: 0.5 }}>
          <Typography fontWeight={mahaActive ? 700 : 400} color={mahaActive ? 'primary.main' : 'inherit'}>
            {maha.label}
          </Typography>
          <Typography variant="body2" color="text.secondary">
            {maha.startDate} – {maha.endDate}
          </Typography>
        </Box>
      </AccordionSummary>
      <AccordionDetails sx={{ p: 0 }}>
        {maha.antardasas.map(antar => {
          const antarActive = antar.is_current;
          return (
            <Box
              key={antar.lord}
              sx={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                px: 2,
                py: 0.75,
                flexWrap: 'wrap',
                gap: 0.5,
                bgcolor: antarActive ? 'action.selected' : undefined,
                borderLeft: antarActive ? '3px solid' : '3px solid transparent',
                borderLeftColor: antarActive ? 'primary.main' : 'transparent',
              }}
            >
              <Typography variant="body2" fontWeight={antarActive ? 600 : 400}>
                {antar.label}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                {antar.startDate} – {antar.endDate}
              </Typography>
            </Box>
          );
        })}
      </AccordionDetails>
    </Accordion>
  );
})}
```

### 6 — Section heading i18n key

```typescript
// i18n.ts — add to UIStrings interface:
vimsottariDasa: string;

// en:
vimsottariDasa: 'Vimsottari Dasa',

// te:
vimsottariDasa: 'వింశోత్తరి దశ',
```

---

## Manual verification checklist

After implementing:

1. Calculate a chart for a past date (e.g., Hyderabad, 1997-03-07T20:34:00)
   - [ ] Vimsottari Dasa section appears below longitude table
   - [ ] 9 accordion rows rendered (first may have fewer Antardasas)
   - [ ] The Mahadasa containing today (March 2026) is pre-expanded
   - [ ] The current Antardasa is highlighted with accent border + background
   - [ ] Clicking a collapsed accordion expands it; clicking again collapses it
   - [ ] Two accordions can be open at the same time

2. Repeat with `?lang=te`
   - [ ] Section heading in Telugu: "వింశోత్తరి దశ"
   - [ ] All Mahadasa and Antardasa labels in Telugu script
   - [ ] Date strings in Telugu month names

3. Resize to mobile width (375px)
   - [ ] No horizontal scroll
   - [ ] Long labels wrap instead of overflowing
   - [ ] Accordion headers are at least 44px tall (touchable)
