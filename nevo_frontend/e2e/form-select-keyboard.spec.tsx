import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { expect, test } from '@playwright/test';
import { Select } from '../components/form/Select';

test('Select supports ArrowDown and Enter keyboard selection', async ({
  page,
  browserName,
}) => {
  test.skip(browserName !== 'chromium', 'Run native select keyboard check in Chromium');

  const markup = renderToStaticMarkup(
    <Select
      id="category"
      label="Category"
      options={[
        { label: 'Education', value: 'education' },
        { label: 'Health', value: 'health' },
      ]}
    />
  );
  await page.setContent(markup);

  const select = page.getByRole('combobox', { name: 'Category' });
  await select.focus();
  await page.keyboard.press('ArrowDown');
  await expect(select).toHaveValue('health');
  await page.keyboard.press('Enter');
  await expect(select).toHaveValue('health');
});
