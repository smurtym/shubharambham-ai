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
