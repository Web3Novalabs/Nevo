import React from 'react';
import { render, screen } from '@testing-library/react';
import PoolDetailPage from '@/app/pools/[id]/page';
import { usePoolsStore } from '@/src/store/poolsStore';

jest.mock('next/link', () => ({
  __esModule: true,
  default: ({ href, children }: { href: string; children: React.ReactNode }) => (
    <a href={href}>{children}</a>
  ),
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
  ErrorBoundary: ({ children }: { children: React.ReactNode }) => <>{children}</>,
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

jest.mock('@/src/store/walletStore', () => ({
  useWalletStore: () => ({ publicKey: null, initialize: jest.fn() }),
}));

jest.mock('@/lib/api-client', () => ({
  closePool: jest.fn(),
  submitSignedXdr: jest.fn(),
  withdrawPool: jest.fn(),
}));

jest.mock('@stellar/freighter-api', () => ({
  signTransaction: jest.fn(),
}));

describe('PoolDetailPage', () => {
  it('renders a not-found state for a missing pool id', () => {
    (usePoolsStore as jest.Mock).mockReturnValue({
      currentPool: null,
      poolLoading: false,
      fetchPool: jest.fn().mockResolvedValue(null),
    });

    render(<PoolDetailPage />);

    expect(screen.getByText('Pool not found')).toBeInTheDocument();
  });
});