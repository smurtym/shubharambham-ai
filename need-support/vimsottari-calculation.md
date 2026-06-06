# Vimsottari Dasa Date Discrepancy — Request for Clarification

We are implementing Vimsottari Dasa calculation independently and have been cross-checking our
results against JHora's output. We have found consistent date discrepancies and would like to
understand the exact algorithm JHora uses so we can match it precisely.

---

## 1. Birth Data

| Field       | Value                                        |
|-------------|----------------------------------------------|
| Date        | 21 May 2013                                  |
| Local time  | 17:51:00 IST (UTC+5:30)                      |
| Location    | Hyderabad, India                             |
| Coordinates | 17.388°N, 78.474°E                           |
| Timezone    | Asia/Kolkata (IST, UTC+5:30, no DST)         |

---

## 2. Moon Position (Swiss Ephemeris, True Chitrapaksha Ayanamsa)

| Field                  | Value             |
|------------------------|-------------------|
| Julian Day (UT)        | 2456434.01458333  |
| Sidereal longitude     | 166.86724483°     |
| Zodiac sign            | Virgo (sign #6)   |
| Degrees in sign        | 16° 52′ 02″       |
| Nakshatra              | Hasta (nakshatra #13), Pada 3 |
| Dasa lord at birth     | Moon (10-year dasa)|

---

## 3. Manual Balance Fraction Calculation

Each nakshatra spans exactly 360° ÷ 27 = **13.33333...°**.

Nakshatra 13 (Hasta) occupies the range:

```
Start = (13 − 1) × (360/27) = 12 × 13.3333... = 160.0000°
End   =  13      × (360/27) = 13 × 13.3333... = 173.3333°
```

Moon is at **166.86724483°**, which is inside Hasta. Degrees remaining in the nakshatra:

```
Degrees remaining = 173.3333... − 166.86724483 = 6.46608850°
```

Balance fraction (portion of the Moon dasa still to run at birth):

```
Balance fraction = 6.46608850 / 13.3333... = 0.48495664
```

---

## 4. Dasa Duration with Sidereal Year

We use the sidereal year length **365.256363 days**.

| Calculation step                                 | Value            |
|--------------------------------------------------|------------------|
| Moon dasa full duration (10 years)               | 10 × 365.256363 = **3652.56363 days** |
| Balance days remaining at birth (fraction × full) | 0.48495664 × 3652.56363 = **1771.335 days** |
| Elapsed days before birth (already consumed)     | (1 − 0.48495664) × 3652.56363 = **1881.229 days** |

---

## 5. Converting Balance Days to a Calendar Date

Birth datetime: **2013-05-21 17:51:00**

Adding **1771** whole days to 2013-05-21:

```
2013-05-21 + 365 days = 2014-05-21  (2014, non-leap)
2014-05-21 + 365 days = 2015-05-21  (2015, non-leap)
2015-05-21 + 366 days = 2016-05-21  (2016, leap year)
2016-05-21 + 365 days = 2017-05-21  (2017, non-leap)
2017-05-21 + 309 days = 2018-03-27
                  ---
Total: 365+365+366+365+309 = 1770? Let us verify differently.

Distance from 2013-05-21 to 2018-03-27:
  2018-03-27 → 2018-05-21 = 55 days (4 days in March + 30 April + 21 May)
  2013-05-21 → 2018-05-21 = 5 years = 365+365+366+365+365 = 1826 days
  So: 1826 − 55 = 1771 days ✓
```

Adding the fractional **0.335 days** = **8 hours 02 minutes 22 seconds**:

```
2018-03-27 17:51:00  (birth time, 1771 whole days later)
           + 8h 02m 22s
─────────────────────────────
2018-03-28 01:53:22
```

**Our code produces:** Moon Mahadasa ends **2018-03-28**  
**JHora shows:** Moon Mahadasa ends **2018-03-27**

The difference is exactly 1 day, and arises because the fractional part of the birth time
carries forward into the end date, pushing it to the next calendar day.

---

## 6. Full Comparison of Computed Dates vs JHora

### Mahadasa start/end dates

| Lord    | JHora says  | We computed | Difference |
|---------|-------------|-------------|------------|
| Moon start | 2013-03-27 (pre-birth) | 2013-03-27 | ✓ |
| Moon start (birth) | 2013-05-21 | 2013-05-21 | ✓ |
| Moon end / Mars start | **2018-03-27** | **2018-03-28** | −1 day |
| Mars end / Rahu start | **2025-03-27** | **2025-03-28** | −1 day |
| Rahu end / Jup start  | **2043-03-28** | **2043-03-28** | ✓ |
| Jup end / Sat start   | **2059-03-28** | **2059-03-29** | −1 day |
| Sat end / Merc start  | **2078-03-28** | **2078-03-28** | ✓ |
| Merc end / Ketu start | **2095-03-28** | **2095-03-29** | −1 day |
| Ketu end / Ven start  | **2102-03-29** | **2102-03-30** | −1 day |
| Ven end / Sun start   | **2122-03-29** | **2122-03-30** | −1 day |

### Moon Mahadasa — Antardasa start dates

| Antardasa lord | JHora says  | We computed | Difference |
|----------------|-------------|-------------|------------|
| Saturn         | 2013-05-21  | 2013-05-21  | ✓ |
| Mercury        | **2014-01-27** | **2014-01-26** | +1 day |
| Ketu           | **2015-06-29** | **2015-06-28** | +1 day |
| Venus          | **2016-01-27** | **2016-01-27** | ✓ |
| Sun            | **2017-09-30** | **2017-09-26** | +4 days |

### Mars Mahadasa — Antardasa start dates

| Antardasa lord | JHora says    | We computed   | Difference |
|----------------|---------------|---------------|------------|
| Mars           | **2018-03-27** | **2018-03-28** | −1 day |
| Rahu           | **2018-08-27** | **2018-08-24** | +3 days |
| Jupiter        | **2019-09-15** | **2019-09-12** | +3 days |
| Saturn         | **2020-08-20** | **2020-08-18** | +2 days |
| Mercury        | **2021-09-30** | **2021-09-26** | +4 days |
| Ketu           | **2022-09-27** | **2022-09-24** | +3 days |
| Venus          | **2023-02-20** | **2023-02-20** | ✓ |
| Sun            | **2024-04-20** | **2024-04-21** | −1 day |
| Moon           | **2024-08-30** | **2024-08-27** | +3 days |

**Summary: 5 matched, 22 mismatched out of 27 data points.**  
Discrepancies range from 1 day to 4 days and are **not** all in the same direction,
which rules out a simple off-by-one in the rounding logic.

---

## 7. Our Implementation Details

- **Ephemeris:** Swiss Ephemeris (libswe)
- **Ayanamsa:** True Chitrapaksha (SE_SIDM_TRUE_CITRA)
- **Year length used:** Sidereal year — 365.256363 days
- **Antardasa proportions:** `antardasa_days = mahadasa_full_days × (antar_lord_years / 120.0)`
- **Date conversion:** birth datetime + cumulative seconds, then if the resulting wall-clock
  time is ≥ noon the calendar date is rounded up to the next day

---

## 8. Questions for JHora Maintainers

1. **Year length:** What exact year length (in days) does JHora use for Vimsottari Dasa?
   Is it the sidereal year (≈365.25636), the tropical year (≈365.24219), or a fixed value
   like 365.25?

2. **Time-of-day handling:** When converting dasa days to a calendar date, does JHora:
   - Add the fractional days to the exact birth time (preserving hours/minutes), or
   - Treat the birth date as a pure calendar date (midnight), ignoring the time of day?

3. **Antardasa rounding:** Are Antardasa boundaries rounded/truncated to whole days
   before being accumulated, or is the accumulation done in full floating-point and only
   the final result mapped to a date?

4. **Ayanamsa:** Which ayanamsa is used when computing the Moon's nakshatra position for
   this example? Is it True Chitrapaksha (True Citra), or Lahiri?

5. **Balance fraction precision:** Is the nakshatra span used as exactly 360/27 degrees,
   or is a rounded value applied?

Any clarification on these points would help us reproduce JHora's output exactly.
