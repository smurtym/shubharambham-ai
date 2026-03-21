# Vimsottari Dasa Duration Analysis: JHora vs Our Formula

## Parameters

- **Sidereal year length used:** 365.256363 days
- **Antardasa formula:** `antar_days = maha_full_days × (antar_lord_years / 120)`
- **Total Mahadasa years:** 120

---

## 1. Mahadasa Total Duration Comparison

Each Mahadasa duration is computed from the exact JHora timestamps and compared
against `lord_years × 365.256363` (our formula).

| Mahadasa | JHora Start | JHora End | JHora Days | Expected Days | Diff | Status |
|---|---|---|---:|---:|---|:---:|
| Moon (10y) | 2008-03-26 21:46 | 2018-03-27 11:23 | 3652.5672 | 3652.5636 | +0.0035 d (+0.09 h) | ✓ |
| Mars (7y) | 2018-03-27 11:23 | 2025-03-27 06:35 | 2556.8002 | 2556.7945 | +0.0057 d (+0.14 h) | ✓ |
| Rahu (18y) | 2025-03-27 06:35 | 2043-03-27 21:07 | 6574.6059 | 6574.6145 | −0.0086 d (−0.21 h) | ✓ |
| Jupiter (16y) | 2043-03-27 21:07 | 2059-03-27 23:40 | 5844.1061 | 5844.1018 | +0.0043 d (+0.10 h) | ✓ |
| Saturn (19y) | 2059-03-27 23:40 | 2078-03-27 20:32 | 6939.8693 | 6939.8709 | −0.0016 d (−0.04 h) | ✓ |
| Mercury (17y) | 2078-03-27 20:32 | 2095-03-28 05:19 | 6209.3657 | 6209.3582 | +0.0075 d (+0.18 h) | ✓ |
| Ketu (7y) | 2095-03-28 05:19 | 2102-03-29 00:09 | 2556.7852 | 2556.7945 | −0.0093 d (−0.22 h) | ✓ |
| Venus (20y) | 2102-03-29 00:09 | 2122-03-29 03:27 | 7305.1376 | 7305.1273 | +0.0103 d (+0.25 h) | ✓ |
| Sun (6y) | 2122-03-29 03:27 | 2128-03-28 16:10 | 2191.5298 | 2191.5382 | −0.0084 d (−0.20 h) | ✓ |

**Mahadasa totals matched:** 9 / 9 (threshold ±0.5 day)

All Mahadasa total durations agree with the sidereal year formula to within ~0.01 days
(< 15 minutes). This confirms that the **year length of 365.256363 days is correct**
for Mahadasa totals.

---

## 2. Antardasa Duration Comparison (per Mahadasa)

For each Antardasa the expected duration is:
`expected = maha_full_days × (antar_lord_years / 120)`

### Moon Mahadasa (10 years = 3652.5636 days)

| Antardasa | JHora Start | JHora End | JHora Days | Expected Days | Diff | Status |
|---|---|---|---:|---:|---|:---:|
| Moon (10y) | 2008-03-26 21:46:21 | 2009-01-26 10:57:58 | 305.5497 | 304.3803 | +1.1694 d (+28.07 h) | ✗ |
| Mars (7y) | 2009-01-26 10:57:58 | 2009-08-29 20:20:20 | 215.3905 | 213.0662 | +2.3243 d (+55.78 h) | ✗ |
| Rahu (18y) | 2009-08-29 20:20:20 | 2011-02-25 15:00:13 | 544.7777 | 547.8845 | −3.1068 d (−74.56 h) | ✗ |
| Jupiter (16y) | 2011-02-25 15:00:13 | 2012-06-27 22:41:22 | 488.3202 | 487.0085 | +1.3118 d (+31.48 h) | ✗ |
| Saturn (19y) | 2012-06-27 22:41:22 | 2014-01-26 17:49:57 | 577.7976 | 578.3226 | −0.5249 d (−12.60 h) | ✗ |
| Mercury (17y) | 2014-01-26 17:49:57 | 2015-06-28 17:10:10 | 517.9724 | 517.4465 | +0.5259 d (+12.62 h) | ✗ |
| Ketu (7y) | 2015-06-28 17:10:10 | 2016-01-27 06:03:24 | 212.5370 | 213.0662 | −0.5292 d (−12.70 h) | ✗ |
| Venus (20y) | 2016-01-27 06:03:24 | 2017-09-29 16:48:38 | 611.4481 | 608.7606 | +2.6875 d (+64.50 h) | ✗ |
| Sun (6y) | 2017-09-29 16:48:38 | 2018-03-27 11:23:05 | 178.7739 | 182.6282 | −3.8543 d (−92.50 h) | ✗ |
| **TOTAL** | | | **3652.5672** | **3652.5636** | +0.0035 d (+0.09 h) | |

