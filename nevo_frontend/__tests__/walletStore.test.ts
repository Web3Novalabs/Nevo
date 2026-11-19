import { useWalletStore } from '@/src/store/walletStore';
import {
  connect,
  disconnect,
  getPublicKey,
  signWithWallet,
  setWallet,
} from '@/app/stellar-wallets-kit';
import { fetchAuthChallenge, verifyAuthSignature } from '@/lib/api-client';
import { getAccountBalances } from '@/lib/stellar';

jest.mock('@/app/stellar-wallets-kit', () => ({
  getPublicKey: jest.fn(),
  connect: jest.fn(),
  disconnect: jest.fn(),
  signWithWallet: jest.fn(),
  setWallet: jest.fn(),
}));

jest.mock('@/lib/api-client', () => ({
  fetchAuthChallenge: jest.fn(),
  verifyAuthSignature: jest.fn(),
}));

jest.mock('@/lib/stellar', () => ({
  getAccountBalances: jest.fn(),
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
      walletId: 'xbull',
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

  it('clears the selected wallet id on disconnect', async () => {
    await useWalletStore.getState().disconnectWallet();

    expect(useWalletStore.getState().walletId).toBeNull();
    expect(disconnect).toHaveBeenCalledTimes(1);
  });
});

describe('walletStore connectWallet SEP-10 auth', () => {
  const PUBLIC_KEY = 'GABC123PUBLIC';
  const NONCE = 'challenge-nonce-xyz';
  const SIGNATURE = 'signed-nonce-hex';
  const ACCESS_TOKEN = 'jwt-access-token';

  beforeEach(() => {
    jest.clearAllMocks();
    window.localStorage.clear();
    useWalletStore.setState({
      publicKey: null,
      accessToken: null,
      balances: null,
      loading: false,
      isAuthenticated: false,
      walletId: null,
    });

    (setWallet as jest.Mock).mockResolvedValue(undefined);
    (connect as jest.Mock).mockImplementation(async (onConnect) => {
      await onConnect();
    });
    (getPublicKey as jest.Mock).mockResolvedValue(PUBLIC_KEY);
    (getAccountBalances as jest.Mock).mockResolvedValue(null);
    (fetchAuthChallenge as jest.Mock).mockResolvedValue({
      nonce: NONCE,
      expiresAt: 9999999999,
    });
    (signWithWallet as jest.Mock).mockResolvedValue(SIGNATURE);
    (verifyAuthSignature as jest.Mock).mockResolvedValue({
      accessToken: ACCESS_TOKEN,
    });
  });

  it('completes SEP-10 auth flow and sets isAuthenticated to true', async () => {
    await useWalletStore.getState().connectWallet();

    expect(fetchAuthChallenge).toHaveBeenCalledWith(PUBLIC_KEY);
    expect(signWithWallet).toHaveBeenCalledWith(NONCE);
    expect(verifyAuthSignature).toHaveBeenCalledWith(PUBLIC_KEY, NONCE, SIGNATURE);

    expect(useWalletStore.getState().publicKey).toBe(PUBLIC_KEY);
    expect(useWalletStore.getState().accessToken).toBe(ACCESS_TOKEN);
    expect(useWalletStore.getState().isAuthenticated).toBe(true);
  });

  it('calls the onSuccess callback after successful auth', async () => {
    const onSuccess = jest.fn();
    await useWalletStore.getState().connectWallet(onSuccess);
    expect(onSuccess).toHaveBeenCalledTimes(1);
  });

  it('does not set isAuthenticated if verifyAuthSignature throws', async () => {
    (verifyAuthSignature as jest.Mock).mockRejectedValue(new Error('Verify failed'));

    await expect(useWalletStore.getState().connectWallet()).rejects.toThrow('Verify failed');

    expect(useWalletStore.getState().isAuthenticated).toBe(false);
    expect(useWalletStore.getState().accessToken).toBeNull();
  });

  it('connects through the selected wallet and persists its id', async () => {
    await useWalletStore.getState().connectWallet(undefined, 'albedo');

    expect(setWallet).toHaveBeenCalledWith('albedo');
    expect(connect).toHaveBeenCalledTimes(1);
    expect(useWalletStore.getState().walletId).toBe('albedo');
    expect(useWalletStore.getState().isAuthenticated).toBe(true);
  });

  it('reuses the previously persisted wallet id when none is passed', async () => {
    useWalletStore.setState({ walletId: 'lobstr' });

    await useWalletStore.getState().connectWallet();

    expect(setWallet).toHaveBeenCalledWith('lobstr');
    expect(useWalletStore.getState().walletId).toBe('lobstr');
  });

  it('does not persist a wallet id when the connection fails', async () => {
    (connect as jest.Mock).mockRejectedValue(new Error('User rejected'));

    await expect(
      useWalletStore.getState().connectWallet(undefined, 'xbull')
    ).rejects.toThrow('User rejected');

    expect(useWalletStore.getState().walletId).toBeNull();
    expect(useWalletStore.getState().isAuthenticated).toBe(false);
  });
});
