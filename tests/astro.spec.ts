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

test('Sun longitude at J2000 ≈ 280.37°', async ({ page }) => {
  const consolePromise = page.waitForEvent('console', {
    predicate: msg => msg.text().startsWith('Sun longitude:'),
    timeout: 10000,
  });
  await page.goto('/');
  const msg = await consolePromise;
  const longitude = parseFloat(msg.text().slice('Sun longitude: '.length));
  expect(Math.abs(longitude - 280.37)).toBeLessThan(0.5);
});
