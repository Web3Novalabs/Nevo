import React from 'react';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import TransactionsPage from '@/app/transactions/page';

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

jest.mock('@/components/CopyButton', () => ({
  CopyButton: () => null,
}));

jest.mock('@/components', () => ({
  BackToTopButton: () => null,
}));

jest.mock('@/lib/api-client', () => ({
  apiClient: { get: jest.fn() },
}));

// eslint-disable-next-line @typescript-eslint/no-var-requires
const { apiClient } = require('@/lib/api-client');

const emptyPage = {
  data: [],
  meta: { page: 1, pageSize: 10, total: 0, totalPages: 0 },
};

const populatedPage = {
  data: [
    {
      id: 'tx-1',
      type: 'payment',
      amount: 100,
      asset: 'XLM',
      status: 'completed',
      createdAt: '2024-01-01T00:00:00.000Z',
    },
  ],
  meta: { page: 2, pageSize: 10, total: 25, totalPages: 3 },
};

describe('TransactionsPage', () => {
  beforeEach(() => {
    apiClient.get.mockReset();
  });

  it('renders the empty state when transaction history is empty', async () => {
    apiClient.get.mockResolvedValue(emptyPage);

    render(<TransactionsPage />);

    await waitFor(() => {
      expect(screen.getByText('No transactions yet')).toBeInTheDocument();
    });
  });

  it('renders an error state with a retry button on API failure', async () => {
    apiClient.get.mockRejectedValueOnce(new Error('boom'));

    render(<TransactionsPage />);

    await waitFor(() => {
      expect(screen.getByText(/failed to load transactions/i)).toBeInTheDocument();
    });

    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
    // Should not show the empty state on failure
    expect(screen.queryByText('No transactions yet')).not.toBeInTheDocument();
  });

  it('retries the request when the retry button is clicked', async () => {
    apiClient.get.mockRejectedValueOnce(new Error('boom'));
    apiClient.get.mockResolvedValueOnce(emptyPage);

    render(<TransactionsPage />);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole('button', { name: /retry/i }));

    await waitFor(() => {
      expect(screen.getByText('No transactions yet')).toBeInTheDocument();
    });

    expect(apiClient.get).toHaveBeenCalledTimes(2);
  });

  it('sends filters and pagination as query params to the server', async () => {
    apiClient.get.mockResolvedValue(populatedPage);

    render(<TransactionsPage />);

    await waitFor(() => {
      expect(apiClient.get).toHaveBeenCalled();
    });

    const firstCallUrl = apiClient.get.mock.calls[0][0] as string;
    expect(firstCallUrl).toContain('/transactions');
    expect(firstCallUrl).toContain('page=1');

    // Apply a filter
    const typeFilter = screen.getByLabelText(/type/i);
    fireEvent.change(typeFilter, { target: { value: 'payment' } });

    await waitFor(() => {
      const lastCallUrl = apiClient.get.mock.calls[
        apiClient.get.mock.calls.length - 1
      ][0] as string;
      expect(lastCallUrl).toContain('type=payment');
      expect(lastCallUrl).toContain('page=1');
    });
  });

  it('renders pagination from server metadata and requests the next page', async () => {
    apiClient.get.mockResolvedValue(populatedPage);

    render(<TransactionsPage />);

    await waitFor(() => {
      expect(screen.getByText(/page 2 of 3/i)).toBeInTheDocument();
    });

    const nextButton = screen.getByRole('button', { name: /next/i });
    fireEvent.click(nextButton);

    await waitFor(() => {
      const lastCallUrl = apiClient.get.mock.calls[
        apiClient.get.mock.calls.length - 1
      ][0] as string;
      expect(lastCallUrl).toContain('page=3');
    });
  });
});
