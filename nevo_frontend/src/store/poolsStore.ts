import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { apiClient } from '@/lib/api-client';

export type PoolStatus = 'Active' | 'Completed';
export type SortOption = 'newest' | 'most_raised' | 'goal_low';

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
  minTarget?: number | null;
  maxTarget?: number | null;
  dateFrom?: string | null;
  dateTo?: string | null;
}

interface PoolsState {
  pools: Pool[];
  filters: PoolFilters;
  sortBy: SortOption;
  loading: boolean;
  poolLoading: boolean;
  error: string | null;
  currentPool: Pool | null;
  setPools: (pools: Pool[]) => void;
  setLoading: (loading: boolean) => void;
  fetchPools: () => Promise<void>;
  fetchPool: (id: number) => Promise<Pool | null>;
  setSearch: (search: string) => void;
  toggleCategory: (category: string) => void;
  toggleStatus: (status: PoolStatus) => void;
  setSortBy: (sortBy: SortOption) => void;
  clearFilters: () => void;
  filteredPools: () => Pool[];
}

const DEFAULT_FILTERS: PoolFilters = {
  search: '',
  categories: [],
  statuses: [],
  minTarget: null,
  maxTarget: null,
  dateFrom: null,
  dateTo: null,
};

const DEFAULT_SORT: SortOption = 'newest';

const MOCK_POOLS: Pool[] = [
  {
    id: '1',
    title: 'Clean Water Initiative',
    description: 'Providing clean drinking water to rural communities in need.',
    category: 'Humanitarian',
    status: 'Active',
    target: 10000,
    raised: 6800,
    imageColor: '#27926e',
    creator: 'GABCDE1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890',
    createdAt: '2025-03-01',
  },
  {
    id: '2',
    title: 'Open Source Dev Fund',
    description: 'Supporting open source contributors building on Stellar.',
    category: 'Technology',
    status: 'Active',
    target: 5000,
    raised: 5000,
    imageColor: '#1c7459',
    creator: 'GB222222222222222222222222222222222222222222222222222222222',
    createdAt: '2025-01-15',
  },
  {
    id: '3',
    title: 'Community Garden Project',
    description: 'Building urban gardens to improve food security locally.',
    category: 'Environment',
    status: 'Completed',
    target: 3000,
    raised: 3200,
    imageColor: '#47ae88',
    creator: 'GC333333333333333333333333333333333333333333333333333333333',
    createdAt: '2024-11-10',
  },
  {
    id: '4',
    title: 'Local Animal Shelter Relief',
    description: 'Funding medical supplies and food for rescued animals.',
    category: 'Animal Welfare',
    status: 'Active',
    target: 8000,
    raised: 1200,
    imageColor: '#ae4747',
    creator: 'GABCDE1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890',
    createdAt: '2025-04-12',
  },
  {
    id: '5',
    title: 'Blockchain Education for Youth',
    description:
      'Providing free workshops on Web3 and blockchain development to high school students.',
    category: 'Education',
    status: 'Active',
    target: 15000,
    raised: 500,
    imageColor: '#476bae',
    creator: 'GD444444444444444444444444444444444444444444444444444444444',
    createdAt: '2025-05-20',
  },
];

export const usePoolsStore = create<PoolsState>()(
  persist(
    (set, get) => ({
      pools: MOCK_POOLS,
      filters: DEFAULT_FILTERS,
      sortBy: DEFAULT_SORT,
      loading: false,
      poolLoading: false,
      error: null,
      currentPool: null,

      setPools: (pools) => set({ pools }),
      setLoading: (loading) => set({ loading }),

      fetchPools: async () => {
        set({ loading: true, error: null });
        try {
          const data = await apiClient.get<Pool[]>('/pools');
          set({ pools: data, loading: false });
        } catch (error) {
          const err = error as Error;
          set({
            error: err?.message || 'Failed to fetch pools',
            loading: false,
          });
        }
      },

      fetchPool: async (id: number) => {
        set({ poolLoading: true, error: null, currentPool: null });
        try {
          const data = await apiClient.get<Pool>(`/pools/${id}`);
          set({ currentPool: data, poolLoading: false });
          return data;
        } catch (error) {
          const err = error as Error;
          set({
            error: err?.message || 'Failed to fetch pool',
            poolLoading: false,
          });
          return null;
        }
      },

      setSearch: (search) =>
        set((s) => ({ filters: { ...s.filters, search } })),

      setPriceRange: (min?: number | null, max?: number | null) =>
        set((s) => ({
          filters: {
            ...s.filters,
            minTarget: min ?? null,
            maxTarget: max ?? null,
          },
        })),

      setDateRange: (from?: string | null, to?: string | null) =>
        set((s) => ({
          filters: { ...s.filters, dateFrom: from ?? null, dateTo: to ?? null },
        })),

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

      setSortBy: (sortBy) => set({ sortBy }),

      clearFilters: () => set({ filters: DEFAULT_FILTERS }),

      filteredPools: () => {
        const { pools, filters, sortBy } = get();
        const filtered = pools.filter((pool) => {
          const searchLower = filters.search.toLowerCase();
          const matchSearch =
            !filters.search ||
            pool.title.toLowerCase().includes(searchLower) ||
            pool.description.toLowerCase().includes(searchLower) ||
            pool.category.toLowerCase().includes(searchLower) ||
            (pool.creator && pool.creator.toLowerCase().includes(searchLower));
          const matchCategory =
            filters.categories.length === 0 ||
            filters.categories.includes(pool.category);
          const matchStatus =
            filters.statuses.length === 0 ||
            filters.statuses.includes(pool.status);
          const matchMinTarget =
            filters.minTarget == null ||
            pool.target >= (filters.minTarget ?? 0);
          const matchMaxTarget =
            filters.maxTarget == null ||
            pool.target <= (filters.maxTarget ?? Infinity);
          const createdTs = new Date(
            pool.createdAt ?? '1970-01-01'
          ).toISOString();
          const matchDateFrom =
            !filters.dateFrom || createdTs >= filters.dateFrom + 'T00:00:00Z';
          const matchDateTo =
            !filters.dateTo || createdTs <= filters.dateTo + 'T23:59:59Z';
          return (
            matchSearch &&
            matchCategory &&
            matchStatus &&
            matchMinTarget &&
            matchMaxTarget &&
            matchDateFrom &&
            matchDateTo
          );
        });

        return [...filtered].sort((a, b) => {
          if (sortBy === 'most_raised') {
            return b.raised - a.raised;
          }
          if (sortBy === 'goal_low') {
            return a.target - b.target;
          }
          const dateA = new Date(a.createdAt ?? '1970-01-01').getTime();
          const dateB = new Date(b.createdAt ?? '1970-01-01').getTime();
          return dateB - dateA;
        });
      },
    }),
    {
      name: 'nevo-pools',
      partialize: (state) => ({ filters: state.filters, sortBy: state.sortBy }),
    }
  )
);
