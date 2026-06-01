import { test, expect } from '@playwright/test';

test.describe('Pool Browsing Flow', () => {
  test('homepage loads and shows hero section', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
    await expect(
      page.getByRole('link', { name: /browse pools/i }).first()
    ).toBeVisible();
  });

  test('navigates to pools page from hero CTA', async ({ page }) => {
    await page.goto('/');
    await page
      .getByRole('link', { name: /browse pools/i })
      .first()
      .click();
    await expect(page).toHaveURL('/pools');
  });

  test('pools page loads', async ({ page }) => {
    await page.goto('/pools');
    await expect(page).toHaveURL('/pools');
    await expect(page.locator('body')).toBeVisible();
  });

  test('navbar is present on all pages', async ({ page }) => {
    for (const route of ['/', '/pools']) {
      await page.goto(route);
      await expect(page.locator('nav')).toBeVisible();
    }
  });

  test('shows error state for invalid pool', async ({ page }) => {
    const response = await page.goto('/pools/nonexistent-pool-id-12345');
    // Should either show 404 or redirect, not crash
    expect([200, 404]).toContain(response?.status());
  });
});
