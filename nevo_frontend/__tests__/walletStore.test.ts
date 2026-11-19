import { useWalletStore } from '@/src/store/walletStore';
import { connect, disconnect, getPublicKey, signWithWallet } from '@/app/stellar-wallets-kit';
import { fetchAuthChallenge, verifyAuthSignature } from '@/lib/api-client';
import { getAccountBalances } from '@/lib/stellar';
import { clearToken, getToken, setToken } from '@/lib/jwt-storage';

jest.mock('@/app/stellar-wallets-kit', () => ({
  getPublicKey: jest.fn(),
  connect: jest.fn(),
  disconnect: jest.fn(),
  signWithWallet: jest.fn(),
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
    setToken('jwt-token');
    useWalletStore.setState({
      publicKey: 'GABC123',
      balances: null,
      loading: false,
    });
    (disconnect as jest.Mock).mockResolvedValue(undefined);
  });

  it('clears the stored JWT and marks the user as unauthenticated', async () => {
    window.localStorage.setItem(
      'nevo-wallet',
      JSON.stringify({ state: { publicKey: 'GABC123' } })
    );

    await useWalletStore.getState().disconnectWallet();

    expect(useWalletStore.getState().isAuthenticated).toBe(false);
    expect(getToken()).toBeNull();
    expect(window.localStorage.getItem('nevo-wallet')).toBeNull();
  });

  it('clearAuth removes the token and all auth data', () => {
    useWalletStore.getState().clearAuth();

    expect(getToken()).toBeNull();
    expect(useWalletStore.getState().publicKey).toBeNull();
    expect(useWalletStore.getState().isAuthenticated).toBe(false);
    expect(window.localStorage.getItem('nevo-wallet')).toBeNull();
  });

  it('derives isAuthenticated from the token module', () => {
    expect(useWalletStore.getState().isAuthenticated).toBe(true);

    clearToken();

    expect(useWalletStore.getState().isAuthenticated).toBe(false);
  });

  it('does not persist isAuthenticated in the nevo-wallet store', async () => {
    await useWalletStore.getState().disconnectWallet();
    useWalletStore.setState({ publicKey: 'GABC123' });

    const persisted = window.localStorage.getItem('nevo-wallet');
    if (persisted) {
      const parsed = JSON.parse(persisted);
      expect(parsed.state).not.toHaveProperty('isAuthenticated');
      expect(parsed.state).not.toHaveProperty('accessToken');
    }
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
      balances: null,
      loading: false,
    });

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
    expect(getToken()).toBe(ACCESS_TOKEN);
    expect(useWalletStore.getState().isAuthenticated).toBe(true);
  });

  it('is the single login path: challenge, sign, verify, then token stored via jwt-storage', async () => {
    const setTokenSpy = jest.spyOn(
      jest.requireActual('@/lib/jwt-storage'),
      'setToken'
    );

    await useWalletStore.getState().connectWallet();

    expect(fetchAuthChallenge).toHaveBeenCalledTimes(1);
    expect(signWithWallet).toHaveBeenCalledTimes(1);
    expect(verifyAuthSignature).toHaveBeenCalledTimes(1);
    expect(setTokenSpy).toHaveBeenCalledWith(ACCESS_TOKEN);
    expect(getToken()).toBe(ACCESS_TOKEN);

    setTokenSpy.mockRestore();
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
    expect(getToken()).toBeNull();
  });
});
