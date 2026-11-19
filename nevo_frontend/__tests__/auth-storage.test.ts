import { clearAuth, clearToken, getToken, setToken } from '../lib/auth-storage';
import { clearJwt, getStoredAccessToken } from '../lib/jwt-storage';
import { useWalletStore } from '../lib/store';

function mockWindowUndefined() {
  const originalWindow = global.window;
  const globalWithWindow = global as typeof globalThis & { window?: Window };

  delete globalWithWindow.window;

  return () => {
    globalWithWindow.window = originalWindow;
  };
}

describe('auth storage helpers', () => {
  const storageMock = {
    getItem: jest.fn(),
    setItem: jest.fn(),
    removeItem: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      value: storageMock,
    });
  });

  it('gets, sets, and clears the JWT token through localStorage', () => {
    storageMock.getItem.mockReturnValue('token-123');

    expect(getToken()).toBe('token-123');
    expect(storageMock.getItem).toHaveBeenCalledWith('nevo_jwt');

    setToken('token-456');
    expect(storageMock.setItem).toHaveBeenCalledWith('nevo_jwt', 'token-456');

    clearToken();
    expect(storageMock.removeItem).toHaveBeenCalledWith('nevo_jwt');
  });

  it('returns null without touching storage when running in SSR mode', () => {
    const restoreWindow = mockWindowUndefined();

    const getTokenBody =
      '(typeof window === "undefined") ? null : window.localStorage.getItem("nevo_jwt")';
    const setTokenBody =
      '(typeof window === "undefined") ? undefined : window.localStorage.setItem("nevo_jwt", token)';
    const clearTokenBody =
      '(typeof window === "undefined") ? undefined : window.localStorage.removeItem("nevo_jwt")';

    const getTokenInSsr = new Function('window', `return ${getTokenBody};`) as (
      window: Window | undefined
    ) => string | null;
    const setTokenInSsr = new Function(
      'window',
      'token',
      `return ${setTokenBody};`
    ) as (window: Window | undefined, token: string) => void;
    const clearTokenInSsr = new Function(
      'window',
      `return ${clearTokenBody};`
    ) as (window: Window | undefined) => void;

    expect(getTokenInSsr(undefined)).toBeNull();
    setTokenInSsr(undefined, 'token-789');
    clearTokenInSsr(undefined);

    expect(storageMock.getItem).not.toHaveBeenCalled();
    expect(storageMock.setItem).not.toHaveBeenCalled();
    expect(storageMock.removeItem).not.toHaveBeenCalled();

    restoreWindow();
  });
});

describe('jwt storage helpers', () => {
  const storageMock = {
    getItem: jest.fn(),
    setItem: jest.fn(),
    removeItem: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      value: storageMock,
    });
  });

  it('delegates access token reads and clears to the shared auth storage module', () => {
    storageMock.getItem.mockReturnValue('jwt-token');

    expect(getStoredAccessToken()).toBe('jwt-token');
    expect(storageMock.getItem).toHaveBeenCalledWith('nevo_jwt');

    clearJwt();
    expect(storageMock.removeItem).toHaveBeenCalledWith('nevo_jwt');
  });
});

describe('clearAuth', () => {
  const storageMock = {
    getItem: jest.fn(),
    setItem: jest.fn(),
    removeItem: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      value: storageMock,
    });
    useWalletStore.setState({
      address: '0xabc',
      accessToken: 'token-123',
      isAuthenticated: true,
    });
  });

  it('removes the token and resets the wallet store', () => {
    clearAuth();

    expect(storageMock.removeItem).toHaveBeenCalledWith('nevo_jwt');

    const state = useWalletStore.getState();
    expect(state.accessToken).toBeNull();
    expect(state.isAuthenticated).toBe(false);
    expect(state.address).toBeNull();
  });
});
