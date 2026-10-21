import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ConnectWallet from '@/components/ConnectWallet';
import { isFreighterInstalled } from '@/app/stellar-wallets-kit';

jest.mock('@/app/stellar-wallets-kit', () => ({
  isFreighterInstalled: jest.fn(),
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

const mockIsFreighterInstalled = jest.mocked(isFreighterInstalled);

describe('ConnectWallet', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockWalletState.publicKey = null;
    mockWalletState.loading = false;
    mockIsFreighterInstalled.mockReturnValue(true);
  });

  it('renders the not installed message when Freighter is missing', async () => {
    mockIsFreighterInstalled.mockReturnValue(false);

    render(<ConnectWallet />);

    expect(
      await screen.findByText(/Freighter wallet not found/i)
    ).toBeInTheDocument();
    expect(
      screen.getByRole('link', { name: 'Install Freighter' })
    ).toHaveAttribute('href', 'https://www.freighter.app/');
  });

  it('connects the wallet and shows a verifying state while pending', async () => {
    let resolveConnection: () => void = () => undefined;
    mockWalletState.connectWallet.mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          resolveConnection = resolve;
        })
    );

    render(<ConnectWallet />);
    const connectButton = await screen.findByRole('button', {
      name: 'Connect Wallet',
    });

    await userEvent.click(connectButton);

    expect(mockWalletState.connectWallet).toHaveBeenCalledTimes(1);
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

    expect(
      await screen.findByText('Signature request was rejected.')
    ).toBeInTheDocument();
  });
});
