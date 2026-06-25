import React from 'react';
import { render, screen, waitFor, act } from '@testing-library/react';
import DashboardPage from '@/app/dashboard/page';
import { usePoolsStore } from '@/src/store/poolsStore';

// Mock ProtectedRoute to render children directly
jest.mock('@/components/ProtectedRoute', () => {
  return function MockProtectedRoute({
    children,
  }: {
    children: React.ReactNode;
  }) {
    return <>{children}</>;
  };
});

// Mock next/navigation
jest.mock('next/navigation', () => ({
  useRouter() {
    return {
      push: jest.fn(),
    };
  },
  usePathname() {
    return '/dashboard';
  },
}));

// Mock useWalletStore
const mockWalletStore = {
  publicKey: 'GABCDE1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890',
  loading: false,
  initialize: jest.fn().mockResolvedValue(undefined),
};

jest.mock('@/src/store/walletStore', () => ({
  useWalletStore: () => mockWalletStore,
}));

describe('DashboardPage', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it('renders empty state when the user has no pools', async () => {
    // Set up pools store to have 0 pools matching the user's publicKey
    usePoolsStore.setState({
      pools: [
        {
          id: '999',
          title: "Someone else's pool",
          description: 'Not mine',
          category: 'Education',
          status: 'Active',
          target: 1000,
          raised: 0,
          imageColor: '#123456',
          creator: 'DIFFERENT_CREATOR_KEY',
        },
      ],
    });

    render(<DashboardPage />);

    // Fast-forward timers for loading state timeout (400ms)
    act(() => {
      jest.advanceTimersByTime(400);
    });

    await waitFor(() => {
      expect(
        screen.getByText("You haven't created any pools yet")
      ).toBeInTheDocument();
    });

    const createPoolBtn = screen.getByRole('link', { name: 'Create Pool' });
    expect(createPoolBtn).toBeInTheDocument();
    expect(createPoolBtn).toHaveAttribute('href', '/pools/new');
  });

  it('renders pool list when the user has pools', async () => {
    // Set up pools store to have 1 pool matching the user's publicKey
    usePoolsStore.setState({
      pools: [
        {
          id: '123',
          title: 'My Custom Pool',
          description: 'This is my fundraising pool',
          category: 'Technology',
          status: 'Active',
          target: 5000,
          raised: 1200,
          imageColor: '#27926e',
          creator:
            'GABCDE1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890',
          createdAt: '2025-03-01',
        },
      ],
    });

    render(<DashboardPage />);

    act(() => {
      jest.advanceTimersByTime(400);
    });

    await waitFor(() => {
      expect(screen.getByText('My Custom Pool')).toBeInTheDocument();
      expect(
        screen.queryByText("You haven't created any pools yet")
      ).not.toBeInTheDocument();
    });
  });
});
