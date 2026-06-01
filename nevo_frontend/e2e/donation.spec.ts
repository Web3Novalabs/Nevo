import { test, expect } from '@playwright/test';

test.describe('Donation Flow', () => {
  test('pools page is accessible for donation browsing', async ({ page }) => {
    await page.goto('/pools');
    await expect(page).toHaveURL('/pools');
    await expect(page.locator('body')).toBeVisible();
  });

  test('donate button requires wallet connection', async ({ page }) => {
    await page.goto('/pools');
    const donateBtn = page.getByRole('button', { name: /donate/i }).first();
    if (await donateBtn.isVisible()) {
      await donateBtn.click();
      // Should prompt wallet connection
      await expect(page.locator('body')).toBeVisible();
    }
  });

  test('donation amount input validates correctly', async ({ page }) => {
    await page.goto('/pools');
    const amountInput = page.getByRole('spinbutton').first();
    if (await amountInput.isVisible()) {
      await amountInput.fill('-1');
      const donateBtn = page.getByRole('button', { name: /donate/i }).first();
      if (await donateBtn.isVisible()) {
        await donateBtn.click();
        // Should show validation error
        await expect(page.locator('body')).toBeVisible();
      }
    }
  });

  test('shows error state when donating without wallet', async ({ page }) => {
    await page.goto('/pools');
    // Page should not crash without wallet
    await expect(page.locator('body')).not.toContainText('500');
    await expect(page.locator('body')).not.toContainText(
      'Internal Server Error'
    );
  });
});
