import React from 'react';
import { render, screen } from '@testing-library/react';
import DashboardPage from '@/app/dashboard/page';

jest.mock('next/link', () => ({
  __esModule: true,
  default: ({ href, children }: { href: string; children: React.ReactNode }) => (
    <a href={href}>{children}</a>
  ),
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
  useWalletStore: () => ({ publicKey: null, initialize: jest.fn() }),
}));

jest.mock('@/lib/api-client', () => ({
  apiClient: { get: jest.fn() },
  submitSignedXdr: jest.fn(),
  withdrawPool: jest.fn(),
  closePool: jest.fn(),
}));

jest.mock('@stellar/freighter-api', () => ({
  signTransaction: jest.fn(),
}));

describe('DashboardPage', () => {
  it('renders the pool summary heading for the dashboard', () => {
    render(<DashboardPage />);

    expect(screen.getByRole('heading', { name: 'My Pools' })).toBeInTheDocument();
  });
});