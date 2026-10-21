import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock next/navigation
const pushMock = vi.fn();
const replaceMock = vi.fn();
const searchParamsMock = {
  get: vi.fn((key: string) => (key === 'from' ? '/dashboard' : null)),
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
const setAccessTokenMock = vi.fn();
let walletStoreState: {
  isAuthenticated: boolean;
  loading: boolean;
  initialize: typeof initializeMock;
  setAccessToken: typeof setAccessTokenMock;
};

vi.mock('@/store/wallet-store', () => ({
  useWalletStore: () => walletStoreState,
}));

// Mock stellar-wallets-kit
const connectMock = vi.fn();
const signWithWalletMock = vi.fn();
const getPublicKeyMock = vi.fn();

vi.mock('@/app/stellar-wallets-kit', () => ({
  connect: (...args: unknown[]) => connectMock(...args),
  signWithWallet: (...args: unknown[]) => signWithWalletMock(...args),
  getPublicKey: (...args: unknown[]) => getPublicKeyMock(...args),
}));

// Mock api-client
const getMock = vi.fn();
const postMock = vi.fn();

vi.mock('@/lib/api-client', () => ({
  apiClient: {
    get: (...args: unknown[]) => getMock(...args),
    post: (...args: unknown[]) => postMock(...args),
  },
}));

// Mock errors helper to keep parseApiError behavior predictable
vi.mock('@/lib/errors', () => ({
  parseApiError: (err: unknown) => {
    if (err instanceof Error) return err.message;
    if (typeof err === 'string') return err;
    if (err && typeof err === 'object' && 'message' in err) {
      return String((err as { message: unknown }).message);
    }
    return 'Something went wrong';
  },
}));

import LoginPage from '@/app/login/page';

const renderLoginPage = () =>
  render(
    <React.Suspense fallback={<div>Loading...</div>}>
      <LoginPage />
    </React.Suspense>
  );

describe('LoginPage auth flow (#1549)', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    walletStoreState = {
      isAuthenticated: false,
      loading: false,
      initialize: initializeMock,
      setAccessToken: setAccessTokenMock,
    };

    searchParamsMock.get.mockImplementation((key: string) =>
      key === 'from' ? '/dashboard' : null
    );

    getPublicKeyMock.mockResolvedValue('GABCDEF1234567890');
    signWithWalletMock.mockResolvedValue('signed-transaction-xdr');
    getMock.mockResolvedValue({ data: { challenge: 'challenge-xdr' } });
    postMock.mockResolvedValue({ data: { token: 'access-token-123' } });

    connectMock.mockImplementation(async (callback: () => Promise<void>) => {
      await callback();
    });
  });

  it('completes wallet connect, verifies auth, and redirects to the from target', async () => {
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
      expect(connectMock).toHaveBeenCalled();
    });

    await waitFor(() => {
      expect(postMock).toHaveBeenCalledWith(
        '/auth/verify',
        expect.objectContaining({
          publicKey: 'GABCDEF1234567890',
          signedChallenge: 'signed-transaction-xdr',
        })
      );
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/dashboard');
    });
  });

  it('shows an error alert when the user rejects the signature', async () => {
    signWithWalletMock.mockRejectedValueOnce(
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

  it('shows an error alert when the wallet connection is rejected', async () => {
    connectMock.mockRejectedValueOnce(new Error('Connection rejected by user'));

    renderLoginPage();

    const termsCheckbox = screen.getByRole('checkbox');
    fireEvent.click(termsCheckbox);

    const connectButton = screen.getByRole('button', {
      name: /connect wallet & sign in/i,
    });
    fireEvent.click(connectButton);

    const alert = await screen.findByRole('alert');
    expect(alert).toBeInTheDocument();
    expect(alert.textContent).toMatch(/connection rejected by user/i);

    expect(pushMock).not.toHaveBeenCalled();
  });

  it('redirects already-authenticated users away from /login', async () => {
    walletStoreState = {
      isAuthenticated: true,
      loading: false,
      initialize: initializeMock,
      setAccessToken: setAccessTokenMock,
    };

    renderLoginPage();

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/dashboard');
    });
  });
});