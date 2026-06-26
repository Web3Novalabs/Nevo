import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

// ── Mock next/navigation ─────────────────────────────────────────────────
const mockPush = jest.fn();
const mockReplace = jest.fn();

jest.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
    replace: mockReplace,
    back: jest.fn(),
    forward: jest.fn(),
    refresh: jest.fn(),
    prefetch: jest.fn(),
  }),
  usePathname: () => '/pools/new',
  useParams: () => ({}),
}));

// ── Mock wallet store ────────────────────────────────────────────────────
const mockWalletState = {
  publicKey: 'GAVQGTVZ25JJSLUC72LD5T73CKE4EHUEKOKY7CKUF4GSESY5V726ISXJ',
  loading: false,
  initialize: jest.fn(),
  connectWallet: jest.fn(),
  disconnectWallet: jest.fn(),
  refreshBalances: jest.fn(),
  balances: null,
};

const mockGetState = jest.fn(() => mockWalletState);

jest.mock('@/src/store/walletStore', () => ({
  useWalletStore: Object.assign(
    jest.fn(() => mockWalletState),
    { getState: mockGetState }
  ),
}));

// ── Mock stellar-wallets-kit (used by walletStore) ───────────────────────
jest.mock('@/app/stellar-wallets-kit', () => ({
  getPublicKey: jest.fn().mockResolvedValue(null),
  connect: jest.fn(),
  disconnect: jest.fn(),
}));

// ── Mock contract-service ────────────────────────────────────────────────
const mockBuildCreatePoolTransaction = jest.fn();
const mockGetPoolCount = jest.fn();

jest.mock('@/lib/contract-service', () => ({
  contractService: {
    buildCreatePoolTransaction: mockBuildCreatePoolTransaction,
    getPoolCount: mockGetPoolCount,
    buildDonateTransaction: jest.fn(),
  },
}));

// ── Mock freighter signing ───────────────────────────────────────────────
const mockSignTransaction = jest.fn();

jest.mock('@stellar/freighter-api', () => ({
  signTransaction: mockSignTransaction,
  isConnected: jest.fn(),
  getPublicKey: jest.fn(),
}));

// ── Mock api-client creation/submission ──────────────────────────────────
const mockCreatePool = jest.fn();
const mockSubmitSignedXdr = jest.fn();

jest.mock('@/lib/api-client', () => {
  const actual = jest.requireActual('@/lib/api-client');
  return {
    ...actual,
    createPool: mockCreatePool,
    submitSignedXdr: mockSubmitSignedXdr,
  };
});

// ── Import the page AFTER all mocks ──────────────────────────────────────
import CreatePoolPage from '@/app/pools/new/page';

