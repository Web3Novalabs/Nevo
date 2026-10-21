import type { Page, Route } from '@playwright/test';

export const TEST_PUBLIC_KEY =
  'GCGHDIXHAFZUFKELWSJXVQNWFPKAI25IJKJD6ALVHTV4WO4FNMVZ3ESS';

export interface TestPool {
  id: string;
  title: string;
  description: string;
  category: string;
  status: 'Active' | 'Completed';
  target: number;
  raised: number;
  imageColor: string;
  createdAt: string;
}

export function makePool(index: number, overrides: Partial<TestPool> = {}) {
  return {
    id: String(index),
    title: `Test Pool ${index}`,
    description: `Description for test pool ${index}`,
    category: 'Education',
    status: 'Active',
    target: 10000,
    raised: 100 * index,
    imageColor: 'bg-sky-500',
    createdAt: `2025-01-${String(index).padStart(2, '0')}`,
    ...overrides,
  } satisfies TestPool;
}

// Intercepts API fetches only, so page navigations to the same paths still load.
export async function mockApi(
  page: Page,
  path: string | RegExp,
  handler: (route: Route) => Promise<void> | void
) {
  await page.route(path, (route) => {
    const type = route.request().resourceType();
    if (type !== 'fetch' && type !== 'xhr') return route.fallback();
    return handler(route);
  });
}

// Fakes the Freighter extension's postMessage bridge and an authenticated session.
export async function mockFreighter(
  page: Page,
  options: { signError?: string } = {}
) {
  await page.route('https://horizon.stellar.org/accounts/**', (route) =>
    route.fulfill({
      json: { balances: [{ asset_type: 'native', balance: '500.0000000' }] },
    })
  );

  await page.addInitScript(
    ({ publicKey, signError }) => {
      Object.assign(window, { freighter: true });
      window.localStorage.setItem('nevo_jwt', 'e2e-token');
      window.addEventListener('message', (event) => {
        const data = event.data;
        if (
          event.source !== window ||
          data?.source !== 'FREIGHTER_EXTERNAL_MSG_REQUEST'
        ) {
          return;
        }
        const response: Record<string, unknown> = {
          source: 'FREIGHTER_EXTERNAL_MSG_RESPONSE',
          messagedId: data.messageId,
        };
        if (
          data.type === 'REQUEST_PUBLIC_KEY' ||
          data.type === 'REQUEST_ACCESS'
        ) {
          response.publicKey = publicKey;
        }
        if (data.type === 'SUBMIT_TRANSACTION') {
          Object.assign(
            response,
            signError
              ? { apiError: signError }
              : {
                  signedTransaction: data.transactionXdr,
                  signerAddress: publicKey,
                }
          );
        }
        window.postMessage(response, window.location.origin);
      });
    },
    { publicKey: TEST_PUBLIC_KEY, signError: options.signError }
  );
}