Antardasa match: 0/9

### Mars Mahadasa (7 years = 2556.7945 days)

| Antardasa | JHora Start | JHora End | JHora Days | Expected Days | Diff | Status |
|---|---|---|---:|---:|---|:---:|
| Mars (7y) | 2018-03-27 11:23:05 | 2018-08-27 01:16:03 | 152.5784 | 149.1463 | +3.4321 d (+82.37 h) | ✗ |
| Rahu (18y) | 2018-08-27 01:16:03 | 2019-09-14 21:07:38 | 383.8275 | 383.5192 | +0.3083 d (+7.40 h) | ✓ |
| Jupiter (16y) | 2019-09-14 21:07:38 | 2020-08-20 08:07:47 | 340.4584 | 340.9059 | −0.4475 d (−10.74 h) | ✓ |
| Saturn (19y) | 2020-08-20 08:07:47 | 2021-09-29 17:31:45 | 405.3916 | 404.8258 | +0.5658 d (+13.58 h) | ✗ |
| Mercury (17y) | 2021-09-29 17:31:45 | 2022-09-26 22:10:41 | 362.1937 | 362.2126 | −0.0189 d (−0.45 h) | ✓ |
| Ketu (7y) | 2022-09-26 22:10:41 | 2023-02-19 17:56:42 | 145.8236 | 149.1463 | −3.3227 d (−79.75 h) | ✗ |
| Venus (20y) | 2023-02-19 17:56:42 | 2024-04-20 10:15:44 | 425.6799 | 426.1324 | −0.4525 d (−10.86 h) | ✓ |
| Sun (6y) | 2024-04-20 10:15:44 | 2024-08-29 16:44:35 | 131.2700 | 127.8397 | +3.4303 d (+82.33 h) | ✗ |
| Moon (10y) | 2024-08-29 16:44:35 | 2025-03-27 06:35:25 | 209.5770 | 213.0662 | −3.4892 d (−83.74 h) | ✗ |
| **TOTAL** | | | **2556.8002** | **2556.7945** | +0.0057 d (+0.14 h) | |

Antardasa match: 4/9

### Rahu Mahadasa (18 years = 6574.6145 days)

| Antardasa | JHora Start | JHora End | JHora Days | Expected Days | Diff | Status |
|---|---|---|---:|---:|---|:---:|
| Rahu (18y) | 2025-03-27 06:35:25 | 2027-12-11 04:32:33 | 988.9147 | 986.1922 | +2.7225 d (+65.34 h) | ✗ |
| Jupiter (16y) | 2027-12-11 04:32:33 | 2030-05-03 07:07:16 | 874.1074 | 876.6153 | −2.5078 d (−60.19 h) | ✗ |
| Saturn (19y) | 2030-05-03 07:07:16 | 2033-03-09 05:25:17 | 1040.9292 | 1040.9806 | −0.0515 d (−1.23 h) | ✓ |
| Mercury (17y) | 2033-03-09 05:25:17 | 2035-09-30 07:37:24 | 935.0917 | 931.4037 | +3.6880 d (+88.51 h) | ✗ |
| Ketu (7y) | 2035-09-30 07:37:24 | 2036-10-17 19:17:51 | 383.4864 | 383.5192 | −0.0328 d (−0.79 h) | ✓ |
| Venus (20y) | 2036-10-17 19:17:51 | 2039-10-18 13:42:36 | 1095.7672 | 1095.7691 | −0.0019 d (−0.05 h) | ✓ |
| Sun (6y) | 2039-10-18 13:42:36 | 2040-09-11 04:15:17 | 328.6060 | 328.7307 | −0.1247 d (−2.99 h) | ✓ |
| Moon (10y) | 2040-09-11 04:15:17 | 2042-03-09 12:56:03 | 544.3616 | 547.8845 | −3.5229 d (−84.55 h) | ✗ |
| Mars (7y) | 2042-03-09 12:56:03 | 2043-03-27 21:07:57 | 383.3416 | 383.5192 | −0.1776 d (−4.26 h) | ✓ |
| **TOTAL** | | | **6574.6059** | **6574.6145** | −0.0086 d (−0.21 h) | |

