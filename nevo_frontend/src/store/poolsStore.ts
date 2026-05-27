import { create } from 'zustand';

export type PoolStatus = 'Active' | 'Completed';

export interface Pool {
  id: string;
  title: string;
  description: string;
  category: string;
  status: PoolStatus;
  target: number;
  raised: number;
  imageColor: string;
  creator?: string;
  createdAt?: string;
}

interface PoolFilters {
  search: string;
  categories: string[];
  statuses: PoolStatus[];
}

interface PoolsState {
  pools: Pool[];
  filters: PoolFilters;
  loading: boolean;
  setPools: (pools: Pool[]) => void;
  setLoading: (loading: boolean) => void;
  setSearch: (search: string) => void;
  toggleCategory: (category: string) => void;
  toggleStatus: (status: PoolStatus) => void;
  clearFilters: () => void;
  filteredPools: () => Pool[];
}

const DEFAULT_FILTERS: PoolFilters = {
  search: '',
  categories: [],
  statuses: [],
};

export const usePoolsStore = create<PoolsState>()((set, get) => ({
  pools: [],
  filters: DEFAULT_FILTERS,
  loading: false,

  setPools: (pools) => set({ pools }),
  setLoading: (loading) => set({ loading }),

  setSearch: (search) => set((s) => ({ filters: { ...s.filters, search } })),

  toggleCategory: (category) =>
    set((s) => ({
      filters: {
        ...s.filters,
        categories: s.filters.categories.includes(category)
          ? s.filters.categories.filter((c) => c !== category)
          : [...s.filters.categories, category],
      },
    })),

  toggleStatus: (status) =>
    set((s) => ({
      filters: {
        ...s.filters,
        statuses: s.filters.statuses.includes(status)
          ? s.filters.statuses.filter((st) => st !== status)
          : [...s.filters.statuses, status],
      },
    })),

  clearFilters: () => set({ filters: DEFAULT_FILTERS }),

  filteredPools: () => {
    const { pools, filters } = get();
    return pools.filter((pool) => {
      const matchSearch =
        !filters.search ||
        pool.title.toLowerCase().includes(filters.search.toLowerCase()) ||
        pool.description.toLowerCase().includes(filters.search.toLowerCase());
      const matchCategory =
        filters.categories.length === 0 ||
        filters.categories.includes(pool.category);
      const matchStatus =
        filters.statuses.length === 0 || filters.statuses.includes(pool.status);
      return matchSearch && matchCategory && matchStatus;
    });
  },
}));
