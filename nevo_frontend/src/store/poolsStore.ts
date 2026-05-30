import { create } from 'zustand';
import { persist } from 'zustand/middleware';

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
}

interface PoolsState {
  pools: Pool[];
  filters: PoolFilters;
  sortBy: SortOption;
  loading: boolean;
  setPools: (pools: Pool[]) => void;
  setLoading: (loading: boolean) => void;
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

      setPools: (pools) => set({ pools }),
      setLoading: (loading) => set({ loading }),

      setSearch: (search) =>
        set((s) => ({ filters: { ...s.filters, search } })),

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
          return matchSearch && matchCategory && matchStatus;
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
