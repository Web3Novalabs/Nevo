import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import DashboardPage from '@/app/dashboard/page';
import { fetchCreatorPools } from '@/lib/api-client';

jest.mock('next/link', () => ({
  __esModule: true,
  default: ({
    href,
    children,
  }: {
    href: string;
    children: React.ReactNode;
  }) => <a href={href}>{children}</a>,
}));

jest.mock('@/components/ProtectedRoute', () => ({
  __esModule: true,
  default: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

jest.mock('@/components/EmptyState', () => ({
  EmptyState: ({ title }: { title: string }) => <div>{title}</div>,
}));

jest.mock('@/components/ConnectWallet', () => ({
  __esModule: true,
  default: () => null,
}));

jest.mock('@/components/WalletAddress', () => ({
  WalletAddress: () => null,
}));

jest.mock('@/components/Toast', () => ({
  toast: jest.fn(),
}));

jest.mock('@/src/store/walletStore', () => ({
  useWalletStore: () => ({ publicKey: 'GCREATORPUBLICKEY', initialize: jest.fn() }),
}));

jest.mock('@/lib/api-client', () => ({
  apiClient: { get: jest.fn() },
  fetchCreatorPools: jest.fn(),
  submitSignedXdr: jest.fn(),
  withdrawPool: jest.fn(),
  closePool: jest.fn(),
}));

jest.mock('@stellar/freighter-api', () => ({
  signTransaction: jest.fn(),
}));

const mockFetchCreatorPools = fetchCreatorPools as jest.MockedFunction<
  typeof fetchCreatorPools
>;

const samplePool = {
  id: 'pool-1',
  creator: 'GCREATORPUBLICKEY',
  name: 'Real Creator Pool',
  status: 'active',
  targetAmount: '1000',
  currentAmount: '250',
  deadline: '2030-01-01T00:00:00.000Z',
};

describe('DashboardPage', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders the pool summary heading for the dashboard', async () => {
    mockFetchCreatorPools.mockResolvedValue([samplePool] as never);

    render(<DashboardPage />);

    expect(screen.getByRole('heading', { name: 'My Pools' })).toBeInTheDocument();

    await waitFor(() => {
      expect(mockFetchCreatorPools).toHaveBeenCalledWith('GCREATORPUBLICKEY');
    });
  });

  it('shows a loading state while pools are being fetched', () => {
    mockFetchCreatorPools.mockReturnValue(new Promise(() => {}) as never);

    render(<DashboardPage />);

    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it('renders the creator pools returned by the API', async () => {
    mockFetchCreatorPools.mockResolvedValue([samplePool] as never);

    render(<DashboardPage />);

    expect(await screen.findByText('Real Creator Pool')).toBeInTheDocument();
    expect(mockFetchCreatorPools).toHaveBeenCalledWith('GCREATORPUBLICKEY');
  });

  it('shows an empty state when the creator has no pools', async () => {
    mockFetchCreatorPools.mockResolvedValue([] as never);

    render(<DashboardPage />);

    expect(await screen.findByText(/no pools/i)).toBeInTheDocument();
  });

  it('shows an error state with a retry button when the fetch fails', async () => {
    mockFetchCreatorPools.mockRejectedValueOnce(new Error('network down'));

    render(<DashboardPage />);

    expect(await screen.findByText(/failed to load/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
  });

  it('retries fetching pools when the retry button is clicked', async () => {
    mockFetchCreatorPools
      .mockRejectedValueOnce(new Error('network down'))
      .mockResolvedValueOnce([samplePool] as never);

    render(<DashboardPage />);

    const retryButton = await screen.findByRole('button', { name: /retry/i });
    fireEvent.click(retryButton);

    expect(await screen.findByText('Real Creator Pool')).toBeInTheDocument();
    expect(mockFetchCreatorPools).toHaveBeenCalledTimes(2);
  });

  it('does not render any mock pool data', async () => {
    mockFetchCreatorPools.mockResolvedValue([] as never);

    render(<DashboardPage />);

    await screen.findByText(/no pools/i);

    expect(screen.queryByText(/mock/i)).not.toBeInTheDocument();
  });
});
