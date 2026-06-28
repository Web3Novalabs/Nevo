import { useWalletStore } from '@/src/store/walletStore';
import { disconnect } from '@/app/stellar-wallets-kit';

jest.mock('@/app/stellar-wallets-kit', () => ({
  getPublicKey: jest.fn(),
  connect: jest.fn(),
  disconnect: jest.fn(),
}));

describe('walletStore disconnectWallet', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    window.localStorage.clear();
    useWalletStore.setState({
      publicKey: 'GABC123',
      accessToken: 'jwt-token',
      balances: null,
      loading: false,
      isAuthenticated: true,
    });
    (disconnect as jest.Mock).mockResolvedValue(undefined);
  });

  it('clears the stored JWT and marks the user as unauthenticated', async () => {
    window.localStorage.setItem(
      'nevo-wallet',
      JSON.stringify({ state: { accessToken: 'jwt-token' } })
    );

    await useWalletStore.getState().disconnectWallet();

    expect(useWalletStore.getState().isAuthenticated).toBe(false);
    expect(useWalletStore.getState().accessToken).toBeNull();
    expect(window.localStorage.getItem('nevo-wallet')).toBeNull();
  });
});
