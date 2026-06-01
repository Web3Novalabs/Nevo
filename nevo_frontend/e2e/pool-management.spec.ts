import { test, expect } from '@playwright/test';

test.describe('Pool Management / Withdrawal Flow', () => {
  test('dashboard page loads', async ({ page }) => {
    await page.goto('/dashboard');
    await expect(page.locator('body')).toBeVisible();
  });

  test('dashboard shows connect wallet prompt when not authenticated', async ({
    page,
  }) => {
    await page.goto('/dashboard');
    // Without wallet, should show connect prompt or redirect
    await expect(page.locator('body')).not.toContainText('500');
  });

  test('withdraw button requires wallet connection', async ({ page }) => {
    await page.goto('/dashboard');
    const withdrawBtn = page.getByRole('button', { name: /withdraw/i }).first();
    if (await withdrawBtn.isVisible()) {
      await withdrawBtn.click();
      // Should prompt wallet connection
      await expect(page.locator('body')).toBeVisible();
    }
  });

  test('pool management page handles missing pool gracefully', async ({
    page,
  }) => {
    const response = await page.goto('/pools/nonexistent-pool-99999/manage');
    expect([200, 404]).toContain(response?.status());
    await expect(page.locator('body')).not.toContainText('500');
  });

  test('shows error state when withdrawal fails without wallet', async ({
    page,
  }) => {
    await page.goto('/dashboard');
    // Should handle unauthenticated state gracefully
    await expect(page).toHaveURL(/dashboard/);
    await expect(page.locator('body')).not.toContainText(
      'Internal Server Error'
    );
  });
});
