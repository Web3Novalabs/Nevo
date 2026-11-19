import { clearToken, getToken, setToken } from './auth-storage';

export { getToken, setToken, clearToken };
import { clearToken, getToken, getTokenExpiry, isTokenExpired } from './auth-storage';

export function getStoredAccessToken(): string | null {
  return getToken();
}

export function getStoredAccessTokenExpiry(): number | null {
  return getTokenExpiry();
}

export function isStoredTokenExpired(): boolean {
  return isTokenExpired();
}

export function clearJwt(): void {
  clearToken();
}

/**
 * Single place to clear all auth data: the stored token plus the persisted
 * wallet store (accessToken / isAuthenticated). The store import is lazy to
 * avoid a module cycle between the store and this module.
 */
export function clearAuth(): void {
  clearToken();

  try {
    // Lazy require to avoid a circular import at module load time.
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const { useWalletStore } = require('./stores/wallet-store') as {
      useWalletStore?: {
        getState: () => {
          logout?: () => void;
          setState?: (partial: Record<string, unknown>) => void;
        };
      };
    };

    const store = useWalletStore;
    if (!store) return;

    const state = store.getState();
    if (typeof state.logout === 'function') {
      state.logout();
    } else if (typeof store.setState === 'function') {
      store.setState({ accessToken: null, isAuthenticated: false });
    }
  } catch {
    // Store module not available (e.g. in tests); token is already cleared.
  }
}