import React from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import PoolDetailPage from '@/app/pools/[id]/page';
import { usePoolsStore } from '@/src/store/poolsStore';
import {
  createPoolComment,
  deletePoolComment,
  fetchPoolComments,
  updatePoolComment,
} from '@/lib/api-client';

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

jest.mock('next/navigation', () => ({
  useParams: () => ({ id: '404' }),
  useRouter: () => ({ replace: jest.fn() }),
}));

jest.mock('@/components/DonateModal', () => ({
  DonateModal: () => null,
}));

jest.mock('@/components/EmptyState', () => ({
  EmptyState: ({ title }: { title: string }) => <div>{title}</div>,
}));

jest.mock('@/components/ErrorBoundary', () => ({
  ErrorBoundary: ({ children }: { children: React.ReactNode }) => (
    <>{children}</>
  ),
}));

jest.mock('@/components/WalletAddress', () => ({
  WalletAddress: () => null,
}));

jest.mock('@/components/CopyButton', () => ({
  CopyButton: () => null,
}));

jest.mock('@/components/Toast', () => ({
  toast: jest.fn(),
}));

jest.mock('@/src/store/poolsStore', () => ({
  usePoolsStore: jest.fn(),
}));

const mockWalletState: { publicKey: string | null } = { publicKey: null };

jest.mock('@/src/store/walletStore', () => ({
  useWalletStore: () => ({ publicKey: mockWalletState.publicKey, initialize: jest.fn() }),
}));

jest.mock('@/lib/api-client', () => ({
  closePool: jest.fn(),
  submitSignedXdr: jest.fn(),
  withdrawPool: jest.fn(),
  fetchPoolComments: jest.fn(),
  createPoolComment: jest.fn(),
  updatePoolComment: jest.fn(),
  deletePoolComment: jest.fn(),
}));

jest.mock('@stellar/freighter-api', () => ({
  signTransaction: jest.fn(),
}));

const mockPool = {
  id: '404',
  title: 'Test Pool',
  description: 'A pool for testing',
  creator: 'GCREATOR',
  token: 'XLM',
  targetAmount: '1000',
  currentAmount: '100',
  deadline: '2099-01-01T00:00:00.000Z',
  status: 'active',
};

const mockComment = {
  id: 'c1',
  poolId: '404',
  author: 'GAUTHOR',
  content: 'Existing comment',
  createdAt: '2024-01-01T00:00:00.000Z',
};

const mockFetchPoolComments = fetchPoolComments as jest.Mock;
const mockCreatePoolComment = createPoolComment as jest.Mock;
const mockUpdatePoolComment = updatePoolComment as jest.Mock;
const mockDeletePoolComment = deletePoolComment as jest.Mock;

describe('PoolDetailPage', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockWalletState.publicKey = null;
    mockFetchPoolComments.mockResolvedValue([]);
    mockCreatePoolComment.mockResolvedValue(mockComment);
    mockUpdatePoolComment.mockResolvedValue(mockComment);
    mockDeletePoolComment.mockResolvedValue(undefined);
  });

  it('renders a not-found state for a missing pool id', () => {
    (usePoolsStore as jest.Mock).mockReturnValue({
      currentPool: null,
      poolLoading: false,
      fetchPool: jest.fn().mockResolvedValue(null),
    });

    render(<PoolDetailPage />);

    expect(screen.getByText('Pool not found')).toBeInTheDocument();
  });

  describe('comments', () => {
    beforeEach(() => {
      (usePoolsStore as jest.Mock).mockReturnValue({
        currentPool: mockPool,
        poolLoading: false,
        fetchPool: jest.fn().mockResolvedValue(mockPool),
      });
    });

    it('loads comments from the backend on mount', async () => {
      mockFetchPoolComments.mockResolvedValue([mockComment]);

      render(<PoolDetailPage />);

      await waitFor(() => {
        expect(mockFetchPoolComments).toHaveBeenCalledWith('404');
      });
      expect(await screen.findByText('Existing comment')).toBeInTheDocument();
    });

    it('posts a comment through the API and shows it optimistically', async () => {
      mockWalletState.publicKey = 'GAUTHOR';
      let resolveCreate: (value: typeof mockComment) => void = () => {};
      mockCreatePoolComment.mockReturnValue(
        new Promise((resolve) => {
          resolveCreate = resolve;
        }),
      );

      render(<PoolDetailPage />);

      const input = await screen.findByPlaceholderText(/comment/i);
      fireEvent.change(input, { target: { value: 'New comment' } });
      fireEvent.click(screen.getByRole('button', { name: /post/i }));

      expect(await screen.findByText('New comment')).toBeInTheDocument();
      expect(mockCreatePoolComment).toHaveBeenCalledWith('404', 'New comment');

      resolveCreate({ ...mockComment, id: 'c2', content: 'New comment' });
      await waitFor(() => {
        expect(screen.getByText('New comment')).toBeInTheDocument();
      });
    });

    it('rolls back the optimistic comment when posting fails', async () => {
      mockWalletState.publicKey = 'GAUTHOR';
      mockCreatePoolComment.mockRejectedValue(new Error('network error'));

      render(<PoolDetailPage />);

      const input = await screen.findByPlaceholderText(/comment/i);
      fireEvent.change(input, { target: { value: 'Doomed comment' } });
      fireEvent.click(screen.getByRole('button', { name: /post/i }));

      await waitFor(() => {
        expect(screen.queryByText('Doomed comment')).not.toBeInTheDocument();
      });
    });

    it('disables posting when unauthenticated', async () => {
      mockWalletState.publicKey = null;

      render(<PoolDetailPage />);

      const input = await screen.findByPlaceholderText(/comment/i);
      fireEvent.change(input, { target: { value: 'Anonymous comment' } });
      const postButton = screen.getByRole('button', { name: /post/i });

      expect(postButton).toBeDisabled();
      fireEvent.click(postButton);
      expect(mockCreatePoolComment).not.toHaveBeenCalled();
    });
  });
});
