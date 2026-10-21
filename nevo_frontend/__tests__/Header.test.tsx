import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Header } from '@/components/Header';
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

beforeAll(() => {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: jest.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
    })),
  });
});

describe('Header', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockIsFreighterInstalled.mockReturnValue(true);
    mockWalletState.connectWallet.mockResolvedValue(undefined);
  });

  it('calls connectWallet when the Connect Wallet button is clicked', async () => {
    render(<Header />);

    await userEvent.click(
      await screen.findByRole('button', { name: 'Connect Wallet' })
    );

    expect(mockWalletState.connectWallet).toHaveBeenCalledTimes(1);
  });

  it('keeps the MobileMenu closed until the menu button is clicked', () => {
    render(<Header />);

    expect(screen.getByRole('button', { name: 'Open menu' })).toHaveAttribute(
      'aria-expanded',
      'false'
    );
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('renders MobileMenu when the menu button is clicked', async () => {
    render(<Header />);

    await userEvent.click(screen.getByRole('button', { name: 'Open menu' }));

    expect(screen.getByRole('button', { name: 'Menu open' })).toHaveAttribute(
      'aria-expanded',
      'true'
    );
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  it('hides MobileMenu again when it is closed', async () => {
    render(<Header />);

    await userEvent.click(screen.getByRole('button', { name: 'Open menu' }));
    await userEvent.click(screen.getByRole('button', { name: 'Close menu' }));

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Open menu' })).toHaveAttribute(
      'aria-expanded',
      'false'
    );
  });
});
