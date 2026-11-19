import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import {
  getPublicKey,
  connect,
  disconnect,
  signWithWallet,
} from '@/app/stellar-wallets-kit';
import { clearAuth, getStoredAccessToken, setStoredAccessToken } from '@/lib/jwt-storage';
import { getAccountBalances, AccountBalances } from '@/lib/stellar';
import { fetchAuthChallenge, verifyAuthSignature } from '@/lib/api-client';

/**
 * Decode the `exp` claim (seconds since epoch) from a JWT payload.
 * Returns `null` when the token is malformed or has no numeric `exp`.
 */
export function getTokenExpiry(token: string | null | undefined): number | null {
  if (!token) return null;
  const parts = token.split('.');
  if (parts.length < 2) return null;
  try {
    const base64 = parts[1].replace(/-/g, '+').replace(/_/g, '/');
    const padded = base64.padEnd(
      base64.length + ((4 - (base64.length % 4)) % 4),
      '='
    );
    const payload = JSON.parse(atob(padded)) as { exp?: unknown };
    return typeof payload.exp === 'number' ? payload.exp : null;
  } catch {
    return null;
  }
}

/** A token is valid only if it has an `exp` in the future. */
export function isTokenValid(token: string | null | undefined): boolean {
  const exp = getTokenExpiry(token);
  if (exp === null) return false;
  return exp * 1000 > Date.now();
}

let logoutTimer: ReturnType<typeof setTimeout> | null = null;

function clearLogoutTimer() {
  if (logoutTimer !== null) {
    clearTimeout(logoutTimer);
    logoutTimer = null;
  }
}

interface WalletState {
  publicKey: string | null;
  accessToken: string | null;
  balances: AccountBalances | null;
  selectedWalletId: string | null;
  loading: boolean;
  isAuthenticated: boolean;
  /** True when the session ended because the token expired (prompt re-auth). */
  reauthRequired: boolean;
  connectWallet: (onSuccess?: () => void) => Promise<void>;
  disconnectWallet: () => Promise<void>;
  refreshBalances: () => Promise<void>;
  initialize: () => Promise<void>;
  setAccessToken: (token: string) => void;
  /** Clear auth state after expiry and surface the re-auth prompt. */
  handleTokenExpiry: () => void;
  /** Dismiss the re-auth prompt without re-authenticating. */
  dismissReauth: () => void;
}
=======

export const useWalletStore = create<WalletState>()(
  persist(
    (set, get) => ({
      publicKey: null,
      accessToken: null,
      balances: null,
      selectedWalletId: null,
      loading: true,
      isAuthenticated: false,
      reauthRequired: false,

      initialize: async () => {
        const { selectedWalletId } = get();
        const key = await getPublicKey(selectedWalletId ?? undefined);
        if (key) {
          const balances = await getAccountBalances(key);
          const accessToken = getStoredAccessToken();
          const accessToken = get().accessToken ?? getToken();
          const valid = isTokenValid(accessToken);
          if (valid && accessToken) {
            get().setAccessToken(accessToken);
          } else {
            clearLogoutTimer();
            if (accessToken) clearToken();
          }
          set({
            publicKey: key,
            balances,
            accessToken: valid ? accessToken : null,
            loading: false,
            isAuthenticated: valid,
            reauthRequired: !!accessToken && !valid,
          });
        } else {
          clearLogoutTimer();
          set({ loading: false, isAuthenticated: false, reauthRequired: false });
        }
      },

      connectWallet: async (onSuccess) => {
        await connect(async (walletId) => {
          const key = await getPublicKey(walletId);
          if (key) {
            const balances = await getAccountBalances(key);
            const { nonce } = await fetchAuthChallenge(key);
            const signature = await signWithWallet(nonce, walletId);
            const { accessToken } = await verifyAuthSignature(
              key,
              nonce,
              signature
            );
            setStoredAccessToken(accessToken);
            set({
              publicKey: key,
              balances,
              accessToken,
              isAuthenticated: !!getStoredAccessToken(),
              selectedWalletId: walletId ?? get().selectedWalletId,
              isAuthenticated: true,
            });
            set({ publicKey: key, balances });
            get().setAccessToken(accessToken);
            onSuccess?.();
          }
        });
      },

      disconnectWallet: async () => {
        await disconnect();
        clearAuth();
        clearLogoutTimer();
        set({
          publicKey: null,
          accessToken: null,
          balances: null,
          selectedWalletId: null,
          isAuthenticated: false,
          reauthRequired: false,
        });
      },

      refreshBalances: async () => {
        const { publicKey } = get();
        if (!publicKey) return;
        const balances = await getAccountBalances(publicKey);
        set({ balances });
      },

      setAccessToken: (token: string) => {
        setStoredAccessToken(token);
        set({
          accessToken: token,
          isAuthenticated: !!get().publicKey && !!getStoredAccessToken(),
        clearLogoutTimer();
        const exp = getTokenExpiry(token);
        const valid = isTokenValid(token);
        if (!valid) {
          clearToken();
          set({
            accessToken: null,
            isAuthenticated: false,
            reauthRequired: !!get().publicKey,
          });
          return;
        }
        setToken(token);
        set({
          accessToken: token,
          isAuthenticated: !!get().publicKey,
          reauthRequired: false,
        });
        if (exp !== null) {
          const delay = Math.max(0, exp * 1000 - Date.now());
          logoutTimer = setTimeout(() => {
            logoutTimer = null;
            get().handleTokenExpiry();
          }, delay);
        }
      },

      handleTokenExpiry: () => {
        clearLogoutTimer();
        clearToken();
        set({
          accessToken: null,
          isAuthenticated: false,
          reauthRequired: !!get().publicKey,
        });
      },

      dismissReauth: () => {
        set({ reauthRequired: false });
      },
    }),
    {
      name: 'nevo-wallet',
      partialize: (state) => ({
        publicKey: state.publicKey,
        balances: state.balances,
        accessToken: state.accessToken,
        selectedWalletId: state.selectedWalletId,
        isAuthenticated: state.isAuthenticated,
      }),
      onRehydrateStorage: () => (state) => {
        // Never trust a persisted `isAuthenticated`; re-derive from the token.
        if (!state) return;
        const valid = isTokenValid(state.accessToken);
        if (valid && state.accessToken) {
          state.setAccessToken(state.accessToken);
        } else {
          clearLogoutTimer();
          if (state.accessToken) clearToken();
          state.accessToken = null;
          state.isAuthenticated = false;
          state.reauthRequired = !!state.publicKey && !!state.accessToken;
        }
      },
    }
  )
);