describe('Pool creation form flow', () => {
  beforeEach(() => {
    jest.clearAllMocks();

    // Default mock implementations
    mockBuildCreatePoolTransaction.mockResolvedValue('AAAA...mocked-xdr...');
    mockSignTransaction.mockResolvedValue({
      signedTxXdr: 'AAAA...signed-xdr...',
      error: undefined,
    });
    mockSubmitSignedXdr.mockResolvedValue({ txHash: '0xdeadbeef' });
    mockGetPoolCount.mockResolvedValue(1);
    mockCreatePool.mockResolvedValue({ id: 'uuid-new-pool', status: 'Active' });
  });

  it('renders the multi-step form with step 1 active', () => {
    render(<CreatePoolPage />);

    expect(screen.getByText('Create a Pool')).toBeInTheDocument();
    expect(screen.getByText('Basic Information')).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Next: Goal/i })
    ).toBeInTheDocument();
  });

  it('shows field validation errors when submitting with empty fields', async () => {
    const user = userEvent.setup();
    render(<CreatePoolPage />);

    // Navigate to step 3 (preview) without filling fields
    const nextBtn = screen.getByRole('button', { name: /Next: Goal/i });
    await user.click(nextBtn);

    // Should still be on step 1 with validation errors
    expect(screen.getByText('Title is required.')).toBeInTheDocument();
    expect(screen.getByText('Description is required.')).toBeInTheDocument();
    expect(screen.getByText('Please select a category.')).toBeInTheDocument();
  });

  // Full end-to-end form submission test
  it('completes the full creation flow: fill, submit, and see success', async () => {
    const user = userEvent.setup();
    render(<CreatePoolPage />);

    // ── Step 1: Fill basics ──────────────────────────────────────────
    await user.type(screen.getByLabelText(/^Title/), 'Clean Water Fund');
    await user.type(
      screen.getByLabelText(/^Description/),
      'Providing clean drinking water to communities in rural areas across Africa.'
    );

    // Select a category
    const categorySelect = screen.getByLabelText(/^Category/);
    await user.selectOptions(categorySelect, 'Humanitarian');

    // Go to step 2
    await user.click(screen.getByRole('button', { name: /Next: Goal/i }));

    // ── Step 2: Fill goal & duration ────────────────────────────────
    await user.type(screen.getByPlaceholderText('e.g. 5000'), '5000');

    // Select a duration option (click the 30 days radio)
    await user.click(screen.getByRole('radio', { name: /30 days/i }));

    // Go to step 3
    await user.click(screen.getByRole('button', { name: /Preview Pool/i }));

    // ── Step 3: Preview & Submit ───────────────────────────────────
    // Verify preview shows our data
    expect(screen.getByText('Clean Water Fund')).toBeInTheDocument();
    expect(screen.getByText('5,000 XLM')).toBeInTheDocument();

    // Submit the form
    await user.click(screen.getByRole('button', { name: /Create Pool/i }));

    // ── Verify contract interaction flow ───────────────────────────
    await waitFor(() => {
      expect(mockBuildCreatePoolTransaction).toHaveBeenCalledWith(
        mockWalletState.publicKey,
        'Clean Water Fund',
        expect.stringContaining('Providing clean drinking water'),
        expect.any(BigInt)
      );
    });

    expect(mockSignTransaction).toHaveBeenCalledWith('AAAA...mocked-xdr...', {
      networkPassphrase: expect.any(String),
    });

    expect(mockSubmitSignedXdr).toHaveBeenCalledWith('AAAA...signed-xdr...');

    expect(mockGetPoolCount).toHaveBeenCalled();

    // ── Verify pool metadata is saved to DB ────────────────────────
    expect(mockCreatePool).toHaveBeenCalledWith({
      contractPoolId: '2', // count (1) + 1
      creatorWallet: mockWalletState.publicKey,
      goal: '50000000000',
      title: 'Clean Water Fund',
      description: expect.stringContaining('Providing clean drinking water'),
      category: 'Humanitarian',
      imageUrl: undefined,
    });

    // ── Verify success screen appears ──────────────────────────────
    await waitFor(() => {
      expect(screen.getByText('Pool Created!')).toBeInTheDocument();
    });
  });

  it('shows an error when the contract build fails', async () => {
    mockBuildCreatePoolTransaction.mockRejectedValue(
      new Error('RPC connection failed')
    );

    const user = userEvent.setup();
    render(<CreatePoolPage />);

    // Fill step 1
    await user.type(screen.getByLabelText(/^Title/), 'Error Test Pool');
    await user.type(
      screen.getByLabelText(/^Description/),
      'Testing the error handling flow for pool creation failure.'
    );
    await user.selectOptions(screen.getByLabelText(/^Category/), 'Technology');
    await user.click(screen.getByRole('button', { name: /Next: Goal/i }));

    // Fill step 2
    await user.type(screen.getByPlaceholderText('e.g. 5000'), '1000');
    await user.click(screen.getByRole('radio', { name: /30 days/i }));
    await user.click(screen.getByRole('button', { name: /Preview Pool/i }));

    // Try to submit
    await user.click(screen.getByRole('button', { name: /Create Pool/i }));

    // Should show the error
    await waitFor(() => {
      expect(screen.getByText('RPC connection failed')).toBeInTheDocument();
    });
  });

  it('shows an error when the wallet is not connected', async () => {
    // Temporarily make wallet not connected
    mockGetState.mockReturnValueOnce({
      ...mockWalletState,
      publicKey: null,
    });

    const user = userEvent.setup();
    render(<CreatePoolPage />);

    // Fill step 1
    await user.type(screen.getByLabelText(/^Title/), 'No Wallet Pool');
    await user.type(
      screen.getByLabelText(/^Description/),
      'Testing wallet not connected error handling.'
    );
    await user.selectOptions(screen.getByLabelText(/^Category/), 'Health');
    await user.click(screen.getByRole('button', { name: /Next: Goal/i }));

    // Fill step 2
    await user.type(screen.getByPlaceholderText('e.g. 5000'), '200');
    await user.click(screen.getByRole('radio', { name: /7 days/i }));
    await user.click(screen.getByRole('button', { name: /Preview Pool/i }));

    // Try to submit
    await user.click(screen.getByRole('button', { name: /Create Pool/i }));

    await waitFor(() => {
      // The error is displayed in a red alert box, distinct from the
      // description text which also mentions "wallet not connected"
      expect(
        screen.getByText(
          'Wallet not connected. Please connect your wallet first.'
        )
      ).toBeInTheDocument();
    });
  });

  it('shows an error when createPool API fails after XDR submission', async () => {
    mockCreatePool.mockRejectedValue(new Error('Database insert failed'));

    const user = userEvent.setup();
    render(<CreatePoolPage />);

    // Fill step 1
    await user.type(screen.getByLabelText(/^Title/), 'API Error Pool');
    await user.type(
      screen.getByLabelText(/^Description/),
      'Testing API error handling after XDR submission.'
    );
    await user.selectOptions(screen.getByLabelText(/^Category/), 'Education');
    await user.click(screen.getByRole('button', { name: /Next: Goal/i }));

    // Fill step 2
    await user.type(screen.getByPlaceholderText('e.g. 5000'), '300');
    await user.click(screen.getByRole('radio', { name: /14 days/i }));
    await user.click(screen.getByRole('button', { name: /Preview Pool/i }));

    // Submit
    await user.click(screen.getByRole('button', { name: /Create Pool/i }));

    await waitFor(() => {
      expect(screen.getByText('Database insert failed')).toBeInTheDocument();
    });
  });

  it('falls back to a local prefix ID when getPoolCount returns -1', async () => {
    mockGetPoolCount.mockResolvedValue(-1);

    const user = userEvent.setup();
    render(<CreatePoolPage />);

    // Fill step 1
    await user.type(screen.getByLabelText(/^Title/), 'Fallback Pool');
    await user.type(
      screen.getByLabelText(/^Description/),
      'Testing fallback contractPoolId when RPC is unavailable.'
    );
    await user.selectOptions(screen.getByLabelText(/^Category/), 'Community');
    await user.click(screen.getByRole('button', { name: /Next: Goal/i }));

    // Fill step 2
    await user.type(screen.getByPlaceholderText('e.g. 5000'), '100');
    await user.click(screen.getByRole('radio', { name: /30 days/i }));
    await user.click(screen.getByRole('button', { name: /Preview Pool/i }));

    // Submit
    await user.click(screen.getByRole('button', { name: /Create Pool/i }));

    // Should use local- prefix fallback
    await waitFor(() => {
      expect(mockCreatePool).toHaveBeenCalled();
      const callArgs = mockCreatePool.mock.calls[0][0];
      expect(callArgs.contractPoolId).toMatch(/^local-\d+$/);
    });

    // Should still show success since createPool succeeds
    await waitFor(() => {
      expect(screen.getByText('Pool Created!')).toBeInTheDocument();
    });
  });
});
