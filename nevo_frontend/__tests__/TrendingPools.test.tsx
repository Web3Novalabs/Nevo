import React from 'react';
import { render, screen } from '@testing-library/react';
import { TrendingPools } from '@/components/TrendingPools';
import { usePoolsStore } from '@/src/store/poolsStore';
import type { Pool } from '@/src/store/poolsStore';

// Mock the store
jest.mock('@/src/store/poolsStore', () => ({
  usePoolsStore: jest.fn(),
}));

// Mock PoolCard to keep tests focused on TrendingPools logic
jest.mock('@/components/PoolCard', () => ({
  PoolCard: ({ pool }: { pool: Pool }) => (
    <div data-testid="pool-card" data-pool-id={pool.id}>
      {pool.title}
    </div>
  ),
}));

const mockFetchPools = jest.fn();

function mockStore(overrides: Partial<{ pools: Pool[]; loading: boolean }>) {
  (usePoolsStore as unknown as jest.Mock).mockReturnValue({
    pools: [],
    loading: false,
    fetchPools: mockFetchPools,
    ...overrides,
  });
}

const makePool = (id: string, status: Pool['status'], raised: number): Pool => ({
  id,
  title: `Pool ${id}`,
  description: `Description for pool ${id}`,
  category: 'Technology',
  status,
  target: 1000,
  raised,
  imageColor: '#aabbcc',
  creator: 'GABCD1234',
});

describe('TrendingPools', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders loading skeletons when loading is true', () => {
    mockStore({ loading: true, pools: [] });
    render(<TrendingPools />);
    // Component returns null until mounted; simulate mount via the animate-pulse skeletons
    const skeletons = document.querySelectorAll('.animate-pulse');
    expect(skeletons.length).toBe(3);
  });

  it('renders the empty-state message when there are no active pools', () => {
    mockStore({
      loading: false,
      pools: [makePool('1', 'Completed', 500)],
    });
    render(<TrendingPools />);
    expect(
      screen.getByText('No trending pools at the moment.')
    ).toBeInTheDocument();
  });

  it('renders only Active pools', () => {
    mockStore({
      loading: false,
      pools: [
        makePool('1', 'Active', 200),
        makePool('2', 'Completed', 800),
        makePool('3', 'Active', 100),
      ],
    });
    render(<TrendingPools />);
    const cards = screen.getAllByTestId('pool-card');
    expect(cards).toHaveLength(2);
    const ids = cards.map((c) => c.getAttribute('data-pool-id'));
    expect(ids).toContain('1');
    expect(ids).toContain('3');
    expect(ids).not.toContain('2');
  });

  it('sorts active pools by raised descending', () => {
    mockStore({
      loading: false,
      pools: [
        makePool('low', 'Active', 100),
        makePool('high', 'Active', 900),
        makePool('mid', 'Active', 500),
      ],
    });
    render(<TrendingPools />);
    const cards = screen.getAllByTestId('pool-card');
    expect(cards[0]).toHaveAttribute('data-pool-id', 'high');
    expect(cards[1]).toHaveAttribute('data-pool-id', 'mid');
    expect(cards[2]).toHaveAttribute('data-pool-id', 'low');
  });

  it('caps displayed pools at 4', () => {
    mockStore({
      loading: false,
      pools: [
        makePool('1', 'Active', 100),
        makePool('2', 'Active', 200),
        makePool('3', 'Active', 300),
        makePool('4', 'Active', 400),
        makePool('5', 'Active', 500),
      ],
    });
    render(<TrendingPools />);
    const cards = screen.getAllByTestId('pool-card');
    expect(cards).toHaveLength(4);
  });

  it('shows top 4 by raised when more than 4 active pools exist', () => {
    mockStore({
      loading: false,
      pools: [
        makePool('a', 'Active', 10),
        makePool('b', 'Active', 50),
        makePool('c', 'Active', 80),
        makePool('d', 'Active', 40),
        makePool('e', 'Active', 90),
      ],
    });
    render(<TrendingPools />);
    const cards = screen.getAllByTestId('pool-card');
    const ids = cards.map((c) => c.getAttribute('data-pool-id'));
    expect(ids).toEqual(['e', 'c', 'b', 'd']);
  });
});
