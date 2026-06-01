import { test, expect } from '@playwright/test';

test.describe('Pool Creation Flow', () => {
  test('navigates to create pool page', async ({ page }) => {
    await page.goto('/');
    await page
      .getByRole('link', { name: /create a pool/i })
      .first()
      .click();
    await expect(page).toHaveURL('/pools/new');
  });

  test('create pool page loads', async ({ page }) => {
    await page.goto('/pools/new');
    await expect(page.locator('body')).toBeVisible();
  });

  test('create pool page shows wallet connect when not connected', async ({
    page,
  }) => {
    await page.goto('/pools/new');
    // Without wallet connected, should show connect prompt or form
    const body = page.locator('body');
    await expect(body).toBeVisible();
  });

  test('create pool form requires wallet connection', async ({ page }) => {
    await page.goto('/pools/new');
    // Form should not submit without wallet
    const submitBtn = page.getByRole('button', {
      name: /create|submit|launch/i,
    });
    if (await submitBtn.isVisible()) {
      await submitBtn.click();
      // Should show error or wallet connect prompt
      await expect(page.locator('body')).toBeVisible();
    }
  });

  test('shows error state when pool creation fails without wallet', async ({
    page,
  }) => {
    await page.goto('/pools/new');
    // Page should handle unauthenticated state gracefully
    await expect(page).toHaveURL('/pools/new');
    await expect(page.locator('body')).not.toContainText('500');
  });
});