Antardasa match: 5/9

### Jupiter Mahadasa (16 years = 5844.1018 days)

| Antardasa | JHora Start | JHora End | JHora Days | Expected Days | Diff | Status |
|---|---|---|---:|---:|---|:---:|
| Jupiter (16y) | 2043-03-27 21:07:57 | 2045-05-15 13:13:00 | 779.6702 | 779.2136 | +0.4566 d (+10.96 h) | ✓ |
| Saturn (19y) | 2045-05-15 13:13:00 | 2047-11-29 11:44:20 | 927.9384 | 925.3161 | +2.6223 d (+62.94 h) | ✗ |
| Mercury (17y) | 2047-11-29 11:44:20 | 2050-03-03 14:21:36 | 825.1092 | 827.9144 | −2.8052 d (−67.33 h) | ✗ |
| Ketu (7y) | 2050-03-03 14:21:36 | 2051-02-08 01:16:51 | 341.4550 | 340.9059 | +0.5491 d (+13.18 h) | ✗ |
| Venus (20y) | 2051-02-08 01:16:51 | 2053-10-12 02:31:34 | 977.0519 | 974.0170 | +3.0349 d (+72.84 h) | ✗ |
| Sun (6y) | 2053-10-12 02:31:34 | 2054-07-30 03:36:49 | 291.0453 | 292.2051 | −1.1598 d (−27.83 h) | ✗ |
| Moon (10y) | 2054-07-30 03:36:49 | 2055-11-29 12:57:49 | 487.3896 | 487.0085 | +0.3811 d (+9.15 h) | ✓ |
| Mars (7y) | 2055-11-29 12:57:49 | 2056-11-04 23:34:54 | 341.4424 | 340.9059 | +0.5365 d (+12.88 h) | ✗ |
| Rahu (18y) | 2056-11-04 23:34:54 | 2059-03-27 23:40:44 | 873.0041 | 876.6153 | −3.6112 d (−86.67 h) | ✗ |
| **TOTAL** | | | **5844.1061** | **5844.1018** | +0.0043 d (+0.10 h) | |

Antardasa match: 2/9

### Saturn Mahadasa (19 years = 6939.8709 days)

| Antardasa | JHora Start | JHora End | JHora Days | Expected Days | Diff | Status |
|---|---|---|---:|---:|---|:---:|
| Saturn (19y) | 2059-03-27 23:40:44 | 2062-03-30 19:01:11 | 1098.8059 | 1098.8129 | −0.0070 d (−0.17 h) | ✓ |
| Mercury (17y) | 2062-03-30 19:01:11 | 2064-12-10 16:12:37 | 985.8829 | 983.1484 | +2.7346 d (+65.63 h) | ✗ |
| Ketu (7y) | 2064-12-10 16:12:37 | 2066-01-18 05:30:41 | 403.5542 | 404.8258 | −1.2716 d (−30.52 h) | ✗ |
| Venus (20y) | 2066-01-18 05:30:41 | 2069-03-18 11:36:16 | 1155.2539 | 1156.6451 | −1.3913 d (−33.39 h) | ✗ |
| Sun (6y) | 2069-03-18 11:36:16 | 2070-02-28 17:46:22 | 347.2570 | 346.9935 | +0.2635 d (+6.32 h) | ✓ |
| Moon (10y) | 2070-02-28 17:46:22 | 2071-10-03 14:12:55 | 581.8518 | 578.3226 | +3.5292 d (+84.70 h) | ✗ |
| Mars (7y) | 2071-10-03 14:12:55 | 2072-11-11 01:25:57 | 404.4674 | 404.8258 | −0.3584 d (−8.60 h) | ✓ |
| Rahu (18y) | 2072-11-11 01:25:57 | 2075-09-18 07:19:42 | 1041.2457 | 1040.9806 | +0.2650 d (+6.36 h) | ✓ |
| Jupiter (16y) | 2075-09-18 07:19:42 | 2078-03-27 20:32:30 | 921.5506 | 925.3161 | −3.7656 d (−90.37 h) | ✗ |
| **TOTAL** | | | **6939.8693** | **6939.8709** | −0.0016 d (−0.04 h) | |

