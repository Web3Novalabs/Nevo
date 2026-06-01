import { usePoolsStore, Pool } from '@/src/store/poolsStore';

const pools: Pool[] = [
  {
    id: '1',
    title: 'Alpha Fund',
    description: 'desc a',
    category: 'DeFi',
    status: 'Active',
    target: 1000,
    raised: 500,
    imageColor: '#fff',
  },
  {
    id: '2',
    title: 'Beta Pool',
    description: 'desc b',
    category: 'NFT',
    status: 'Completed',
    target: 2000,
    raised: 2000,
    imageColor: '#000',
  },
];

beforeEach(() =>
  usePoolsStore.setState({
    pools: [],
    filters: { search: '', categories: [], statuses: [] },
    loading: false,
  })
);

describe('poolsStore', () => {
  it('sets pools', () => {
    usePoolsStore.getState().setPools(pools);
    expect(usePoolsStore.getState().pools).toHaveLength(2);
  });

  it('filters by search', () => {
    usePoolsStore.getState().setPools(pools);
    usePoolsStore.getState().setSearch('alpha');
    expect(usePoolsStore.getState().filteredPools()).toHaveLength(1);
  });

  it('filters by category', () => {
    usePoolsStore.getState().setPools(pools);
    usePoolsStore.getState().toggleCategory('NFT');
    expect(usePoolsStore.getState().filteredPools()).toHaveLength(1);
    expect(usePoolsStore.getState().filteredPools()[0].id).toBe('2');
  });

  it('filters by status', () => {
    usePoolsStore.getState().setPools(pools);
    usePoolsStore.getState().toggleStatus('Active');
    expect(usePoolsStore.getState().filteredPools()).toHaveLength(1);
  });

  it('clears filters', () => {
    usePoolsStore.getState().setPools(pools);
    usePoolsStore.getState().setSearch('alpha');
    usePoolsStore.getState().clearFilters();
    expect(usePoolsStore.getState().filteredPools()).toHaveLength(2);
  });

  it('toggles category off when already selected', () => {
    usePoolsStore.getState().toggleCategory('DeFi');
    usePoolsStore.getState().toggleCategory('DeFi');
    expect(usePoolsStore.getState().filters.categories).toHaveLength(0);
  });
});
