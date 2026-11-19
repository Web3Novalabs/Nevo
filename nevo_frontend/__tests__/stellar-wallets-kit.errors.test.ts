import {
  isConnected,
  requestAccess,
  getAddress,
  signMessage,
} from '@stellar/freighter-api';
import {
  connect,
  getPublicKey,
  signWithWallet,
} from '@/app/stellar-wallets-kit';

jest.mock('@stellar/freighter-api', () => ({
  isConnected: jest.fn(),
  requestAccess: jest.fn(),
  getAddress: jest.fn(),
  signMessage: jest.fn(),
}));

const mockIsConnected = isConnected as jest.Mock;
const mockRequestAccess = requestAccess as jest.Mock;
const mockGetAddress = getAddress as jest.Mock;
const mockSignMessage = signMessage as jest.Mock;

describe('stellar-wallets-kit error paths', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('getPublicKey', () => {
    it('returns null instead of throwing when isConnected() throws', async () => {
      mockIsConnected.mockRejectedValue(new Error('extension crashed'));

      await expect(getPublicKey()).resolves.toBeNull();
      expect(mockGetAddress).not.toHaveBeenCalled();
    });
  });

  describe('connect', () => {
    it('throws when Freighter is not connected', async () => {
      mockIsConnected.mockResolvedValue({ isConnected: false });
      const onConnect = jest.fn();

      await expect(connect(onConnect)).rejects.toThrow(
        'Freighter extension is not installed.'
      );
      expect(mockRequestAccess).not.toHaveBeenCalled();
      expect(onConnect).not.toHaveBeenCalled();
    });

    it('throws access.error and never calls onConnect when requestAccess fails', async () => {
      mockIsConnected.mockResolvedValue({ isConnected: true });
      mockRequestAccess.mockResolvedValue({
        address: '',
        error: 'User declined access',
      });
      const onConnect = jest.fn();

      await expect(connect(onConnect)).rejects.toThrow('User declined access');
      expect(onConnect).not.toHaveBeenCalled();
    });
  });

  describe('signWithWallet', () => {
    it('returns result.signature on a successful signMessage call', async () => {
      mockSignMessage.mockResolvedValue({ signature: 'signed-nonce' });

      await expect(signWithWallet('nonce')).resolves.toBe('signed-nonce');
      expect(mockSignMessage).toHaveBeenCalledWith('nonce');
    });

    it('throws the error message when signMessage returns an error', async () => {
      mockSignMessage.mockResolvedValue({
        signature: 'ignored',
        error: 'User rejected the request',
      });

      await expect(signWithWallet('nonce')).rejects.toThrow(
        'User rejected the request'
      );
    });
  });
});