Antardasa match: 4/9

### Mercury Mahadasa (17 years = 6209.3582 days)

| Antardasa | JHora Start | JHora End | JHora Days | Expected Days | Diff | Status |
|---|---|---|---:|---:|---|:---:|
| Mercury (17y) | 2078-03-27 20:32:30 | 2080-08-26 22:33:13 | 883.0838 | 879.6591 | +3.4248 d (+82.19 h) | ✗ |
| Ketu (7y) | 2080-08-26 22:33:13 | 2081-08-24 01:57:35 | 362.1419 | 362.2126 | −0.0706 d (−1.70 h) | ✓ |
| Venus (20y) | 2081-08-24 01:57:35 | 2084-06-22 02:33:36 | 1033.0250 | 1034.8930 | −1.8680 d (−44.83 h) | ✗ |
| Sun (6y) | 2084-06-22 02:33:36 | 2085-04-27 05:22:07 | 309.1170 | 310.4679 | −1.3509 d (−32.42 h) | ✗ |
| Moon (10y) | 2085-04-27 05:22:07 | 2086-09-30 09:13:44 | 521.1608 | 517.4465 | +3.7143 d (+89.14 h) | ✗ |
| Mars (7y) | 2086-09-30 09:13:44 | 2087-09-27 14:00:48 | 362.1994 | 362.2126 | −0.0132 d (−0.32 h) | ✓ |
| Rahu (18y) | 2087-09-27 14:00:48 | 2090-04-12 03:39:59 | 927.5689 | 931.4037 | −3.8348 d (−92.04 h) | ✗ |
| Jupiter (16y) | 2090-04-12 03:39:59 | 2092-07-20 11:11:57 | 830.3139 | 827.9144 | +2.3994 d (+57.59 h) | ✗ |
| Saturn (19y) | 2092-07-20 11:11:57 | 2095-03-28 05:19:06 | 980.7550 | 983.1484 | −2.3934 d (−57.44 h) | ✗ |
| **TOTAL** | | | **6209.3657** | **6209.3582** | +0.0075 d (+0.18 h) | |

Antardasa match: 2/9

### Ketu Mahadasa (7 years = 2556.7945 days)

| Antardasa | JHora Start | JHora End | JHora Days | Expected Days | Diff | Status |
|---|---|---|---:|---:|---|:---:|
| Ketu (7y) | 2095-03-28 05:19:06 | 2095-08-27 18:46:40 | 152.5608 | 149.1463 | +3.4145 d (+81.95 h) | ✗ |
| Venus (20y) | 2095-08-27 18:46:40 | 2096-10-27 05:31:13 | 426.4476 | 426.1324 | +0.3152 d (+7.56 h) | ✓ |
| Sun (6y) | 2096-10-27 05:31:13 | 2097-02-28 15:54:55 | 124.4331 | 127.8397 | −3.4066 d (−81.76 h) | ✗ |
| Moon (10y) | 2097-02-28 15:54:55 | 2097-10-03 06:06:27 | 216.5913 | 213.0662 | +3.5251 d (+84.60 h) | ✗ |
| Mars (7y) | 2097-10-03 06:06:27 | 2098-02-25 22:29:56 | 145.6830 | 149.1463 | −3.4634 d (−83.12 h) | ✗ |
| Rahu (18y) | 2098-02-25 22:29:56 | 2099-03-16 03:53:26 | 383.2247 | 383.5192 | −0.2945 d (−7.07 h) | ✓ |
| Jupiter (16y) | 2099-03-16 03:53:26 | 2100-02-20 11:39:21 | 341.3236 | 340.9059 | +0.4176 d (+10.02 h) | ✓ |
| **TOTAL** | | | **2556.7852** | **2556.7945** | −0.0093 d (−0.22 h) | |

