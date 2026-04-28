import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface Donation {
  id: string;
  poolId: string;
  poolName: string;
  amount: string;
  asset: 'XLM' | 'USDC';
  txHash: string;
  timestamp: string;
  status: 'pending' | 'confirmed' | 'failed';
}

interface DonationsState {
  history: Donation[];
  activeDonation: Donation | null;
  addDonation: (donation: Donation) => void;
  setActiveDonation: (donation: Donation | null) => void;
  updateDonationStatus: (id: string, status: Donation['status']) => void;
  clearHistory: () => void;
}

export const useDonationsStore = create<DonationsState>()(
  persist(
    (set) => ({
      history: [],
      activeDonation: null,

      addDonation: (donation) =>
        set((s) => ({ history: [donation, ...s.history] })),

      setActiveDonation: (donation) => set({ activeDonation: donation }),

      updateDonationStatus: (id, status) =>
        set((s) => ({
          history: s.history.map((d) => (d.id === id ? { ...d, status } : d)),
        })),

      clearHistory: () => set({ history: [] }),
    }),
    {
      name: 'nevo-donations',
      partialize: (state) => ({ history: state.history }),
    }
  )
);
