import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { DonateModal } from '@/components/DonateModal';
import { useWalletStore } from '@/src/store/walletStore';
import { contractService } from '@/lib/contract-service';
import type { Pool } from '@/src/store/poolsStore';

// Mock contractService
jest.mock('@/lib/contract-service', () => ({
  contractService: {
    buildDonateTransaction: jest.fn(),
  },
}));

const mockPoolActive: Pool = {
  id: '1',
  title: 'Active Cause',
  description: 'An active fundraising cause',
  category: 'Humanitarian',
  status: 'Active',
  target: 10000,
  raised: 500,
  imageColor: '#123456',
  creator: 'CREATOR_KEY',
};

const mockPoolCompleted: Pool = {
  id: '2',
  title: 'Completed Cause',
  description: 'A completed cause',
  category: 'Humanitarian',
  status: 'Completed',
  target: 10000,
  raised: 10000,
  imageColor: '#123456',
  creator: 'CREATOR_KEY',
};

describe('DonateModal', () => {
  const onClose = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    // Default wallet mock state
    useWalletStore.setState({
      publicKey: 'GDONOR1234567890',
      balances: { xlm: '100', usdc: '50' },
      loading: false,
    });
  });

  it('renders input form with asset balance', () => {
    render(<DonateModal pool={mockPoolActive} onClose={onClose} />);
    expect(
      screen.getByRole('heading', { name: 'Donate to Pool' })
    ).toBeInTheDocument();
    expect(screen.getByText('Active Cause')).toBeInTheDocument();
    expect(screen.getByText(/Available:/i)).toHaveTextContent(
      'Available: 100 XLM'
    );
  });

  it('transitions to error state if pool is completed (closed)', async () => {
    render(<DonateModal pool={mockPoolCompleted} onClose={onClose} />);

    // Set amount and submit
    const amountInput = screen.getByRole('spinbutton', { name: /amount/i });
    fireEvent.change(amountInput, { target: { value: '10' } });

    const submitBtn = screen.getByRole('button', { name: /donate/i });
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(screen.getByText('Donation failed')).toBeInTheDocument();
      expect(screen.getByText('Pool is closed')).toBeInTheDocument();
    });
  });

  it('transitions to error state if balance is insufficient', async () => {
    render(<DonateModal pool={mockPoolActive} onClose={onClose} />);

    // Set amount greater than available balance (100 XLM)
    const amountInput = screen.getByRole('spinbutton', { name: /amount/i });
    fireEvent.change(amountInput, { target: { value: '150' } });

    const submitBtn = screen.getByRole('button', { name: /donate/i });
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(screen.getByText('Donation failed')).toBeInTheDocument();
      expect(screen.getByText('Insufficient balance')).toBeInTheDocument();
    });
  });

  it('transitions to error state if contractService throws an error', async () => {
    (contractService.buildDonateTransaction as jest.Mock).mockRejectedValue(
      new Error('Stellar error mock details')
    );

    render(<DonateModal pool={mockPoolActive} onClose={onClose} />);

    const amountInput = screen.getByRole('spinbutton', { name: /amount/i });
    fireEvent.change(amountInput, { target: { value: '10' } });

    const submitBtn = screen.getByRole('button', { name: /donate/i });
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(screen.getByText('Donation failed')).toBeInTheDocument();
      expect(screen.getByText('Transaction failed')).toBeInTheDocument();
    });
  });

  it('resets error state back to input form when Try Again is clicked', async () => {
    render(<DonateModal pool={mockPoolCompleted} onClose={onClose} />);

    const amountInput = screen.getByRole('spinbutton', { name: /amount/i });
    fireEvent.change(amountInput, { target: { value: '10' } });

    const submitBtn = screen.getByRole('button', { name: /donate/i });
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(screen.getByText('Donation failed')).toBeInTheDocument();
    });

    const tryAgainBtn = screen.getByRole('button', { name: /try again/i });
    fireEvent.click(tryAgainBtn);

    expect(
      screen.getByRole('heading', { name: 'Donate to Pool' })
    ).toBeInTheDocument();
    expect(screen.queryByText('Donation failed')).not.toBeInTheDocument();
  });
});
