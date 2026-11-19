import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ConnectWallet from '@/components/ConnectWallet';
import { SUPPORTED_WALLETS } from '@/app/stellar-wallets-kit';

jest.mock('@/app/stellar-wallets-kit', () => ({
  SUPPORTED_WALLETS: [
    { id: 'freighter', name: 'Freighter' },
    { id: 'xbull', name: 'xBull' },
    { id: 'albedo', name: 'Albedo' },
    { id: 'lobstr', name: 'Lobstr' },
  ],
}));

const mockWalletState = {
  publicKey: null,
  balances: null,
  loading: false,
  connectWallet: jest.fn(),
  disconnectWallet: jest.fn(),
  initialize: jest.fn(),
};

jest.mock('@/src/store/walletStore', () => ({
  useWalletStore: jest.fn(() => mockWalletState),
}));

describe('ConnectWallet', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockWalletState.publicKey = null;
    mockWalletState.loading = false;
  });

  it('opens a wallet picker modal listing multiple wallets', async () => {
    render(<ConnectWallet />);

    await userEvent.click(
      await screen.findByRole('button', { name: 'Connect Wallet' })
    );

    const dialog = await screen.findByRole('dialog');
    expect(dialog).toBeInTheDocument();
    expect(SUPPORTED_WALLETS.length).toBeGreaterThan(1);
    for (const wallet of SUPPORTED_WALLETS) {
      expect(
        screen.getByRole('button', { name: wallet.name })
      ).toBeInTheDocument();
    }
  });

  it('connects the selected wallet and shows a verifying state while pending', async () => {
    let resolveConnection: () => void = () => undefined;
    mockWalletState.connectWallet.mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          resolveConnection = resolve;
        })
    );

    render(<ConnectWallet />);
    await userEvent.click(
      await screen.findByRole('button', { name: 'Connect Wallet' })
    );

    await userEvent.click(await screen.findByRole('button', { name: 'xBull' }));

    expect(mockWalletState.connectWallet).toHaveBeenCalledTimes(1);
    expect(mockWalletState.connectWallet).toHaveBeenCalledWith('xbull');
    expect(screen.getByRole('button', { name: /Verifying…/ })).toBeDisabled();

    resolveConnection();
    await waitFor(() =>
      expect(
        screen.getByRole('button', { name: 'Connect Wallet' })
      ).toBeEnabled()
    );
  });

  it('renders a friendly message when the signature request is rejected', async () => {
    mockWalletState.connectWallet.mockRejectedValue(
      new Error('User rejected the request')
    );

    render(<ConnectWallet />);
    await userEvent.click(
      await screen.findByRole('button', { name: 'Connect Wallet' })
    );
    await userEvent.click(
      await screen.findByRole('button', { name: 'Freighter' })
    );

    expect(
      await screen.findByText('Signature request was rejected.')
    ).toBeInTheDocument();
  });
});
