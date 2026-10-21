import { test, expect, type Page } from '@playwright/test';
import { makePool, mockApi } from './helpers';

async function mockPools(page: Page, pools: ReturnType<typeof makePool>[]) {
  await mockApi(page, /\/pools(\?.*)?$/, (route) =>
    route.fulfill({ json: pools })
  );
}

function poolTitles(page: Page) {
  return page.getByRole('heading', { level: 3, name: /^Test Pool/ });
}

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

  test('shows EmptyState when a filter matches zero pools', async ({
    page,
  }) => {
    await mockPools(page, [makePool(1), makePool(2), makePool(3)]);
    await page.goto('/pools');
    await expect(poolTitles(page)).toHaveCount(3);

    await page.getByRole('checkbox', { name: /completed/i }).check();

    await expect(page.getByText('0 results')).toBeVisible();
    await expect(
      page.getByRole('heading', { name: 'No pools found' })
    ).toBeVisible();
    await expect(
      page.getByText('No pools match the current filter combination.')
    ).toBeVisible();
    await expect(poolTitles(page)).toHaveCount(0);

    await page.getByRole('button', { name: 'Clear filters' }).click();
    await expect(poolTitles(page)).toHaveCount(3);
  });

  test('paginates to the last page', async ({ page }) => {
    await mockPools(
      page,
      Array.from({ length: 14 }, (_, i) => makePool(i + 1))
    );
    await page.goto('/pools');
    await expect(poolTitles(page)).toHaveCount(6);

    await page.getByRole('button', { name: 'Page 3' }).click();

    await expect(page.getByRole('button', { name: 'Page 3' })).toHaveAttribute(
      'aria-current',
      'page'
    );
    await expect(page).toHaveURL(/[?&]page=3/);
    await expect(poolTitles(page)).toHaveText(['Test Pool 2', 'Test Pool 1']);
  });

  test('search narrows results and sort reorders them', async ({ page }) => {
    await mockPools(page, [
      makePool(1, { title: 'Test Pool Water', raised: 900 }),
      makePool(2, { title: 'Test Pool Books', raised: 100 }),
      makePool(3, { title: 'Test Pool Wells', raised: 500 }),
    ]);
    await page.goto('/pools');
    await expect(poolTitles(page)).toHaveText([
      'Test Pool Wells',
      'Test Pool Books',
      'Test Pool Water',
    ]);

    await page.getByLabel('Search campaigns').fill('w');
    await expect(page).toHaveURL(/[?&]q=w/);
    await expect(poolTitles(page)).toHaveText([
      'Test Pool Wells',
      'Test Pool Water',
    ]);

    await page.getByLabel('Sort').selectOption('most-funded');
    await expect(page).toHaveURL(/[?&]sort=most-funded/);
    await expect(poolTitles(page)).toHaveText([
      'Test Pool Water',
      'Test Pool Wells',
    ]);
  });
});
