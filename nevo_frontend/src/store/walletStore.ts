import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import {
  getPublicKey,
  connect,
  disconnect,
  signWithWallet,
} from '@/app/stellar-wallets-kit';
import { clearToken, getToken, setToken } from '@/lib/auth-storage';
import { getAccountBalances, AccountBalances } from '@/lib/stellar';
import { fetchAuthChallenge, verifyAuthSignature } from '@/lib/api-client';

interface WalletState {
  publicKey: string | null;
  accessToken: string | null;
  balances: AccountBalances | null;
  loading: boolean;
  isAuthenticated: boolean;
  connectWallet: (onSuccess?: () => void) => Promise<void>;
  disconnectWallet: () => Promise<void>;
  refreshBalances: () => Promise<void>;
  initialize: () => Promise<void>;
  setAccessToken: (token: string) => void;
}

export const useWalletStore = create<WalletState>()(
  persist(
    (set, get) => ({
      publicKey: null,
      accessToken: null,
      balances: null,
      loading: true,
      isAuthenticated: false,

      initialize: async () => {
        const key = await getPublicKey();
        if (key) {
          const balances = await getAccountBalances(key);
          const accessToken = get().accessToken ?? getToken();
          set({
            publicKey: key,
            balances,
            accessToken,
            loading: false,
            isAuthenticated: !!accessToken,
          });
        } else {
          set({ loading: false, isAuthenticated: false });
        }
      },

      connectWallet: async (onSuccess) => {
        await connect(async () => {
          const key = await getPublicKey();
          if (key) {
            const balances = await getAccountBalances(key);
            const { nonce } = await fetchAuthChallenge(key);
            const signature = await signWithWallet(nonce);
            const { accessToken } = await verifyAuthSignature(
              key,
              nonce,
              signature
            );
            setToken(accessToken);
            set({
              publicKey: key,
              balances,
              accessToken,
              isAuthenticated: true,
            });
            onSuccess?.();
          }
        });
      },

      disconnectWallet: async () => {
        await disconnect();
        set({
          publicKey: null,
          accessToken: null,
          balances: null,
          isAuthenticated: false,
        });
        clearToken();
      },

      refreshBalances: async () => {
        const { publicKey } = get();
        if (!publicKey) return;
        const balances = await getAccountBalances(publicKey);
        set({ balances });
      },

      setAccessToken: (token: string) => {
        setToken(token);
        set({
          accessToken: token,
          isAuthenticated: !!get().publicKey && !!token,
        });
      },
    }),
    {
      name: 'nevo-wallet',
      partialize: (state) => ({
        publicKey: state.publicKey,
        accessToken: state.accessToken,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
);