Antardasa match: 3/7

### Venus Mahadasa (20 years = 7305.1273 days)

*(no Antardasa data in source file)*

### Sun Mahadasa (6 years = 2191.5382 days)

*(no Antardasa data in source file)*

---

## 3. Overall Summary

| Category | Matched | Total | Match rate |
|---|---:|---:|---:|
| Mahadasa totals | 9 | 9 | 100% |
| Antardasa durations | 20 | 61 | 33% |

---

## 4. Key Observations

### 4.1 Mahadasa totals match the sidereal year exactly

All 9 Mahadasa total durations computed from JHora timestamps agree with
`lord_years × 365.256363` to within **< 0.011 days (< 16 minutes)**.
This is consistent with floating-point rounding when JHora converts fractional days
to wall-clock timestamps and back. The year length is confirmed as correct.

### 4.2 Antardasa durations are systematically inconsistent with our formula

Within each Mahadasa, the individual Antardasa durations from JHora differ from
our proportional formula by **up to ±4 days**. The errors have no fixed sign and
do not scale with Antardasa lord years in a predictable way.

For example, within the **Mars Mahadasa**:

| Antardasa | Expected | JHora Actual | Diff |
|---|---:|---:|---:|
| Mars (7y) | 149.146 | 152.578 | +3.432 |
| Rahu (18y) | 383.519 | 383.827 | +0.308 |
| Jupiter (16y) | 340.906 | 340.458 | -0.448 |
| Saturn (19y) | 404.826 | 405.392 | +0.566 |
| Mercury (17y) | 362.213 | 362.194 | -0.019 |
| Ketu (7y) | 149.146 | 145.824 | -3.323 |
| Venus (20y) | 426.132 | 425.680 | -0.453 |
| Sun (6y) | 127.840 | 131.270 | +3.430 |
| Moon (10y) | 213.066 | 209.577 | -3.489 |

The Mars/Mars and Sun Antardasa are overestimated by ~3.4 days, while Ketu and
Moon are underestimated by a similar amount — yet adjacent Antardasa like Rahu,
Jupiter, Mercury, Venus match to within 0.5 days.

### 4.3 Antardasa fractions are not exact integer-year proportions

If JHora used `maha_full_days × (antar_years / 120)` each Antardasa fraction should
equal exactly `antar_years / 120`. Below are the actual fractions × 120 for Moon MD:

| Antardasa | Expected years | Actual × 120 | Δ years |
|---|---:|---:|---:|
| Moon | 10 | 10.0384 | +0.0384 |
| Mars | 7 | 7.0764 | +0.0764 |
| Rahu | 18 | 17.8979 | -0.1021 |
| Jupiter | 16 | 16.0431 | +0.0431 |
| Saturn | 19 | 18.9827 | -0.0173 |
| Mercury | 17 | 17.0173 | +0.0173 |
| Ketu | 7 | 6.9826 | -0.0174 |
| Venus | 20 | 20.0883 | +0.0883 |
| Sun | 6 | 5.8734 | -0.1266 |

None of the proportions are exact integers, which rules out a simple rounding error
in our code. JHora appears to use a **different formula** for Antardasa durations.

---

## 5. Hypothesis

The most likely explanation is that JHora converts Antardasa durations to calendar
dates by adding *fractional sidereal years* (or tropical years) directly to a Julian
Day Number rather than multiplying by a fixed constant. Because the actual length of
a sidereal year varies slightly over centuries, this produces Antardasa boundaries
that drift from our fixed `365.256363 days/year` formula by 1–4 days over the course
of a Mahadasa.

**Specific questions to ask JHora maintainers:**

1. Does JHora compute Antardasa end dates by adding `(maha_years × antar_years / 120)`
   **actual sidereal years** (variable length) to a JD, rather than a fixed days constant?
2. Or does JHora compute each Antardasa duration as a fixed number of days using a
   different year length than the Mahadasa year length?
3. Is there a balance-fraction sub-calculation applied within each Antardasa, similar
   to how the birth Nakshatra fraction is applied at the Mahadasa level?
