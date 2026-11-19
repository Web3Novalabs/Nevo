import { getAccountBalances } from '@/lib/stellar';
import { RATE_LIMIT_EVENT } from '@/lib/rate-limit';

const mockFetch = jest.fn();
global.fetch = mockFetch;

const TESTNET_USDC_ISSUER = 'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5';
const MAINNET_USDC_ISSUER = 'GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN';

describe('getAccountBalances', () => {
  const originalNetwork = process.env.NEXT_PUBLIC_STELLAR_NETWORK;

  beforeEach(() => {
    mockFetch.mockReset();
    delete process.env.NEXT_PUBLIC_STELLAR_NETWORK;
  });

  afterAll(() => {
    if (originalNetwork === undefined) {
      delete process.env.NEXT_PUBLIC_STELLAR_NETWORK;
    } else {
      process.env.NEXT_PUBLIC_STELLAR_NETWORK = originalNetwork;
    }
  });

  it('returns parsed XLM and USDC balances', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        balances: [
          { asset_type: 'native', balance: '100.5000000' },
          {
            asset_code: 'USDC',
            asset_issuer: TESTNET_USDC_ISSUER,
            balance: '50.2500000',
          },
        ],
      }),
    });
    const result = await getAccountBalances('GABC');
    expect(result).toEqual({ xlm: '100.50', usdc: '50.25' });
  });

  it('only counts USDC when the issuer matches', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        balances: [
          { asset_type: 'native', balance: '10.0000000' },
          {
            asset_code: 'USDC',
            asset_issuer: 'GFAKEISSUER',
            balance: '999.0000000',
          },
          {
            asset_code: 'USDC',
            asset_issuer: TESTNET_USDC_ISSUER,
            balance: '25.0000000',
          },
        ],
      }),
    });
    const result = await getAccountBalances('GABC');
    expect(result).toEqual({ xlm: '10.00', usdc: '25.00' });
  });

  it('queries testnet Horizon when configured for testnet', async () => {
    process.env.NEXT_PUBLIC_STELLAR_NETWORK = 'testnet';
    mockFetch.mockResolvedValueOnce({ ok: true, json: async () => ({ balances: [] }) });
    await getAccountBalances('GABC');
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('horizon-testnet.stellar.org'),
      expect.anything()
    );
  });

  it('queries mainnet Horizon when configured for mainnet', async () => {
    process.env.NEXT_PUBLIC_STELLAR_NETWORK = 'public';
    mockFetch.mockResolvedValueOnce({ ok: true, json: async () => ({ balances: [] }) });
    await getAccountBalances('GABC');
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('horizon.stellar.org'),
      expect.anything()
    );
    expect(mockFetch).not.toHaveBeenCalledWith(
      expect.stringContaining('horizon-testnet.stellar.org'),
      expect.anything()
    );
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
