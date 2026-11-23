// Must be set before the module is loaded
process.env.NEXT_PUBLIC_CONTRACT_ID =
  'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';

import {
  Account,
  Address,
  Asset,
  Contract,
  Keypair,
  Networks,
  nativeToScVal,
  scValToNative,
} from '@stellar/stellar-sdk';

const TEST_PUBKEY = 'GAVQGTVZ25JJSLUC72LD5T73CKE4EHUEKOKY7CKUF4GSESY5V726ISXJ';
const CUSTOM_TOKEN = 'CA2E53VHFZ6YSWQIEIPBXJQGT6VW3VKWWZO555XKRQXYJ63GEBJJGHY7';

const mockPrepareTransaction = jest.fn();
const mockGetAccount = jest.fn();
const mockSimulateTransaction = jest.fn();

jest.mock('@stellar/stellar-sdk/rpc', () => ({
  Server: jest.fn().mockImplementation(() => ({
    getAccount: (...args: unknown[]) => mockGetAccount(...args),
    prepareTransaction: (...args: unknown[]) => mockPrepareTransaction(...args),
    simulateTransaction: (...args: unknown[]) =>
      mockSimulateTransaction(...args),
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
import { ApplicationStatus } from '../lib/contract-types';

let callSpy: jest.SpyInstance;

beforeEach(() => {
  jest.clearAllMocks();
  mockGetAccount.mockReset().mockResolvedValue(new Account(TEST_PUBKEY, '100'));
  mockPrepareTransaction
    .mockReset()
    .mockImplementation((tx) => Promise.resolve(tx));
  mockSimulateTransaction.mockReset();
  (scValToNative as jest.Mock).mockReturnValue(42);
  callSpy = jest.spyOn(Contract.prototype, 'call');
});

afterEach(() => {
  jest.restoreAllMocks();
});

describe('ContractService.getPoolCount', () => {
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

describe('ContractService.buildCreatePoolTransaction', () => {
  it('encodes the goal as an i128 scVal', async () => {
    const goal = BigInt(50_000_000);

    const xdr = await contractService.buildCreatePoolTransaction(
      TEST_PUBKEY,
      'Test Pool',
      goal,
      30
    );

    expect(typeof xdr).toBe('string');
    expect(xdr.length).toBeGreaterThan(0);

    const [method, , , goalArg] = callSpy.mock.calls[0];
    expect(method).toBe('create_pool');
    expect(goalArg.switch().name).toBe('scvI128');
    expect(goalArg.toXDR('base64')).toBe(
      nativeToScVal(goal, { type: 'i128' }).toXDR('base64')
    );
  });
});

describe('ContractService.buildDonateTransaction', () => {
  it('builds a token-backed donation with the native XLM SAC token', async () => {
    const amount = BigInt(5_000_000);

    const xdr = await contractService.buildDonateTransaction(
      7,
      TEST_PUBKEY,
      amount
    );

    expect(typeof xdr).toBe('string');
    expect(xdr.length).toBeGreaterThan(0);

    const [method, pool, donor, token, donationAmount] = callSpy.mock.calls[0];
    expect(method).toBe('donate_with_token');
    expect(pool.u32()).toBe(7);
    expect(donor.toXDR('base64')).toBe(
      Address.account(Keypair.fromPublicKey(TEST_PUBKEY).rawPublicKey())
        .toScVal()
        .toXDR('base64')
    );

    const nativeSac = Asset.native().contractId(Networks.TESTNET);
    expect(token.toXDR('base64')).toBe(
      new Address(nativeSac).toScVal().toXDR('base64')
    );
    expect(donationAmount.switch().name).toBe('scvI128');
    expect(donationAmount.toXDR('base64')).toBe(
      nativeToScVal(amount, { type: 'i128' }).toXDR('base64')
    );
  });

  it('uses the supplied pool token address when provided', async () => {
    await contractService.buildDonateTransaction(
      1,
      TEST_PUBKEY,
      BigInt(10_000_000),
      CUSTOM_TOKEN
    );

    const token = callSpy.mock.calls[0][3];
    expect(token.toXDR('base64')).toBe(
      new Address(CUSTOM_TOKEN).toScVal().toXDR('base64')
    );
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

describe('ContractService.getApplicationStatus', () => {
  it('decodes the enum-based status returned by the contract', async () => {
    mockSimulateTransaction.mockResolvedValue({
      result: { retval: 'mock-sc-val' },
    });

    (scValToNative as jest.Mock).mockReturnValue(ApplicationStatus.Approved);

    const status = await contractService.getApplicationStatus(1, TEST_PUBKEY);

    expect(status).toBe(ApplicationStatus.Approved);
    expect(mockSimulateTransaction).toHaveBeenCalledTimes(1);
    expect(scValToNative).toHaveBeenCalledWith('mock-sc-val');
  });

  it('decodes every enum variant without string conversion', async () => {
    mockSimulateTransaction.mockResolvedValue({
      result: { retval: 'mock-sc-val' },
    });

    for (const variant of [
      ApplicationStatus.Pending,
      ApplicationStatus.Approved,
      ApplicationStatus.Rejected,
    ]) {
      (scValToNative as jest.Mock).mockReturnValue(variant);
      const status = await contractService.getApplicationStatus(1, TEST_PUBKEY);
      expect(status).toBe(variant);
      expect(typeof status).not.toBe('string');
    }
  });

  it('returns Pending when the simulation has an error', async () => {
    mockSimulateTransaction.mockResolvedValue({
      error: 'Contract execution error',
    });

    const status = await contractService.getApplicationStatus(1, TEST_PUBKEY);

    expect(status).toBe(ApplicationStatus.Pending);
    expect(scValToNative).not.toHaveBeenCalled();
  });

  it('returns Pending when the RPC server throws', async () => {
    mockSimulateTransaction.mockRejectedValue(new Error('RPC timeout'));

    const status = await contractService.getApplicationStatus(1, TEST_PUBKEY);

    expect(status).toBe(ApplicationStatus.Pending);
  });
});
