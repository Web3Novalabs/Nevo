import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock next/navigation
const pushMock = vi.fn();
const replaceMock = vi.fn();
const searchParamsMock = {
  get: vi.fn((key: string) => (key === 'next' ? '/dashboard' : null)),
};

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
    replace: replaceMock,
    back: vi.fn(),
    forward: vi.fn(),
    refresh: vi.fn(),
    prefetch: vi.fn(),
  }),
  useSearchParams: () => searchParamsMock,
  usePathname: () => '/login',
}));

// Mock wallet store
const initializeMock = vi.fn();
const connectWalletMock = vi.fn();
let walletStoreState: {
  isAuthenticated: boolean;
  loading: boolean;
  initialize: typeof initializeMock;
  connectWallet: typeof connectWalletMock;
};

vi.mock('@/store/wallet-store', () => ({
  useWalletStore: () => walletStoreState,
}));

import LoginPage from '@/app/login/page';

const renderLoginPage = () =>
  render(
    <React.Suspense fallback={<div>Loading...</div>}>
      <LoginPage />
    </React.Suspense>
  );

describe('LoginPage auth flow (#1549, #1727)', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    walletStoreState = {
      isAuthenticated: false,
      loading: false,
      initialize: initializeMock,
      connectWallet: connectWalletMock,
    };

    searchParamsMock.get.mockImplementation((key: string) =>
      key === 'next' ? '/dashboard' : null
    );

    connectWalletMock.mockResolvedValue(undefined);
  });

  it('calls the store connectWallet action and redirects to the next target', async () => {
    renderLoginPage();

    // Accept terms
    const termsCheckbox = screen.getByRole('checkbox');
    fireEvent.click(termsCheckbox);

    // Click connect button
    const connectButton = screen.getByRole('button', {
      name: /connect wallet & sign in/i,
    });
    fireEvent.click(connectButton);

    await waitFor(() => {
      expect(connectWalletMock).toHaveBeenCalledTimes(1);
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/dashboard');
    });
  });

  it('falls back to /dashboard when no next param is provided', async () => {
    searchParamsMock.get.mockImplementation(() => null);

    renderLoginPage();

    const termsCheckbox = screen.getByRole('checkbox');
    fireEvent.click(termsCheckbox);

    const connectButton = screen.getByRole('button', {
      name: /connect wallet & sign in/i,
    });
    fireEvent.click(connectButton);

    await waitFor(() => {
      expect(connectWalletMock).toHaveBeenCalledTimes(1);
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/dashboard');
    });
  });

  it('shows an error alert when connectWallet rejects', async () => {
    connectWalletMock.mockRejectedValueOnce(
      new Error('User rejected the signature request')
    );

    renderLoginPage();

    const termsCheckbox = screen.getByRole('checkbox');
    fireEvent.click(termsCheckbox);

    const connectButton = screen.getByRole('button', {
      name: /connect wallet & sign in/i,
    });
    fireEvent.click(connectButton);

    const alert = await screen.findByRole('alert');
    expect(alert).toBeInTheDocument();
    expect(alert.textContent).toMatch(/user rejected the signature request/i);

    expect(pushMock).not.toHaveBeenCalled();
  });

  it('redirects already-authenticated users away from /login', async () => {
    walletStoreState = {
      isAuthenticated: true,
      loading: false,
      initialize: initializeMock,
      connectWallet: connectWalletMock,
    };

    renderLoginPage();

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/dashboard');
    });
  });
});
