import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

// Mock the pools store
const mockPools = [
  {
    id: 'pool-1',
    title: 'Alpha Pool',
    category: 'Savings',
    target: 10000,
    raised: 2500,
    imageColor: '#FF0000',
    status: 'active',
  },
  {
    id: 'pool-2',
    title: 'Beta Pool',
    category: 'Investment',
    target: 20000,
    raised: 5000,
    imageColor: '#00FF00',
    status: 'active',
  },
  {
    id: 'pool-3',
    title: 'Gamma Pool',
    category: 'Savings',
    target: 15000,
    raised: 7500,
    imageColor: '#0000FF',
    status: 'active',
  },
  {
    id: 'pool-4',
    title: 'Delta Pool',
    category: 'Investment',
    target: 30000,
    raised: 10000,
    imageColor: '#FFFF00',
    status: 'active',
  },
];

jest.mock('@/src/store/poolsStore', () => ({
  usePoolsStore: jest.fn(() => ({
    pools: mockPools,
    loading: false,
    error: null,
    fetchPools: jest.fn(),
  })),
}));

// Mock next/navigation if used
jest.mock('next/navigation', () => ({
  useRouter: () => ({
    push: jest.fn(),
    replace: jest.fn(),
    back: jest.fn(),
  }),
  usePathname: () => '/pools/compare',
}));

// Import the page after mocks
import ComparePoolsPage from '@/app/pools/compare/page';

describe('Pools Compare Page (#1553)', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('selects pools and shows selection count, then switches to comparison view', async () => {
    const user = userEvent.setup();
    render(<ComparePoolsPage />);

    // Page renders pool cards
    expect(screen.getByText('Alpha Pool')).toBeInTheDocument();
    expect(screen.getByText('Beta Pool')).toBeInTheDocument();

    // Initially no selection
    expect(screen.getByText(/0\/4 selected/i)).toBeInTheDocument();

    // Select a pool
    await user.click(screen.getByText('Alpha Pool'));

    await waitFor(() => {
      expect(screen.getByText(/1\/4 selected/i)).toBeInTheDocument();
    });

    // Compare button should reflect count
    const compareButton = screen.getByRole('button', { name: /compare 1 pool/i });
    expect(compareButton).toBeInTheDocument();

    // Click compare
    await user.click(compareButton);

    await waitFor(() => {
      // Comparison view shows selected pool title
      expect(screen.getByText('Alpha Pool')).toBeInTheDocument();
    });
  });

  it('renders empty/selection state with zero pools selected without error', () => {
    render(<ComparePoolsPage />);

    expect(screen.getByText(/0\/4 selected/i)).toBeInTheDocument();
    // Compare button disabled or shows 0
    const compareButton = screen.queryByRole('button', { name: /compare 0 pools/i });
    if (compareButton) {
      expect(compareButton).toBeDisabled();
    }
  });

  it('renders partial comparison view with exactly one pool selected', async () => {
    const user = userEvent.setup();
    render(<ComparePoolsPage />);

    await user.click(screen.getByText('Beta Pool'));

    await waitFor(() => {
      expect(screen.getByText(/1\/4 selected/i)).toBeInTheDocument();
    });

    const compareButton = screen.getByRole('button', { name: /compare 1 pool/i });
    await user.click(compareButton);

    await waitFor(() => {
      expect(screen.getByText('Beta Pool')).toBeInTheDocument();
    });
  });

  it('removes a pool from comparison and updates the view', async () => {
    const user = userEvent.setup();
    render(<ComparePoolsPage />);

    // Select two pools
    await user.click(screen.getByText('Alpha Pool'));
    await user.click(screen.getByText('Beta Pool'));

    await waitFor(() => {
      expect(screen.getByText(/2\/4 selected/i)).toBeInTheDocument();
    });

    const compareButton = screen.getByRole('button', { name: /compare 2 pools/i });
    await user.click(compareButton);

    await waitFor(() => {
      expect(screen.getByText('Alpha Pool')).toBeInTheDocument();
      expect(screen.getByText('Beta Pool')).toBeInTheDocument();
    });

    // Find and click remove action for Alpha Pool
    const removeButtons = screen.getAllByRole('button', { name: /remove/i });
    expect(removeButtons.length).toBeGreaterThan(0);

    await user.click(removeButtons[0]);

    await waitFor(() => {
      expect(screen.queryByText('Alpha Pool')).not.toBeInTheDocument();
      expect(screen.getByText('Beta Pool')).toBeInTheDocument();
    });
  });
});