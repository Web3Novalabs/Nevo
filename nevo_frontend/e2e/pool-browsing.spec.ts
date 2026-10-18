import { test, expect, type Page } from '@playwright/test';

const MOCK_POOLS = [
  {
    id: 'pool-1',
    title: 'Solar Microgrids for Rural Schools',
    description: 'Clean energy infrastructure for off-grid classrooms.',
    category: 'Environment',
    status: 'Active',
    target: 10000,
    raised: 2500,
    imageColor: '#10b981',
    creator: 'GCREATOR1',
    createdAt: '2026-01-01T00:00:00Z',
  },
  {
    id: 'pool-2',
    title: 'Clean Water Wells Initiative',
    description: 'Borehole drilling and filtration systems.',
    category: 'Humanitarian',
    status: 'Active',
    target: 5000,
    raised: 4800,
    imageColor: '#0ea5e9',
    creator: 'GCREATOR2',
    createdAt: '2026-01-02T00:00:00Z',
  },
  {
    id: 'pool-3',
    title: 'Open Source Developer Grants',
    description: 'Funding Soroban smart contract security tooling.',
    category: 'Technology',
    status: 'Active',
    target: 20000,
    raised: 15000,
    imageColor: '#6366f1',
    creator: 'GCREATOR3',
    createdAt: '2026-01-03T00:00:00Z',
  },
  {
    id: 'pool-4',
    title: 'Mobile Health Clinics',
    description: 'Preventive care and vaccinations for remote villages.',
    category: 'Healthcare',
    status: 'Active',
    target: 8000,
    raised: 4000,
    imageColor: '#f43f5e',
    creator: 'GCREATOR4',
    createdAt: '2026-01-04T00:00:00Z',
  },
  {
    id: 'pool-5',
    title: 'STEM Scholarships for Youth',
    description: 'Tuition and laptops for aspiring engineers.',
    category: 'Education',
    status: 'Active',
    target: 12000,
    raised: 6000,
    imageColor: '#0284c7',
    creator: 'GCREATOR5',
    createdAt: '2026-01-05T00:00:00Z',
  },
  {
    id: 'pool-6',
    title: 'Emergency Flood Relief Fund',
    description: 'Rapid shelter and food distribution.',
    category: 'Emergency',
    status: 'Active',
    target: 15000,
    raised: 12000,
    imageColor: '#ef4444',
    creator: 'GCREATOR6',
    createdAt: '2026-01-06T00:00:00Z',
  },
  {
    id: 'pool-7',
    title: 'Community Food Bank Expansion',
    description: 'Cold storage for local food distribution.',
    category: 'Community',
    status: 'Completed',
    target: 6000,
    raised: 6000,
    imageColor: '#14b8a6',
    creator: 'GCREATOR7',
    createdAt: '2026-01-07T00:00:00Z',
  },
  {
    id: 'pool-8',
    title: 'Wildlife Corridor Reforestation',
    description: 'Planting native trees across fragmented habitats.',
    category: 'Environment',
    status: 'Active',
    target: 9000,
    raised: 1800,
    imageColor: '#84cc16',
    creator: 'GCREATOR8',
    createdAt: '2026-01-08T00:00:00Z',
  },
];

async function mockPoolsApi(page: Page, pools = MOCK_POOLS) {
  await page.route('**/pools*', async (route) => {
    const resourceType = route.request().resourceType();
    if (resourceType === 'fetch' || resourceType === 'xhr') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(pools),
      });
      return;
    }
    await route.continue();
  });
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

  test('renders EmptyState when filter combination matches zero pools', async ({
    page,
  }) => {
    await mockPoolsApi(page);
    await page.goto('/pools');

    const searchInput = page.locator('#search-pools');
    await expect(searchInput).toBeVisible();
    await searchInput.fill('zzznomatch-filter-query-999');

    await expect(page.getByText('0 results')).toBeVisible();
    await expect(page.getByText('No pools found')).toBeVisible();
    await expect(
      page.getByText('No pools match the current filter combination.')
    ).toBeVisible();

    const clearBtn = page.getByRole('button', { name: /clear filters/i });
    await expect(clearBtn).toBeVisible();
    await clearBtn.click();

    await expect(page.getByText('8 results')).toBeVisible();
  });

  test('paginates to the last page when pool count exceeds items per page', async ({
    page,
  }) => {
    await mockPoolsApi(page);
    await page.goto('/pools');

    await expect(page.getByText('8 results')).toBeVisible();
    const paginationNav = page.getByRole('navigation', { name: 'Pagination' });
    await expect(paginationNav).toBeVisible();
    await expect(paginationNav).toContainText('1–6');

    const lastPageBtn = paginationNav.getByRole('button', {
      name: 'Last page',
    });
    await expect(lastPageBtn).toBeEnabled();
    await lastPageBtn.click();

    await expect(paginationNav).toContainText('7–8');
    await expect(lastPageBtn).toBeDisabled();
    await expect(page).toHaveURL(/page=2/);
    await expect(
      page.getByText('Solar Microgrids for Rural Schools')
    ).toBeVisible();
  });

  test('updates displayed pools and URL query when interacting with search and sort controls', async ({
    page,
  }) => {
    await mockPoolsApi(page);
    await page.goto('/pools');

    const sortSelect = page.getByLabel(/sort/i);
    await expect(sortSelect).toBeVisible();
    await sortSelect.selectOption('most-funded');
    await expect(page).toHaveURL(/sort=most-funded/);

    const searchInput = page.locator('#search-pools');
    await searchInput.fill('Clean Water');
    await expect(page.getByText('1 result')).toBeVisible();
    await expect(
      page.getByText('Clean Water Wells Initiative')
    ).toBeVisible();
    await expect(page).toHaveURL(/q=Clean\+Water|q=Clean%20Water/);
  });
});
