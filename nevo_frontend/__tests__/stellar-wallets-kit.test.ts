import {
  StellarWalletsKit,
  WalletNetwork,
  allowAllModules,
  FREIGHTER_ID,
  XBULL_ID,
  ALBEDO_ID,
} from '@creit.tech/stellar-wallets-kit';
import {
  connect,
  disconnect,
  getPublicKey,
  signWithWallet,
  setWallet,
  getSelectedWalletId,
} from '@/app/stellar-wallets-kit';
import { useWalletStore } from '@/src/store/walletStore';

jest.mock('@creit.tech/stellar-wallets-kit', () => {
  const kit = {
    openModal: jest.fn(),
    setWallet: jest.fn(),
    getAddress: jest.fn(),
    signMessage: jest.fn(),
    signTransaction: jest.fn(),
  };
  return {
    StellarWalletsKit: {
      init: jest.fn(() => kit),
    },
    WalletNetwork: { PUBLIC: 'PUBLIC', TESTNET: 'TESTNET' },
    allowAllModules: jest.fn(() => []),
    FREIGHTER_ID: 'freighter',
    XBULL_ID: 'xbull',
    ALBEDO_ID: 'albedo',
    LOBSTR_ID: 'lobstr',
    __kit: kit,
  };
});

const kit = (jest.requireMock('@creit.tech/stellar-wallets-kit') as any).__kit;

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

describe('stellar-wallets-kit connect', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    window.localStorage.clear();
    useWalletStore.setState({ ...CONNECTED_STATE, publicKey: null, accessToken: null });
  });

  it('initializes the kit with all modules', async () => {
    kit.openModal.mockImplementation(({ onWalletSelected }: any) =>
      onWalletSelected({ id: XBULL_ID }),
    );
    kit.getAddress.mockResolvedValue({ address: 'GXBULL123' });

    await connect();

    expect(StellarWalletsKit.init).toHaveBeenCalledWith(
      expect.objectContaining({
        network: WalletNetwork.PUBLIC,
        modules: expect.any(Array),
      }),
    );
    expect(allowAllModules).toHaveBeenCalled();
  });

  it('connects with xBull and returns the public key', async () => {
    kit.openModal.mockImplementation(({ onWalletSelected }: any) =>
      onWalletSelected({ id: XBULL_ID }),
    );
    kit.getAddress.mockResolvedValue({ address: 'GXBULL123' });

    const publicKey = await connect();

    expect(kit.setWallet).toHaveBeenCalledWith(XBULL_ID);
    expect(publicKey).toBe('GXBULL123');
    expect(useWalletStore.getState().publicKey).toBe('GXBULL123');
  });

  it('connects with Albedo and returns the public key', async () => {
    kit.openModal.mockImplementation(({ onWalletSelected }: any) =>
      onWalletSelected({ id: ALBEDO_ID }),
    );
    kit.getAddress.mockResolvedValue({ address: 'GALBEDO456' });

    const publicKey = await connect();

    expect(kit.setWallet).toHaveBeenCalledWith(ALBEDO_ID);
    expect(publicKey).toBe('GALBEDO456');
    expect(useWalletStore.getState().publicKey).toBe('GALBEDO456');
  });

  it('persists the selected wallet', async () => {
    kit.openModal.mockImplementation(({ onWalletSelected }: any) =>
      onWalletSelected({ id: ALBEDO_ID }),
    );
    kit.getAddress.mockResolvedValue({ address: 'GALBEDO456' });

    await connect();

    expect(window.localStorage.getItem('nevo_wallet_id')).toBe(ALBEDO_ID);
    expect(getSelectedWalletId()).toBe(ALBEDO_ID);
  });

  it('rejects when no wallet is selected', async () => {
    kit.openModal.mockImplementation(({ onClosed }: any) => onClosed && onClosed());

    await expect(connect()).rejects.toThrow(/wallet/i);
    expect(kit.getAddress).not.toHaveBeenCalled();
  });

  it('rejects when the selected wallet is not installed', async () => {
    kit.openModal.mockImplementation(({ onWalletSelected }: any) =>
      onWalletSelected({ id: XBULL_ID }),
    );
    kit.getAddress.mockRejectedValue(new Error('Wallet not installed'));

    await expect(connect()).rejects.toThrow(/not installed/i);
  });
});

describe('stellar-wallets-kit getPublicKey', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    window.localStorage.clear();
  });

  it('returns the address from the active wallet', async () => {
    kit.getAddress.mockResolvedValue({ address: 'GXBULL123' });

    await expect(getPublicKey()).resolves.toBe('GXBULL123');
  });

  it('throws when the wallet is unavailable', async () => {
    kit.getAddress.mockRejectedValue(new Error('Wallet not installed'));

    await expect(getPublicKey()).rejects.toThrow(/not installed/i);
  });
});

describe('stellar-wallets-kit signWithWallet', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    window.localStorage.clear();
  });

  it('signs a SEP-53 message with xBull', async () => {
    setWallet(XBULL_ID);
    kit.signMessage.mockResolvedValue({ signedMessage: 'sig-xbull' });

    const signature = await signWithWallet('hello', 'GXBULL123');

    expect(kit.signMessage).toHaveBeenCalledWith('hello', {
      address: 'GXBULL123',
    });
    expect(signature).toBe('sig-xbull');
  });

  it('signs a SEP-53 message with Albedo', async () => {
    setWallet(ALBEDO_ID);
    kit.signMessage.mockResolvedValue({ signedMessage: 'sig-albedo' });

    const signature = await signWithWallet('hello', 'GALBEDO456');

    expect(kit.signMessage).toHaveBeenCalledWith('hello', {
      address: 'GALBEDO456',
    });
    expect(signature).toBe('sig-albedo');
  });

  it('throws when the wallet cannot sign messages', async () => {
    setWallet(XBULL_ID);
    kit.signMessage.mockRejectedValue(new Error('Signing not supported'));

    await expect(signWithWallet('hello', 'GXBULL123')).rejects.toThrow(
      /signing not supported/i,
    );
  });
});

describe('stellar-wallets-kit disconnect', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    window.localStorage.clear();
    useWalletStore.setState(CONNECTED_STATE);
  });

  it('resolves without throwing and performs no wallet-side action', async () => {
    await expect(disconnect()).resolves.toBeUndefined();

    expect(kit.openModal).not.toHaveBeenCalled();
    expect(kit.setWallet).not.toHaveBeenCalled();
    expect(kit.getAddress).not.toHaveBeenCalled();
    expect(kit.signMessage).not.toHaveBeenCalled();
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
    expect(kit.openModal).not.toHaveBeenCalled();
  });
});
