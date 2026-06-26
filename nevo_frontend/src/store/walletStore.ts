import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { getPublicKey, connect, disconnect } from '@/app/stellar-wallets-kit';
import { getAccountBalances, AccountBalances } from '@/lib/stellar';

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
          const accessToken = get().accessToken;
          set({
            publicKey: key,
            balances,
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
            const accessToken = get().accessToken;
            set({
              publicKey: key,
              balances,
              isAuthenticated: !!accessToken,
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
      },

      refreshBalances: async () => {
        const { publicKey } = get();
        if (!publicKey) return;
        const balances = await getAccountBalances(publicKey);
        set({ balances });
      },

      setAccessToken: (token: string) => {
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
