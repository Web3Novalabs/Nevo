import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DonateModal } from '@/components/DonateModal';
import type { Pool } from '@/src/store/poolsStore';

const TOKEN_ADDRESS =
  'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC';

jest.mock('@/src/store/walletStore', () => ({
  useWalletStore: jest.fn(),
}));

jest.mock('@/src/store/donationsStore', () => ({
  useDonationsStore: jest.fn(),
}));

jest.mock('@/src/store/notificationsStore', () => ({
  useNotificationsStore: jest.fn(),
}));

jest.mock('@/lib/contract-service', () => ({
  contractService: {
    buildDonateTransaction: jest.fn().mockResolvedValue('unsigned-xdr'),
  },
  getDonationTokenAddress: jest.fn(
    () => 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC'
  ),
}));

jest.mock('@/lib/errors', () => ({
  parseApiError: (error: unknown) => String(error),
}));

jest.mock(
  '../lib/useXdrTransaction',
  () => {
    const signAndSubmit = jest.fn().mockResolvedValue('tx-hash');
    return {
      useXdrTransaction: () => ({ signAndSubmit }),
      __signAndSubmit: signAndSubmit,
    };
  },
  { virtual: true }
);

jest.mock('@stellar/freighter-api', () => ({
  signTransaction: jest.fn(),
}));

jest.mock('@stellar/stellar-sdk', () => ({
  Networks: { TESTNET: 'Test SDF Network ; September 2015' },
  Transaction: class {},
  TransactionBuilder: {
    fromXDR: jest.fn(),
  },
}));

jest.mock('@stellar/stellar-sdk/rpc', () => ({
  Server: jest.fn(),
}));

const { useWalletStore } = jest.requireMock('@/src/store/walletStore');
const { useDonationsStore } = jest.requireMock('@/src/store/donationsStore');
const { useNotificationsStore } = jest.requireMock(
  '@/src/store/notificationsStore'
);
const { contractService, getDonationTokenAddress } = jest.requireMock(
  '@/lib/contract-service'
);
const { __signAndSubmit: signAndSubmit } = jest.requireMock(
  '../lib/useXdrTransaction'
);

const pool: Pool = {
  id: '1',
  title: 'Community Pool',
  description: 'A community pool for local initiatives',
  category: 'Community',
  raised: 100,
  target: 1000,
  status: 'Active',
  imageColor: '#123456',
};

const addDonation = jest.fn();
const addNotification = jest.fn();

beforeEach(() => {
  jest.clearAllMocks();
  (useWalletStore as jest.Mock).mockReturnValue({
    publicKey: 'wallet-address',
    balances: { xlm: '10', usdc: '5' },
  });
  (useDonationsStore as jest.Mock).mockReturnValue({ addDonation });
  (useNotificationsStore as jest.Mock).mockReturnValue({ addNotification });
  (contractService.buildDonateTransaction as jest.Mock).mockResolvedValue(
    'unsigned-xdr'
  );
  (signAndSubmit as jest.Mock).mockResolvedValue('tx-hash');
  (getDonationTokenAddress as jest.Mock).mockReturnValue(TOKEN_ADDRESS);
});

describe('DonateModal', () => {
  it('keeps focus inside the modal when tabbing through focusable elements', async () => {
    const user = userEvent.setup();
    render(
      <>
        <button type="button">Open donate</button>
        <DonateModal pool={pool} onClose={jest.fn()} />
      </>
    );

    const closeButton = screen.getByRole('button', {
      name: /close donation modal/i,
    });
    const trigger = screen.getByRole('button', { name: /open donate/i });

    closeButton.focus();
    await user.tab();

    expect(trigger).not.toHaveFocus();
    expect(screen.getByRole('dialog')).toContainElement(document.activeElement);
  });

  it('restores focus to the triggering button when the modal closes', () => {
    const onClose = jest.fn();
    render(
      <>
        <button type="button">Open donate</button>
        <DonateModal pool={pool} onClose={onClose} />
      </>
    );

    const trigger = screen.getByRole('button', { name: /open donate/i });
    trigger.focus();

    const closeButton = screen.getByRole('button', {
      name: /close donation modal/i,
    });
    closeButton.click();

    expect(trigger).toHaveFocus();
  });

  it('builds a token-backed donation and signs it before reporting success', async () => {
    const user = userEvent.setup();
    const onDonationSuccess = jest.fn();
    render(
      <DonateModal
        pool={pool}
        onClose={jest.fn()}
        onDonationSuccess={onDonationSuccess}
      />
    );

    await user.type(screen.getByLabelText(/amount/i), '2');
    await user.click(screen.getByRole('button', { name: /donate 2 XLM/i }));

    await waitFor(() => {
      expect(contractService.buildDonateTransaction).toHaveBeenCalledWith(
        1,
        'wallet-address',
        BigInt(20_000_000),
        TOKEN_ADDRESS
      );
    });
    expect(getDonationTokenAddress).toHaveBeenCalledWith('XLM');
    expect(signAndSubmit).toHaveBeenCalledWith(
      'unsigned-xdr',
      'wallet-address'
    );
    expect(await screen.findByText(/donation successful/i)).toBeInTheDocument();
    expect(onDonationSuccess).toHaveBeenCalledTimes(1);
  });

  it('uses the selected asset token and its base-unit precision', async () => {
    const user = userEvent.setup();
    render(<DonateModal pool={pool} onClose={jest.fn()} />);

    await user.click(screen.getByRole('radio', { name: 'USDC' }));
    await user.type(screen.getByLabelText(/amount/i), '1.5');
    await user.click(screen.getByRole('button', { name: /donate 1.5 USDC/i }));

    await waitFor(() => {
      expect(contractService.buildDonateTransaction).toHaveBeenCalledWith(
        1,
        'wallet-address',
        BigInt(1_500_000),
        TOKEN_ADDRESS
      );
    });
    expect(getDonationTokenAddress).toHaveBeenCalledWith('USDC');
  });

  it('shows a clear error and does not sign when the balance is too low', async () => {
    const user = userEvent.setup();
    (useWalletStore as jest.Mock).mockReturnValue({
      publicKey: 'wallet-address',
      balances: { xlm: '1', usdc: '5' },
    });
    render(<DonateModal pool={pool} onClose={jest.fn()} />);

    await user.type(screen.getByLabelText(/amount/i), '2');
    await user.click(screen.getByRole('button', { name: /donate 2 XLM/i }));

    expect(
      await screen.findByText(/insufficient XLM balance. Available: 1 XLM/i)
    ).toBeInTheDocument();
    expect(contractService.buildDonateTransaction).not.toHaveBeenCalled();
    expect(signAndSubmit).not.toHaveBeenCalled();
  });
});
