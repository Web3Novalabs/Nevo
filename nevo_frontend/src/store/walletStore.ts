import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { getPublicKey, connect, disconnect } from '@/app/stellar-wallets-kit';
import { getAccountBalances, AccountBalances } from '@/lib/stellar';

interface WalletState {
  publicKey: string | null;
  balances: AccountBalances | null;
  loading: boolean;
  connectWallet: (onSuccess?: () => void) => Promise<void>;
  disconnectWallet: () => Promise<void>;
  refreshBalances: () => Promise<void>;
  initialize: () => Promise<void>;
}

export const useWalletStore = create<WalletState>()(
  persist(
    (set, get) => ({
      publicKey: null,
      balances: null,
      loading: true,

      initialize: async () => {
        const key = await getPublicKey();
        if (key) {
          const balances = await getAccountBalances(key);
          set({ publicKey: key, balances, loading: false });
        } else {
          set({ loading: false });
        }
      },

      connectWallet: async (onSuccess) => {
        await connect(async () => {
          const key = await getPublicKey();
          if (key) {
            const balances = await getAccountBalances(key);
            set({ publicKey: key, balances });
            onSuccess?.();
          }
        });
      },

      disconnectWallet: async () => {
        await disconnect();
        set({ publicKey: null, balances: null });
      },

      refreshBalances: async () => {
        const { publicKey } = get();
        if (!publicKey) return;
        const balances = await getAccountBalances(publicKey);
        set({ balances });
      },
    }),
    {
      name: 'nevo-wallet',
      partialize: (state) => ({ publicKey: state.publicKey }),
    }
  )
);
