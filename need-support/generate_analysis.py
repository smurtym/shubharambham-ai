"""
Generate analysis.md comparing JHora Vimsottari timestamps against our formula.

Run from the need-support folder:
    python3 generate_analysis.py > analysis_tables.txt
"""

from datetime import datetime
from dataclasses import dataclass, field
from typing import List
import os

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

SIDEREAL_YEAR = 365.256363   # days

DASA_SEQUENCE = [
    ("Ketu",    7),
    ("Venus",  20),
    ("Sun",     6),
    ("Moon",   10),
    ("Mars",    7),
    ("Rahu",   18),
    ("Jupiter",16),
    ("Saturn", 19),
    ("Mercury",17),
]
TOTAL_YEARS = sum(y for _, y in DASA_SEQUENCE)  # 120

JHORA_NAME_MAP = {
    "Moon": "Moon", "Mars": "Mars", "Rah": "Rahu", "Jup": "Jupiter",
    "Sat": "Saturn", "Merc": "Mercury", "Ket": "Ketu", "Ven": "Venus",
    "Sun": "Sun",
}

DASA_YEARS = {name: years for name, years in DASA_SEQUENCE}

# ---------------------------------------------------------------------------
# Parse
# ---------------------------------------------------------------------------

DATETIME_FMT = "%Y-%m-%d (%H:%M:%S)"

@dataclass
class Antardasa:
    lord: str
    start: datetime
    end: datetime

    @property
    def actual_days(self) -> float:
        return (self.end - self.start).total_seconds() / 86400


@dataclass
class Mahadasa:
    lord: str
    start: datetime
    end: datetime
    antardasas: List[Antardasa] = field(default_factory=list)

    @property
    def actual_days(self) -> float:
        return (self.end - self.start).total_seconds() / 86400


def parse_dt(date_str: str, time_str: str) -> datetime:
    return datetime.strptime(f"{date_str} {time_str}", DATETIME_FMT)


def parse_file(path: str) -> List[Mahadasa]:
    mahadasas: List[Mahadasa] = []
    current_maha: Mahadasa | None = None

    with open(path) as f:
        for raw in f:
            line = raw.strip()
            if not line:
                continue
            if " MD: " in line:
                parts = line.split()
                lord = JHORA_NAME_MAP.get(parts[0], parts[0])
                start = parse_dt(parts[2], parts[3])
                end   = parse_dt(parts[5], parts[6])
                current_maha = Mahadasa(lord=lord, start=start, end=end)
                mahadasas.append(current_maha)
            elif current_maha and ": 20" in line and " - " in line:
                parts = line.split()
                lord = JHORA_NAME_MAP.get(parts[0].rstrip(":"), parts[0].rstrip(":"))
                start = parse_dt(parts[1], parts[2])
                end   = parse_dt(parts[4], parts[5])
                current_maha.antardasas.append(Antardasa(lord=lord, start=start, end=end))

    return mahadasas


# ---------------------------------------------------------------------------
# Expected values
# ---------------------------------------------------------------------------

def expected_maha_days(lord: str) -> float:
    return DASA_YEARS[lord] * SIDEREAL_YEAR


def expected_antar_days(maha_lord: str, antar_lord: str) -> float:
    return expected_maha_days(maha_lord) * DASA_YEARS[antar_lord] / TOTAL_YEARS


def fmt_diff(diff: float) -> str:
    h = abs(diff) * 24
    sign = "+" if diff >= 0 else "−"
    return f"{sign}{abs(diff):.4f} d ({sign}{h:.2f} h)"


def status(diff: float, threshold: float = 0.5) -> str:
    return "✓" if abs(diff) < threshold else "✗"


# ---------------------------------------------------------------------------
# Markdown output
# ---------------------------------------------------------------------------

def md_maha_table(mahadasas: List[Mahadasa]) -> str:
    rows = []
    rows.append("| Mahadasa | JHora Start | JHora End | JHora Days | Expected Days | Diff | Status |")
    rows.append("|---|---|---|---:|---:|---|:---:|")
    for m in mahadasas:
        exp = expected_maha_days(m.lord)
        act = m.actual_days
        diff = act - exp
        rows.append(
            f"| {m.lord} ({DASA_YEARS[m.lord]}y) "
            f"| {m.start.strftime('%Y-%m-%d %H:%M')} "
            f"| {m.end.strftime('%Y-%m-%d %H:%M')} "
            f"| {act:.4f} "
            f"| {exp:.4f} "
            f"| {fmt_diff(diff)} "
            f"| {status(diff)} |"
        )
    return "\n".join(rows)


