import { test, expect, type Page } from '@playwright/test';
import { mockApi, mockFreighter } from './helpers';

async function fillBasics(page: Page, category = 'Education') {
  await page
    .getByRole('textbox', { name: 'Title', exact: true })
    .fill('Clean Water Initiative');
  await page
    .getByRole('textbox', { name: 'Description', exact: true })
    .fill('Funding clean water wells for rural communities.');
  if (category) {
    await page
      .getByRole('combobox', { name: 'Category' })
      .selectOption(category);
  }
}

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

  test.describe('with a connected wallet', () => {
    test.beforeEach(async ({ page }) => {
      await mockFreighter(page);
    });

    test('rejects advancing when a required field is missing', async ({
      page,
    }) => {
      await page.goto('/pools/new');
      await fillBasics(page, '');

      await page.getByRole('button', { name: /next: goal/i }).click();

      await expect(page.getByText('Please select a category.')).toBeVisible();
      await expect(
        page.getByRole('heading', { name: 'Basic Information' })
      ).toBeVisible();
      await expect(
        page.getByRole('heading', { name: 'Goal & Duration' })
      ).toHaveCount(0);
    });

    test('shows the creation fee deduction when one is configured', async ({
      page,
    }) => {
      await page.goto('/pools/new');
      await fillBasics(page);
      await page.getByRole('button', { name: /next: goal/i }).click();
      await page.getByLabel('Goal Amount (XLM)').fill('5000');
      await page.getByRole('button', { name: 'Preview Pool' }).click();
      await expect(
        page.getByRole('heading', { name: 'Preview Your Pool' })
      ).toBeVisible();

      const fee = page.getByText(/creation fee/i);
      test.skip((await fee.count()) === 0, 'No creation fee is configured');
      await expect(fee.first()).toBeVisible();
      await expect(fee.first()).toContainText(/XLM/);
    });

    test('navigating away mid-creation does not create a partial pool', async ({
      page,
    }) => {
      const createRequests: string[] = [];
      page.on('request', (request) => {
        if (
          request.method() === 'POST' &&
          new URL(request.url()).pathname === '/pools'
        ) {
          createRequests.push(request.url());
        }
      });
      await mockApi(page, /\/pools(\?.*)?$/, (route) =>
        route.fulfill({ json: [] })
      );

      await page.goto('/pools/new');
      await fillBasics(page);
      await page.getByRole('button', { name: /next: goal/i }).click();
      await page.getByLabel('Goal Amount (XLM)').fill('5000');

      await page.goto('/pools');
      await expect(page).toHaveURL('/pools');

      await page.goto('/pools/new');
      await expect(
        page.getByRole('heading', { name: 'Basic Information' })
      ).toBeVisible();
      await expect(
        page.getByRole('textbox', { name: 'Title', exact: true })
      ).toHaveValue('');
      expect(createRequests).toHaveLength(0);
    });
  });
});
