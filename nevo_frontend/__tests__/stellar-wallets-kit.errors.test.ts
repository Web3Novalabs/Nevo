import {
  connect,
  getPublicKey,
  signWithWallet,
} from '@/app/stellar-wallets-kit';

const mockGetAddress = jest.fn();
const mockSignMessage = jest.fn();
const mockSetWallet = jest.fn();
const mockGetSelectedWallet = jest.fn();

jest.mock('@creit.tech/stellar-wallets-kit', () => ({
  StellarWalletsKit: {
    init: jest.fn(),
    getAddress: (...args: unknown[]) => mockGetAddress(...args),
    signMessage: (...args: unknown[]) => mockSignMessage(...args),
    setWallet: (...args: unknown[]) => mockSetWallet(...args),
    getSelectedWallet: (...args: unknown[]) => mockGetSelectedWallet(...args),
  },
}));

jest.mock('@creit.tech/stellar-wallets-kit/modules/utils', () => ({
  defaultModules: jest.fn(() => []),
}));

describe('stellar-wallets-kit error paths', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    localStorage.clear();
  });

  describe('getPublicKey', () => {
    it('returns null instead of throwing when the kit throws', async () => {
      mockGetAddress.mockRejectedValue(new Error('wallet crashed'));

      await expect(getPublicKey()).resolves.toBeNull();
    });
  });

  describe('connect', () => {
    it('throws when no wallet is selected or available', async () => {
      mockGetSelectedWallet.mockReturnValue(undefined);
      mockGetAddress.mockRejectedValue(new Error('No wallet selected'));
      const onConnect = jest.fn();

      await expect(connect(onConnect)).rejects.toThrow();
      expect(onConnect).not.toHaveBeenCalled();
    });

    it('throws and never calls onConnect when the kit returns no address', async () => {
      mockGetSelectedWallet.mockReturnValue({ id: 'xbull' });
      mockGetAddress.mockResolvedValue({ address: '' });
      const onConnect = jest.fn();

      await expect(connect(onConnect)).rejects.toThrow();
      expect(onConnect).not.toHaveBeenCalled();
    });
  });

  describe('signWithWallet', () => {
    it('returns the signature on a successful signMessage call', async () => {
      mockSignMessage.mockResolvedValue({ signedMessage: 'signed-nonce' });

      await expect(signWithWallet('nonce')).resolves.toBe('signed-nonce');
      expect(mockSignMessage).toHaveBeenCalledWith(
        'nonce',
        expect.objectContaining({ address: expect.any(String) })
      );
    });

    it('surfaces kit signing errors', async () => {
      mockSignMessage.mockRejectedValue(new Error('User rejected the request'));

      await expect(signWithWallet('nonce')).rejects.toThrow(
        'User rejected the request'
      );
    });
  });
});
