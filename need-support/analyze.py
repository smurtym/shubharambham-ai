"""
Analyze JHora Vimsottari Dasa output vs our computed durations.

Reads calculated-dasas.txt, parses every start/end timestamp, computes
exact durations in days (as floats), then compares against what our
algorithm would produce using SOLAR_YEAR_DAYS = 365.256363.
"""

from datetime import datetime
from dataclasses import dataclass, field
from typing import List

# ---------------------------------------------------------------------------
# Constants — must mirror vimsottari.rs exactly
# ---------------------------------------------------------------------------

SIDEREAL_YEAR = 365.256363   # days  (changed from tropical 365.2425)

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
TOTAL_YEARS = sum(y for _, y in DASA_SEQUENCE)   # 120

# Map short JHora names → our canonical keys
JHORA_NAME_MAP = {
    "Moon": "Moon", "Mars": "Mars", "Rah": "Rahu", "Jup": "Jupiter",
    "Sat": "Saturn", "Merc": "Mercury", "Ket": "Ketu", "Ven": "Venus",
    "Sun": "Sun",
}

# ---------------------------------------------------------------------------
# Parse the JHora file
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
    # date_str = "2018-03-27"  time_str = "(11:23:05)"
    return datetime.strptime(f"{date_str} {time_str}", DATETIME_FMT)


def parse_file(path: str) -> List[Mahadasa]:
    mahadasas: List[Mahadasa] = []
    current_maha: Mahadasa | None = None

    with open(path) as f:
        for raw in f:
            line = raw.strip()
            if not line:
                continue

            # Mahadasa header:  " Moon MD: 2008-03-26 (21:46:21) - 2018-03-27 (11:23:05)"
            if " MD: " in line:
                parts = line.split()
                lord_short = parts[0]
                lord = JHORA_NAME_MAP.get(lord_short, lord_short)
                # parts: Lord MD: date1 (time1) - date2 (time2)
                start = parse_dt(parts[2], parts[3])
                end   = parse_dt(parts[5], parts[6])
                current_maha = Mahadasa(lord=lord, start=start, end=end)
                mahadasas.append(current_maha)
                continue

            # Antardasa line: "  Moon: 2008-03-26 (21:46:21) - 2009-01-26 (10:57:58)"
            if current_maha and ": 20" in line and " - " in line:
                parts = line.split()
                lord_short = parts[0].rstrip(":")
                lord = JHORA_NAME_MAP.get(lord_short, lord_short)
                start = parse_dt(parts[1], parts[2])
                end   = parse_dt(parts[4], parts[5])
                current_maha.antardasas.append(Antardasa(lord=lord, start=start, end=end))

    return mahadasas


# ---------------------------------------------------------------------------
# Expected durations from our algorithm
# ---------------------------------------------------------------------------

def expected_maha_days(lord_key: str) -> float:
    for name, years in DASA_SEQUENCE:
        if name == lord_key:
            return years * SIDEREAL_YEAR
    raise ValueError(f"Unknown lord: {lord_key}")


def expected_antar_days(maha_lord_key: str, antar_lord_key: str) -> float:
    maha_full = expected_maha_days(maha_lord_key)
    for name, years in DASA_SEQUENCE:
        if name == antar_lord_key:
            return maha_full * years / TOTAL_YEARS
    raise ValueError(f"Unknown antar lord: {antar_lord_key}")


# ---------------------------------------------------------------------------
# Report
# ---------------------------------------------------------------------------

SEP = "-" * 90

def fmt_diff(actual: float, expected: float) -> str:
    diff = actual - expected
    sign = "+" if diff >= 0 else ""
    return f"{sign}{diff:+.6f} d  ({sign}{diff*24*60:+.2f} min)"


def run(path: str):
    mahadasas = parse_file(path)

    print(SEP)
    print("VIMSOTTARI DASA — JHora actual vs our expected durations")
    print(f"Sidereal year used: {SIDEREAL_YEAR} days")
    print(SEP)

    grand_match = grand_mismatch = 0

    for maha in mahadasas:
        exp_maha = expected_maha_days(maha.lord)
        act_maha = maha.actual_days
        diff_maha = act_maha - exp_maha
        status = "✓" if abs(diff_maha) < 0.5 else "✗"

        print(f"\n{'='*90}")
        print(f"  MAHADASA: {maha.lord:10s}  |  "
              f"actual={act_maha:.6f} d  expected={exp_maha:.6f} d  "
              f"diff={diff_maha:+.6f} d  {status}")
        print(f"  Start: {maha.start}   End: {maha.end}")
        print(f"{'='*90}")
        print(f"  {'Lord':10s}  {'Actual (days)':>16s}  {'Expected (days)':>16s}  "
              f"{'Diff (days)':>14s}  {'Diff (min)':>12s}  Status")
        print(f"  {'-'*80}")

        if abs(diff_maha) < 0.5:
            grand_match += 1
        else:
            grand_mismatch += 1

        for ad in maha.antardasas:
            exp_ad  = expected_antar_days(maha.lord, ad.lord)
            act_ad  = ad.actual_days
            diff_ad = act_ad - exp_ad
            ok = "✓" if abs(diff_ad) < 0.5 else "✗"
            if abs(diff_ad) < 0.5:
                grand_match += 1
            else:
                grand_mismatch += 1
            print(f"  {ad.lord:10s}  {act_ad:>16.6f}  {exp_ad:>16.6f}  "
                  f"{diff_ad:>+14.6f}  {diff_ad*24*60:>+12.2f}  {ok}")

        print()

    print(SEP)
    print(f"SUMMARY: {grand_match} matched  |  {grand_mismatch} mismatched  "
          f"|  total {grand_match+grand_mismatch} antardasa+mahadasa rows")
    print(SEP)


if __name__ == "__main__":
    import os
    here = os.path.dirname(os.path.abspath(__file__))
    run(os.path.join(here, "calculated-dasas.txt"))
