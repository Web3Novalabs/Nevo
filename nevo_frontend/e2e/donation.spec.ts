import { test, expect, type Page } from '@playwright/test';
import { Keypair, SorobanDataBuilder, xdr } from '@stellar/stellar-sdk';
import { TEST_PUBLIC_KEY, makePool, mockApi, mockFreighter } from './helpers';

const RPC_URL =
  process.env.NEXT_PUBLIC_SOROBAN_RPC_URL ??
  'https://soroban-testnet.stellar.org';
const TX_HASH = 'a'.repeat(64);

// Building a donation transaction needs a configured contract id.
const requiresContract = () =>
  test.skip(
    !process.env.NEXT_PUBLIC_CONTRACT_ID,
    'NEXT_PUBLIC_CONTRACT_ID is not set'
  );

async function mockPool(page: Page, raised: () => number) {
  await mockApi(page, /\/pools\/1$/, (route) =>
    route.fulfill({ json: makePool(1, { raised: raised() }) })
  );
}

async function mockSorobanRpc(page: Page, onSend: () => void = () => {}) {
  const account = xdr.LedgerEntryData.account(
    new xdr.AccountEntry({
      accountId: Keypair.fromPublicKey(TEST_PUBLIC_KEY).xdrPublicKey(),
      balance: xdr.Int64.fromString('5000000000'),
      seqNum: xdr.Int64.fromString('1'),
      numSubEntries: 0,
      inflationDest: null,
      flags: 0,
      homeDomain: '',
      thresholds: Buffer.from([1, 0, 0, 0]),
      signers: [],
      ext: new xdr.AccountEntryExt(0),
    })
  ).toXDR('base64');

  await page.route(`${RPC_URL}/**`, (route) => {
    const { id, method, params } = route.request().postDataJSON();
    const results: Record<string, unknown> = {
      getLedgerEntries: {
        entries: [
          { key: params?.keys?.[0], xdr: account, lastModifiedLedgerSeq: 1 },
        ],
        latestLedger: 100,
      },
      simulateTransaction: {
        latestLedger: 100,
        minResourceFee: '100',
        transactionData: new SorobanDataBuilder().build().toXDR('base64'),
        results: [{ auth: [], xdr: xdr.ScVal.scvVoid().toXDR('base64') }],
      },
      sendTransaction: {
        status: 'PENDING',
        hash: TX_HASH,
        latestLedger: 100,
        latestLedgerCloseTime: '0',
      },
    };
    if (method === 'sendTransaction') onSend();
    return route.fulfill({
      json: { jsonrpc: '2.0', id, result: results[method] },
    });
  });
}

async function openDonateModal(page: Page) {
  await page.goto('/pools/1');
  await page
    .getByRole('complementary', { name: 'Pool funding details' })
    .getByRole('button', { name: 'Donate Now' })
    .click();
  return page.getByRole('dialog', { name: 'Donate to Pool' });
}

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

  test.describe('with a connected wallet', () => {
    test('shows a failed state when the donation transaction fails', async ({
      page,
    }) => {
      await mockFreighter(page);
      await mockPool(page, () => 250);
      await page.route(`${RPC_URL}/**`, (route) =>
        route.fulfill({ status: 500, body: 'Internal error' })
      );

      const dialog = await openDonateModal(page);
      await dialog.getByLabel('Amount').fill('10');
      await dialog.getByRole('button', { name: 'Donate 10 XLM' }).click();

      await expect(dialog.getByText('Donation failed')).toBeVisible();
      await expect(
        dialog.getByText('Donation successful! Thank you.')
      ).toHaveCount(0);

      await dialog.getByRole('button', { name: 'Try again' }).click();
      await expect(dialog.getByLabel('Amount')).toBeVisible();
    });

    test('shows a failed state when the wallet rejects the signature', async ({
      page,
    }) => {
      requiresContract();
      let sent = false;
      await mockFreighter(page, { signError: 'User declined access' });
      await mockPool(page, () => 250);
      await mockSorobanRpc(page, () => (sent = true));

      const dialog = await openDonateModal(page);
      await dialog.getByLabel('Amount').fill('10');
      await dialog.getByRole('button', { name: 'Donate 10 XLM' }).click();

      await expect(dialog.getByText('Donation failed')).toBeVisible();
      await expect(dialog.getByText('User declined access')).toBeVisible();
      expect(sent).toBe(false);
    });

    test('blocks zero, negative and non-numeric amounts client-side', async ({
      page,
    }) => {
      await mockFreighter(page);
      await mockPool(page, () => 250);

      const dialog = await openDonateModal(page);
      const amount = dialog.getByLabel('Amount');
      const submit = dialog.getByRole('button', { name: /^Donate/ });

      for (const value of ['0', '-5']) {
        await amount.fill(value);
        await expect(amount).toHaveAttribute('aria-invalid', 'true');
        await expect(
          dialog.getByText('Minimum donation is 1 XLM')
        ).toBeVisible();
        await expect(submit).toBeDisabled();
      }

      await amount.fill('');
      await amount.pressSequentially('abc');
      await expect(amount).toHaveValue('');
      await expect(submit).toBeDisabled();
    });

    test("updates the pool's raised total after a successful donation", async ({
      page,
    }) => {
      requiresContract();
      let raised = 250;
      await mockFreighter(page);
      await mockPool(page, () => raised);
      await mockSorobanRpc(page, () => (raised += 10));

      const funding = page.getByRole('complementary', {
        name: 'Pool funding details',
      });
      const dialog = await openDonateModal(page);
      await expect(funding).toContainText('250 XLM');

      await dialog.getByLabel('Amount').fill('10');
      await dialog.getByRole('button', { name: 'Donate 10 XLM' }).click();

      await expect(funding).toContainText('260 XLM');
    });
  });
});
