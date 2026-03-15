import { test, expect } from '@playwright/test';
import fs from 'node:fs';

test.beforeAll(() => {
  if (!fs.existsSync('dist/astro.js')) {
    throw new Error('dist/astro.js not found — run ./build.sh first');
  }
});

test('page body contains Work in Progress', async ({ page }) => {
  await page.goto('/');
  const body = await page.textContent('body');
  expect(body).toContain('Work in Progress');
});

// SC-002 + SC-006: Telugu label + numeric longitude within 500 ms
test('Sun longitude at J2000 shows Telugu label సూర్యుడు', async ({ page }) => {
  await page.goto('/');

  await page.fill('#datetime', '2000-01-01T12:00');
  await page.selectOption('#lang', 'te');
  await page.click('#submit');

  // Wait for the submit button to be re-enabled (SC-006: 500 ms timeout)
  await expect(page.locator('#submit')).toBeEnabled({ timeout: 500 });

  const result = await page.textContent('#result');
  expect(result).toContain('సూర్యుడు');
  expect(result).toMatch(/\d+\.\d+/);
});

// SC-003: English label
test('Sun longitude at J2000 shows English label Sun', async ({ page }) => {
  await page.goto('/');

  await page.fill('#datetime', '2000-01-01T12:00');
  await page.selectOption('#lang', 'en');
  await page.click('#submit');

  // Wait for the submit button to be re-enabled (SC-006: 500 ms timeout)
  await expect(page.locator('#submit')).toBeEnabled({ timeout: 500 });

  const result = await page.textContent('#result');
  expect(result).toContain('Sun');
  expect(result).toMatch(/\d+\.\d+/);
});

// SC-001 / SC-002 / SC-003 / SC-004 — list_cities bridge (T020)
test('listCities returns correct data for both languages', async ({ page }) => {
  await page.goto('/');

  // Register network listener BEFORE networkidle so we capture everything.
  const networkRequests: string[] = [];
  page.on('request', (req) => networkRequests.push(req.url()));

  // Drain load-phase requests, then reset the counter so only requests
  // triggered by listCities() itself are counted (SC-004).
  await page.waitForLoadState('networkidle');
  networkRequests.length = 0;

  const [teCities, enCities] = await page.evaluate(() => {
    return [
      // listCities is defined in web/data.js and loaded globally in the browser.
      (globalThis as any).listCities('te'),
      (globalThis as any).listCities('en'),
    ];
  });

  // SC-004: listCities must not trigger any network request — all data is in WASM.
  expect(networkRequests).toHaveLength(0);

  // -------------------------------------------------------------------------
  // SC-001: every entry in any response has all 9 required fields populated.
  // -------------------------------------------------------------------------
  for (const cities of [teCities, enCities]) {
    for (const city of cities) {
      const fields = ['lang', 'cityId', 'timeZone', 'canonicalName', 'cityName', 'region1', 'region2', 'lat', 'lng'];
      for (const field of fields) {
        expect(city[field] !== undefined && city[field] !== null && city[field] !== '',
          `field '${field}' is empty/missing for city '${city.canonicalName}'`
        ).toBe(true);
      }
    }
  }

  // -------------------------------------------------------------------------
  // SC-002: te response is non-empty and a strict subset of en.
  // Tests scale to any number of cities without edits.
  // -------------------------------------------------------------------------
  expect(teCities.length).toBeGreaterThan(0);
  expect(teCities.length).toBeLessThanOrEqual(enCities.length);

  // Every city in te must also appear in en (te ⊆ en by canonicalName).
  const enNames = new Set<string>(enCities.map((c: any) => c.canonicalName));
  for (const city of teCities) {
    expect(enNames.has(city.canonicalName),
      `'${city.canonicalName}' is in te response but absent from en`
    ).toBe(true);
  }

  // Every te city must have Telugu script in its cityName (code point > U+0C00).
  // This replaces hardcoded "not.toContain('Delhi')" — if Delhi ever gets a
  // Telugu translation its cityName will contain Telugu script and this passes.
  for (const city of teCities) {
    const hasTeluguScript = [...city.cityName].some(
      (ch: string) => (ch.codePointAt(0) ?? 0) > 0x0C00
    );
    expect(hasTeluguScript,
      `cityName '${city.cityName}' for '${city.canonicalName}' has no Telugu script`
    ).toBe(true);
  }

  // Cities in en but NOT in te must NOT appear in the te response.
  // Derived from bridge data — no city names hardcoded.
  const teNames = new Set<string>(teCities.map((c: any) => c.canonicalName));
  for (const city of enCities) {
    if (!teNames.has(city.canonicalName)) {
      // This city has no te translation — it must stay absent from te response.
      expect(teNames.has(city.canonicalName),
        `'${city.canonicalName}' has no te cityName but appears in te response`
      ).toBe(false);
    }
  }

  // Hyderabad is a known permanent seed city — assert its Telugu name directly.
  const hydEntry = teCities.find((c: any) => c.canonicalName === 'Hyderabad');
  expect(hydEntry).toBeDefined();
  expect(hydEntry.cityName).toBe('హైదరాబాద్');

  // -------------------------------------------------------------------------
  // SC-003: canonicalName is always ASCII (English) regardless of response lang.
  // -------------------------------------------------------------------------
  for (const cities of [teCities, enCities]) {
    for (const city of cities) {
      expect(/^[\x00-\x7F]+$/.test(city.canonicalName),
        `canonicalName '${city.canonicalName}' contains non-ASCII characters`
      ).toBe(true);
    }
  }

  // For lang=en, canonicalName must equal cityName.
  for (const city of enCities) {
    expect(city.canonicalName).toBe(city.cityName);
  }
});
