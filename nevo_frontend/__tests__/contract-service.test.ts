// Must be set before the module is loaded
process.env.NEXT_PUBLIC_CONTRACT_ID =
  'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';

import { Account } from '@stellar/stellar-sdk';
import { scValToNative } from '@stellar/stellar-sdk';

const TEST_PUBKEY = 'GAVQGTVZ25JJSLUC72LD5T73CKE4EHUEKOKY7CKUF4GSESY5V726ISXJ';

const mockPrepareTransaction = jest
  .fn()
  .mockImplementation((tx) => Promise.resolve(tx));
const mockGetAccount = jest.fn();
const mockSimulateTransaction = jest.fn();

jest.mock('@stellar/stellar-sdk/rpc', () => ({
  Server: jest.fn().mockImplementation(() => ({
    getAccount: mockGetAccount,
    prepareTransaction: mockPrepareTransaction,
    simulateTransaction: mockSimulateTransaction,
  })),
}));

// Mock scValToNative inline (jest.mock is hoisted, so we can't reference
// external variables from the factory — create the mock inside the factory).
jest.mock('@stellar/stellar-sdk', () => {
  const actual = jest.requireActual('@stellar/stellar-sdk');
  return {
    ...actual,
    scValToNative: jest.fn(() => 42),
    // Keep all other exports from the real module (Keypair, TransactionBuilder, etc.)
  };
});

import { contractService } from '../lib/contract-service';

beforeEach(() => {
  mockGetAccount.mockResolvedValue(new Account(TEST_PUBKEY, '100'));
  jest.clearAllMocks();
});

describe('ContractService.getPoolCount', () => {
  beforeEach(() => {
    // Reset scValToNative mock to return default value
    (scValToNative as jest.Mock).mockReturnValue(42);
  });

  it('returns the pool count when the RPC simulation succeeds', async () => {
    mockSimulateTransaction.mockResolvedValue({
      result: {
        retval: 'mock-sc-val',
      },
    });

    const count = await contractService.getPoolCount();

    expect(count).toBe(42);
    expect(mockSimulateTransaction).toHaveBeenCalledTimes(1);
    expect(scValToNative).toHaveBeenCalledWith('mock-sc-val');
  });

  it('returns -1 when the RPC simulation has an error', async () => {
    mockSimulateTransaction.mockResolvedValue({
      error: 'Contract execution error',
    });

    const count = await contractService.getPoolCount();

    expect(count).toBe(-1);
    expect(scValToNative).not.toHaveBeenCalled();
  });

  it('returns -1 when the simulation result has no retval', async () => {
    mockSimulateTransaction.mockResolvedValue({
      result: {},
    });

    const count = await contractService.getPoolCount();

    expect(count).toBe(-1);
  });

  it('returns -1 when the RPC server throws', async () => {
    mockSimulateTransaction.mockRejectedValue(new Error('RPC timeout'));

    const count = await contractService.getPoolCount();

    expect(count).toBe(-1);
  });

  it('handles boundary numeric values', async () => {
    mockSimulateTransaction.mockResolvedValue({
      result: { retval: 'mock' },
    });

    (scValToNative as jest.Mock).mockReturnValue(0);
    expect(await contractService.getPoolCount()).toBe(0);

    (scValToNative as jest.Mock).mockReturnValue(1);
    expect(await contractService.getPoolCount()).toBe(1);

    (scValToNative as jest.Mock).mockReturnValue(999);
    expect(await contractService.getPoolCount()).toBe(999);
  });
});

describe('ContractService.buildDonateTransaction', () => {
  beforeEach(() => {
    mockGetAccount.mockResolvedValue(new Account(TEST_PUBKEY, '100'));
    mockPrepareTransaction.mockImplementation((tx) => Promise.resolve(tx));
  });

  it('returns a non-empty XDR string', async () => {
    const xdr = await contractService.buildDonateTransaction(
      1,
      TEST_PUBKEY,
      BigInt(5_000_000)
    );
    expect(typeof xdr).toBe('string');
    expect(xdr.length).toBeGreaterThan(0);
  });

  it('produces a base64-decodable XDR envelope', async () => {
    const xdr = await contractService.buildDonateTransaction(
      42,
      TEST_PUBKEY,
      BigInt(10_000_000_000)
    );
    expect(() => Buffer.from(xdr, 'base64')).not.toThrow();
    expect(Buffer.from(xdr, 'base64').length).toBeGreaterThan(0);
  });

  it('handles large bigint amounts without serialization errors', async () => {
    const largeAmount = BigInt('9007199254740992'); // > Number.MAX_SAFE_INTEGER
    await expect(
      contractService.buildDonateTransaction(1, TEST_PUBKEY, largeAmount)
    ).resolves.toBeDefined();
  });
});
