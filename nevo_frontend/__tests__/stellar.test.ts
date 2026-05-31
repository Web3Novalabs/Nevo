import { getAccountBalances } from '@/lib/stellar';
import { RATE_LIMIT_EVENT } from '@/lib/rate-limit';

const mockFetch = jest.fn();
global.fetch = mockFetch;

describe('getAccountBalances', () => {
  beforeEach(() => mockFetch.mockReset());

  it('returns parsed XLM and USDC balances', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        balances: [
          { asset_type: 'native', balance: '100.5000000' },
          { asset_code: 'USDC', balance: '50.2500000' },
        ],
      }),
    });
    const result = await getAccountBalances('GABC');
    expect(result).toEqual({ xlm: '100.50', usdc: '50.25' });
  });

  it('returns zeros when response is not ok', async () => {
    mockFetch.mockResolvedValueOnce({ ok: false });
    expect(await getAccountBalances('GABC')).toEqual({ xlm: '0', usdc: '0' });
  });

  it('returns zeros on network error', async () => {
    mockFetch.mockRejectedValueOnce(new Error('Network error'));
    expect(await getAccountBalances('GABC')).toEqual({ xlm: '0', usdc: '0' });
  });

  it('returns zeros when balances array is missing', async () => {
    mockFetch.mockResolvedValueOnce({ ok: true, json: async () => ({}) });
    expect(await getAccountBalances('GABC')).toEqual({ xlm: '0', usdc: '0' });
  });

  it('notifies users when Horizon returns a rate limit response', async () => {
    const listener = jest.fn();
    window.addEventListener(RATE_LIMIT_EVENT, listener);
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 429,
      headers: { get: () => '5' },
    });

    expect(await getAccountBalances('GABC')).toEqual({ xlm: '0', usdc: '0' });
    expect(listener).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: expect.objectContaining({
          retryAfterMs: 5000,
        }),
      })
    );

    window.removeEventListener(RATE_LIMIT_EVENT, listener);
  });
});
