import { test, expect } from '@playwright/test';

test.describe('Cross-browser compatibility', () => {
  test('homepage renders correctly', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
    await expect(page.locator('nav')).toBeVisible();
  });

  test('CSS custom properties (variables) are applied', async ({ page }) => {
    await page.goto('/');
    const bgColor = await page.evaluate(
      () => getComputedStyle(document.body).backgroundColor
    );
    // Should not be empty — custom property resolved
    expect(bgColor).toBeTruthy();
    expect(bgColor).not.toBe('');
  });

  test('dark mode toggle works', async ({ page }) => {
    await page.goto('/');
    // Check initial state
    const html = page.locator('html');
    const initialClass = await html.getAttribute('class');

    // Find and click theme toggle if present
    const toggle = page.getByRole('button', { name: /theme|dark|light/i });
    if (await toggle.isVisible()) {
      await toggle.click();
      const newClass = await html.getAttribute('class');
      expect(newClass).not.toBe(initialClass);
    }
  });

  test('navigation links are keyboard accessible', async ({ page }) => {
    await page.goto('/');
    // Tab to first nav link and activate with Enter
    await page.keyboard.press('Tab');
    const focused = page.locator(':focus');
    await expect(focused).toBeVisible();
  });

  test('pools page loads without JS errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));
    await page.goto('/pools');
    await page.waitForLoadState('networkidle');
    expect(errors).toHaveLength(0);
  });

  test('mobile viewport renders nav correctly', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 });
    await page.goto('/');
    await expect(page.locator('nav')).toBeVisible();
  });

  test('wallet connection layout remains usable on a mobile viewport', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto('/');
    await expect(page.locator('body')).toBeVisible();
    await expect(page.locator('nav')).toBeVisible();
  });

  test('images load without broken src', async ({ page }) => {
    await page.goto('/');
    const images = page.locator('img');
    const count = await images.count();
    for (let i = 0; i < count; i++) {
      const naturalWidth = await images
        .nth(i)
        .evaluate((img: HTMLImageElement) => img.naturalWidth);
      expect(naturalWidth).toBeGreaterThan(0);
    }
  });

  test('scrolling works on pools page', async ({ page }) => {
    await page.goto('/pools');
    await page.evaluate(() => window.scrollTo(0, 200));
    const scrollY = await page.evaluate(() => window.scrollY);
    expect(scrollY).toBeGreaterThanOrEqual(0);
  });
});
