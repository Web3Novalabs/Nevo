import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import BrowsePoolsPage from '@/app/pools/page';
import { usePoolsStore } from '@/src/store/poolsStore';

jest.mock('next/link', () => ({
  __esModule: true,
  default: ({ href, children }: { href: string; children: React.ReactNode }) => (
    <a href={href}>{children}</a>
  ),
}));

jest.mock('@/components/EmptyState', () => ({
  EmptyState: ({ title }: { title: string }) => <div>{title}</div>,
}));

jest.mock('@/components/ErrorBoundary', () => ({
  ErrorBoundary: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

jest.mock('@/components', () => ({
  BackToTopButton: () => null,
  Pagination: () => null,
  PoolCard: () => null,
  Skeleton: () => null,
}));

jest.mock('@/lib/getDonorCount', () => ({
  getDonorCount: () => 0,
}));

jest.mock('@/src/store/poolsStore', () => ({
  usePoolsStore: jest.fn(),
}));

describe('BrowsePoolsPage', () => {
  it('requests pools and renders the browse page shell', async () => {
    const fetchPools = jest.fn();
    (usePoolsStore as jest.Mock).mockReturnValue({
      pools: [],
      loading: false,
      error: null,
      fetchPools,
    });

    render(<BrowsePoolsPage />);

    expect(screen.getByRole('heading', { name: 'Browse Donation Pools' })).toBeInTheDocument();
    await waitFor(() => expect(fetchPools).toHaveBeenCalledTimes(1));
  });
});