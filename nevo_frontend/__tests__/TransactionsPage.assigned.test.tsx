import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import TransactionsPage from '@/app/transactions/page';

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

jest.mock('@/components/CopyButton', () => ({
  CopyButton: () => null,
}));

jest.mock('@/components', () => ({
  BackToTopButton: () => null,
}));

jest.mock('@/lib/api-client', () => ({
  apiClient: { get: jest.fn().mockResolvedValue([]) },
}));

describe('TransactionsPage', () => {
  it('renders the empty state when transaction history is empty', async () => {
    render(<TransactionsPage />);

    await waitFor(() => {
      expect(screen.getByText('No transactions yet')).toBeInTheDocument();
    });
  });
});