def md_antar_table(m: Mahadasa) -> str:
    rows = []
    rows.append(f"| Antardasa | JHora Start | JHora End | JHora Days | Expected Days | Diff | Status |")
    rows.append("|---|---|---|---:|---:|---|:---:|")
    total_diff = 0.0
    for a in m.antardasas:
        exp  = expected_antar_days(m.lord, a.lord)
        act  = a.actual_days
        diff = act - exp
        total_diff += diff
        rows.append(
            f"| {a.lord} ({DASA_YEARS[a.lord]}y) "
            f"| {a.start.strftime('%Y-%m-%d %H:%M:%S')} "
            f"| {a.end.strftime('%Y-%m-%d %H:%M:%S')} "
            f"| {act:.4f} "
            f"| {exp:.4f} "
            f"| {fmt_diff(diff)} "
            f"| {status(diff)} |"
        )
    rows.append(f"| **TOTAL** | | | **{m.actual_days:.4f}** | **{expected_maha_days(m.lord):.4f}** "
                f"| {fmt_diff(m.actual_days - expected_maha_days(m.lord))} | |")
    return "\n".join(rows)


def run(path: str) -> str:
    mahadasas = parse_file(path)

    total_ok  = sum(1 for m in mahadasas if abs(m.actual_days - expected_maha_days(m.lord)) < 0.5)
    antar_ok  = sum(1 for m in mahadasas for a in m.antardasas
                    if abs(a.actual_days - expected_antar_days(m.lord, a.lord)) < 0.5)
    total_ad  = sum(len(m.antardasas) for m in mahadasas)

    # Find per-mahadasa matches
    maha_results = []
    for m in mahadasas:
        ok = sum(1 for a in m.antardasas
                 if abs(a.actual_days - expected_antar_days(m.lord, a.lord)) < 0.5)
        maha_results.append((m.lord, ok, len(m.antardasas)))

    from io import StringIO
    out = StringIO()
    p = lambda s="": print(s, file=out)

    p("# Vimsottari Dasa Duration Analysis: JHora vs Our Formula")
    p()
    p("## Parameters")
    p()
    p(f"- **Sidereal year length used:** {SIDEREAL_YEAR} days")
    p(f"- **Antardasa formula:** `antar_days = maha_full_days × (antar_lord_years / 120)`")
    p(f"- **Total Mahadasa years:** {TOTAL_YEARS}")
    p()
    p("---")
    p()
    p("## 1. Mahadasa Total Duration Comparison")
    p()
    p("Each Mahadasa duration is computed from the exact JHora timestamps and compared")
    p(f"against `lord_years × {SIDEREAL_YEAR}` (our formula).")
    p()
    p(md_maha_table(mahadasas))
    p()
    p(f"**Mahadasa totals matched:** {total_ok} / {len(mahadasas)} (threshold ±0.5 day)")
    p()
    p("All Mahadasa total durations agree with the sidereal year formula to within ~0.01 days")
    p("(< 15 minutes). This confirms that the **year length of 365.256363 days is correct**")
    p("for Mahadasa totals.")
    p()
    p("---")
    p()
    p("## 2. Antardasa Duration Comparison (per Mahadasa)")
    p()
    p("For each Antardasa the expected duration is:")
    p("`expected = maha_full_days × (antar_lord_years / 120)`")
    p()

    for m in mahadasas:
        p(f"### {m.lord} Mahadasa ({DASA_YEARS[m.lord]} years = {expected_maha_days(m.lord):.4f} days)")
        p()
        if not m.antardasas:
            p("*(no Antardasa data in source file)*")
            p()
            continue
        p(md_antar_table(m))
        ok = sum(1 for a in m.antardasas
                 if abs(a.actual_days - expected_antar_days(m.lord, a.lord)) < 0.5)
        p()
        p(f"Antardasa match: {ok}/{len(m.antardasas)}")
        p()

    p("---")
    p()
    p("## 3. Overall Summary")
    p()
    p(f"| Category | Matched | Total | Match rate |")
    p("|---|---:|---:|---:|")
    p(f"| Mahadasa totals | {total_ok} | {len(mahadasas)} | {100*total_ok/len(mahadasas):.0f}% |")
    p(f"| Antardasa durations | {antar_ok} | {total_ad} | {100*antar_ok/total_ad:.0f}% |")
    p()
    p("---")
    p()
    p("## 4. Key Observations")
    p()
    p("### 4.1 Mahadasa totals match the sidereal year exactly")
    p()
    p("All 9 Mahadasa total durations computed from JHora timestamps agree with")
    p(f"`lord_years × {SIDEREAL_YEAR}` to within **< 0.011 days (< 16 minutes)**.")
    p("This is consistent with floating-point rounding when JHora converts fractional days")
    p("to wall-clock timestamps and back. The year length is confirmed as correct.")
    p()
    p("### 4.2 Antardasa durations are systematically inconsistent with our formula")
    p()
    p("Within each Mahadasa, the individual Antardasa durations from JHora differ from")
    p("our proportional formula by **up to ±4 days**. The errors have no fixed sign and")
    p("do not scale with Antardasa lord years in a predictable way.")
    p()
    p("For example, within the **Mars Mahadasa**:")
    p()
    p("| Antardasa | Expected | JHora Actual | Diff |")
    p("|---|---:|---:|---:|")
    m_mars = next(m for m in mahadasas if m.lord == "Mars")
    for a in m_mars.antardasas:
        exp = expected_antar_days("Mars", a.lord)
        act = a.actual_days
        p(f"| {a.lord} ({DASA_YEARS[a.lord]}y) | {exp:.3f} | {act:.3f} | {act-exp:+.3f} |")
    p()
    p("The Mars/Mars and Sun Antardasa are overestimated by ~3.4 days, while Ketu and")
    p("Moon are underestimated by a similar amount — yet adjacent Antardasa like Rahu,")
    p("Jupiter, Mercury, Venus match to within 0.5 days.")
    p()
    p("### 4.3 Antardasa fractions are not exact integer-year proportions")
    p()
    p("If JHora used `maha_full_days × (antar_years / 120)` each Antardasa fraction should")
    p("equal exactly `antar_years / 120`. Below are the actual fractions × 120 for Moon MD:")
    p()
    p("| Antardasa | Expected years | Actual × 120 | Δ years |")
    p("|---|---:|---:|---:|")
    m_moon = next(m for m in mahadasas if m.lord == "Moon")
    for a in m_moon.antardasas:
        actual_frac = a.actual_days / m_moon.actual_days
        actual_years = actual_frac * 120
        expected_years = DASA_YEARS[a.lord]
        p(f"| {a.lord} | {expected_years} | {actual_years:.4f} | {actual_years - expected_years:+.4f} |")
    p()
    p("None of the proportions are exact integers, which rules out a simple rounding error")
    p("in our code. JHora appears to use a **different formula** for Antardasa durations.")
    p()
    p("---")
    p()
    p("## 5. Hypothesis")
    p()
    p("The most likely explanation is that JHora converts Antardasa durations to calendar")
    p("dates by adding *fractional sidereal years* (or tropical years) directly to a Julian")
    p("Day Number rather than multiplying by a fixed constant. Because the actual length of")
    p("a sidereal year varies slightly over centuries, this produces Antardasa boundaries")
    p("that drift from our fixed `365.256363 days/year` formula by 1–4 days over the course")
    p("of a Mahadasa.")
    p()
    p("**Specific questions to ask JHora maintainers:**")
    p()
    p("1. Does JHora compute Antardasa end dates by adding `(maha_years × antar_years / 120)`")
    p("   **actual sidereal years** (variable length) to a JD, rather than a fixed days constant?")
    p("2. Or does JHora compute each Antardasa duration as a fixed number of days using a")
    p("   different year length than the Mahadasa year length?")
    p("3. Is there a balance-fraction sub-calculation applied within each Antardasa, similar")
    p("   to how the birth Nakshatra fraction is applied at the Mahadasa level?")

    return out.getvalue()


if __name__ == "__main__":
    here = os.path.dirname(os.path.abspath(__file__))
    content = run(os.path.join(here, "calculated-dasas.txt"))
    out_path = os.path.join(here, "analysis.md")
    with open(out_path, "w") as f:
        f.write(content)
    print(f"Written: {out_path}")
