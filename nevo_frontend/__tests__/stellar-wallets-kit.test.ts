import * as freighter from '@stellar/freighter-api';
import { disconnect } from '@/app/stellar-wallets-kit';
import { useWalletStore } from '@/src/store/walletStore';

jest.mock('@stellar/freighter-api', () => ({
  isConnected: jest.fn(),
  requestAccess: jest.fn(),
  getAddress: jest.fn(),
  signMessage: jest.fn(),
}));

jest.mock('@/lib/api-client', () => ({
  fetchAuthChallenge: jest.fn(),
  verifyAuthSignature: jest.fn(),
}));

jest.mock('@/lib/stellar', () => ({
  getAccountBalances: jest.fn(),
}));

const CONNECTED_STATE = {
  publicKey: 'GABC123',
  accessToken: 'jwt-token',
  balances: { xlm: '10.00', usdc: '0.00' },
  loading: false,
  isAuthenticated: true,
};

describe('stellar-wallets-kit disconnect', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    window.localStorage.clear();
    useWalletStore.setState(CONNECTED_STATE);
  });

  it('resolves without throwing and performs no wallet-side action', async () => {
    await expect(disconnect()).resolves.toBeUndefined();

    expect(freighter.isConnected).not.toHaveBeenCalled();
    expect(freighter.requestAccess).not.toHaveBeenCalled();
    expect(freighter.getAddress).not.toHaveBeenCalled();
    expect(freighter.signMessage).not.toHaveBeenCalled();
  });

  it('does not clear local wallet state on its own', async () => {
    window.localStorage.setItem('nevo_jwt', 'jwt-token');

    await disconnect();

    expect(useWalletStore.getState()).toMatchObject(CONNECTED_STATE);
    expect(window.localStorage.getItem('nevo_jwt')).toBe('jwt-token');
  });

  it('leaves clearing local state to the wallet store disconnectWallet action', async () => {
    window.localStorage.setItem('nevo_jwt', 'jwt-token');

    await useWalletStore.getState().disconnectWallet();

    expect(useWalletStore.getState()).toMatchObject({
      publicKey: null,
      accessToken: null,
      balances: null,
      isAuthenticated: false,
    });
    expect(window.localStorage.getItem('nevo_jwt')).toBeNull();
    expect(freighter.isConnected).not.toHaveBeenCalled();
  });
});
