import { HttpStatus } from '@nestjs/common';
import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import {
  TransactionBuilder,
  Networks,
  Keypair,
  nativeToScVal,
  Account,
  xdr,
} from '@stellar/stellar-sdk';
import { ContractService } from './contract.service.js';
import { StellarError } from './stellar.error.js';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const SOURCE = Keypair.random().publicKey();
const TOKEN = Keypair.random().publicKey();
const DONOR = Keypair.random().publicKey();
const NETWORK = Networks.TESTNET;

/** Decode the first InvokeHostFunction operation from a base64-XDR envelope */
function decodeFirstOp(xdrString: string) {
  const tx = TransactionBuilder.fromXDR(xdrString, NETWORK) as any;
  return tx.operations[0];
}

/** Build a minimal fake AccountResponse that StellarRpc.Server.getAccount resolves to */
function fakeAccount(publicKey: string): StellarRpc.Api.GetAccountResponse {
  return {
    id: publicKey,
    sequence: '100',
    accountId: () => publicKey,
    sequenceNumber: () => '100',
    incrementSequenceNumber: () => {},
  } as unknown as StellarRpc.Api.GetAccountResponse;
}

/** Build a SimulateTransactionResponse that carries a return value */
function fakeSimResult(scVal: xdr.ScVal): StellarRpc.Api.SimulateTransactionSuccessResponse {
  return {
    result: { retval: scVal, auth: [] },
    latestLedger: 1,
    minResourceFee: '100',
    transactionData: '',
    cost: { cpuInsns: '0', memBytes: '0' },
    events: [],
    stateChanges: [],
  } as unknown as StellarRpc.Api.SimulateTransactionSuccessResponse;
}

/** Build a SimulateTransactionResponse that carries an error */
function fakeSimError(): StellarRpc.Api.SimulateTransactionErrorResponse {
  return {
    error: 'contract error',
    latestLedger: 1,
    events: [],
  } as unknown as StellarRpc.Api.SimulateTransactionErrorResponse;
}

// ---------------------------------------------------------------------------
// Mock RPC server
// ---------------------------------------------------------------------------

const mockRpcServer = {
  sendTransaction: jest.fn(),
  getAccount: jest.fn(),
  simulateTransaction: jest.fn(),
};

// Patch the constructor so every `new StellarRpc.Server(...)` returns our mock
jest.mock('@stellar/stellar-sdk', () => {
  const actual = jest.requireActual('@stellar/stellar-sdk') as Record<string, unknown>;
  const actualRpc = actual['rpc'] as Record<string, unknown>;
  return {
    ...actual,
    rpc: {
      ...actualRpc,
      Server: jest.fn().mockImplementation(() => mockRpcServer),
    },
  };
});

const mockConfigService = {
  get: (key: string) => {
    if (key === 'STELLAR_RPC_URL') return 'https://soroban-testnet.stellar.org';
    if (key === 'CONTRACT_ID')
      return 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';
    return undefined;
  },
};

// ---------------------------------------------------------------------------
// Test suite
// ---------------------------------------------------------------------------

describe('ContractService', () => {
  let service: ContractService;

  beforeEach(async () => {
    jest.clearAllMocks();
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        ContractService,
        { provide: ConfigService, useValue: mockConfigService },
      ],
    }).compile();
    service = module.get(ContractService);
  });

  // =========================================================================
  // 1. Transaction envelopes — correct source account and sequence number
  // =========================================================================

  describe('Transaction envelope construction', () => {
    it('buildCreatePoolTransaction — source account matches the creator', () => {
      const xdrStr = service.buildCreatePoolTransaction({
        creator: SOURCE,
        goal: '1000',
        token: TOKEN,
        title: 'My Pool',
        description: 'desc',
      });

      expect(typeof xdr).toBe('string');
      expect(xdr.length).toBeGreaterThan(0);
      // Must be parseable back into a transaction
      expect(() => TransactionBuilder.fromXDR(xdr, NETWORK)).not.toThrow();
    });

    it('buildCreatePoolTransaction — sequence number is 1 (Account seeded with 0)', () => {
      const xdrStr = service.buildCreatePoolTransaction({
        creator: SOURCE,
        goal: '1000',
        token: TOKEN,
        title: 'My Pool',
        description: 'desc',
      });
      const tx = TransactionBuilder.fromXDR(xdrStr, NETWORK) as any;
      // TransactionBuilder increments from 0 → 1 on build()
      expect(tx.sequence).toBe('1');
    });

    it('buildDonateTransaction — source account matches the caller', () => {
      const xdrStr = service.buildDonateTransaction(SOURCE, 5, '250');
      const tx = TransactionBuilder.fromXDR(xdrStr, NETWORK) as any;
      expect(tx.source).toBe(SOURCE);
    });

    it('buildDonateTransaction — sequence number is 1', () => {
      const xdrStr = service.buildDonateTransaction(SOURCE, 5, '250');
      const tx = TransactionBuilder.fromXDR(xdrStr, NETWORK) as any;
      expect(tx.sequence).toBe('1');
    });

    it('buildWithdrawTransaction — source account matches the caller', () => {
      const xdrStr = service.buildWithdrawTransaction(SOURCE, 3, TOKEN);
      const tx = TransactionBuilder.fromXDR(xdrStr, NETWORK) as any;
      expect(tx.source).toBe(SOURCE);
    });

    it('buildClosePoolTransaction — source account matches the caller', () => {
      const xdrStr = service.buildClosePoolTransaction(SOURCE, 7);
      const tx = TransactionBuilder.fromXDR(xdrStr, NETWORK) as any;
      expect(tx.source).toBe(SOURCE);
    });

    it('all build methods produce parseable XDR', () => {
      const cases = [
        () =>
          service.buildCreatePoolTransaction({
            creator: SOURCE,
            goal: '500',
            token: TOKEN,
            title: 'T',
            description: 'D',
          }),
        () => service.buildDonateTransaction(SOURCE, 1, '100'),
        () => service.buildWithdrawTransaction(SOURCE, 1, TOKEN),
        () => service.buildClosePoolTransaction(SOURCE, 1),
      ];
      for (const build of cases) {
        expect(() => TransactionBuilder.fromXDR(build(), NETWORK)).not.toThrow();
      }
    });
  });

  // =========================================================================
  // 2. Contract invocation parameters are correctly encoded
  // =========================================================================

  describe('Contract invocation parameter encoding', () => {
    it('buildCreatePoolTransaction encodes goal as i128', () => {
      const xdrStr = service.buildCreatePoolTransaction({
        creator: SOURCE,
        goal: '9999',
        token: TOKEN,
        title: 'Pool',
        description: 'desc',
      });
      const op = decodeFirstOp(xdrStr);
      const args: xdr.ScVal[] = op.func?.invokeContract?.()?.args?.() ?? [];
      // arg[1] is the goal
      expect(args[1].switch().name).toBe('scvI128');
    });

    it('buildCreatePoolTransaction encodes title as string', () => {
      const xdrStr = service.buildCreatePoolTransaction({
        creator: SOURCE,
        goal: '100',
        token: TOKEN,
        title: 'Hello',
        description: 'desc',
      });
      const op = decodeFirstOp(xdrStr);
      const args: xdr.ScVal[] = op.func?.invokeContract?.()?.args?.() ?? [];
      // arg[3] is the title
      expect(args[3].switch().name).toBe('scvString');
      expect(args[3].str().toString()).toBe('Hello');
    });

    it('buildDonateTransaction encodes poolId as u32', () => {
      const xdrStr = service.buildDonateTransaction(SOURCE, 42, '100');
      const op = decodeFirstOp(xdrStr);
      const args: xdr.ScVal[] = op.func?.invokeContract?.()?.args?.() ?? [];
      expect(args[0].switch().name).toBe('scvU32');
      expect(args[0].u32()).toBe(42);
    });

    it('buildDonateTransaction encodes amount as i128', () => {
      const xdrStr = service.buildDonateTransaction(SOURCE, 1, '777');
      const op = decodeFirstOp(xdrStr);
      const args: xdr.ScVal[] = op.func?.invokeContract?.()?.args?.() ?? [];
      expect(args[1].switch().name).toBe('scvI128');
    });

    it('buildWithdrawTransaction encodes poolId as u32 and token as address', () => {
      const xdrStr = service.buildWithdrawTransaction(SOURCE, 9, TOKEN);
      const op = decodeFirstOp(xdrStr);
      const args: xdr.ScVal[] = op.func?.invokeContract?.()?.args?.() ?? [];
      expect(args[0].switch().name).toBe('scvU32');
      expect(args[0].u32()).toBe(9);
      expect(args[1].switch().name).toBe('scvAddress');
    });

    it('buildClosePoolTransaction encodes poolId as u32 and caller as address', () => {
      const xdrStr = service.buildClosePoolTransaction(SOURCE, 11);
      const op = decodeFirstOp(xdrStr);
      const args: xdr.ScVal[] = op.func?.invokeContract?.()?.args?.() ?? [];
      expect(args[0].switch().name).toBe('scvU32');
      expect(args[0].u32()).toBe(11);
      expect(args[1].switch().name).toBe('scvAddress');
    });
  });

  describe('buildTransaction', () => {
    it('builds a valid transaction from a contract operation', () => {
      const operation = service['contract'].call(
        'close_pool',
        nativeToScVal(1, { type: 'u32' }),
        nativeToScVal(SOURCE, { type: 'address' }),
      );

      const xdr = service['buildTransaction'](SOURCE, operation);
      expect(() => TransactionBuilder.fromXDR(xdr, NETWORK)).not.toThrow();
    });
  });

  describe('submitSignedXdr', () => {
    it('throws StellarError when given invalid XDR', async () => {
      await expect(
        service.submitSignedXdr('not-valid-xdr'),
      ).rejects.toBeInstanceOf(StellarError);
    });

    it('returns the transaction hash on success', async () => {
      mockRpcServer.sendTransaction.mockResolvedValueOnce({
        hash: 'abc123txhash',
        status: 'PENDING',
      });
      const validXdr = service.buildDonateTransaction(SOURCE, 1, '100');
      const hash = await service.submitSignedXdr(validXdr);
      expect(hash).toBe('abc123txhash');
    });
  });

  // =========================================================================
  // 4. Retries / timeouts handled per configured policy
  // =========================================================================

  describe('Timeout and error handling policy', () => {
    it('getContributionOnChain returns 0n when getAccount throws (network timeout)', async () => {
      mockRpcServer.getAccount.mockRejectedValueOnce(new Error('timeout'));
      const result = await service.getContributionOnChain(1, DONOR);
      expect(result).toBe(0n);
    });

    it('getContributionOnChain returns 0n when simulateTransaction returns error shape', async () => {
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(fakeSimError());
      const result = await service.getContributionOnChain(1, DONOR);
      expect(result).toBe(0n);
    });

    it('getContributionOnChain returns 0n when retval is missing', async () => {
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce({
        result: { retval: null, auth: [] },
        latestLedger: 1,
      });
      const result = await service.getContributionOnChain(1, DONOR);
      expect(result).toBe(0n);
    });

    it('getTotalRaisedOnChain returns 0n when getAccount throws', async () => {
      mockRpcServer.getAccount.mockRejectedValueOnce(
        new Error('connection refused'),
      );
      const result = await service.getTotalRaisedOnChain(1);
      expect(result).toBe(0n);
    });

    it('getTotalRaisedOnChain returns 0n when simulate returns error shape', async () => {
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(fakeSimError());
      const result = await service.getTotalRaisedOnChain(1);
      expect(result).toBe(0n);
    });

    it('decodes scvI128 return value correctly', async () => {
      // Mock RPC getAccount to return a valid Account
      const mockAccount = new Account(Keypair.random().publicKey(), '0');
      jest.spyOn(service['rpcServer'], 'getAccount').mockResolvedValue(mockAccount as any);

      // Mock simulateTransaction to return a result with retval
      const mockResult = {
        result: {
          retval: {
            toXDR: () => Buffer.from([0]),
          },
        },
      };
      jest.spyOn(service['rpcServer'], 'simulateTransaction').mockResolvedValue(mockResult as any);

      // Mock xdr.ScVal.fromXDR to return an object indicating scvI128 with parts
      jest.spyOn(xdr.ScVal, 'fromXDR').mockReturnValue({
        switch: () => ({ name: 'scvI128' }),
        i128: () => ({
          hi: () => ({ toString: () => '0' }),
          lo: () => ({ toString: () => '123' }),
        }),
        u128: undefined,
      } as any);

      const donor = Keypair.random().publicKey();
      const contribution = await service.getContributionOnChain(1, donor);
      expect(contribution).toBe(123n);
    });
  });

    it('getDonorCountOnChain returns 0 when getAccount throws', async () => {
      mockRpcServer.getAccount.mockRejectedValueOnce(new Error('timeout'));
      const result = await service.getDonorCountOnChain(1);
      expect(result).toBe(0);
    });

    it('getDonorCountOnChain returns 0 when simulate returns error shape', async () => {
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(fakeSimError());
      const result = await service.getDonorCountOnChain(1);
      expect(result).toBe(0);
    });

    it('getPoolOnChain returns null when getAccount throws', async () => {
      mockRpcServer.getAccount.mockRejectedValueOnce(new Error('timeout'));
      const result = await service.getPoolOnChain(1);
      expect(result).toBeNull();
    });

    it('getPoolOnChain returns null when simulate returns error shape', async () => {
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(fakeSimError());
      const result = await service.getPoolOnChain(1);
      expect(result).toBeNull();
    });
  });

  // =========================================================================
  // 5. Service methods with mocked Soroban RPC responses
  // =========================================================================

  describe('getContributionOnChain — mocked RPC', () => {
    it('decodes a non-zero i128 return value', async () => {
      const scVal = nativeToScVal(BigInt(500), { type: 'i128' });
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(scVal),
      );
      const result = await service.getContributionOnChain(1, DONOR);
      expect(result).toBe(500n);
    });

    it('returns 0n when retval switch is neither i128 nor u128', async () => {
      const scVal = nativeToScVal(42, { type: 'u32' });
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(scVal),
      );
      const result = await service.getContributionOnChain(1, DONOR);
      expect(result).toBe(0n);
    });

    it('returns the total raised from the simulation result', async () => {
      const mockAccount = new Account(Keypair.random().publicKey(), '0');
      jest.spyOn(service['rpcServer'], 'getAccount').mockResolvedValue(mockAccount as any);
      jest.spyOn(service['rpcServer'], 'simulateTransaction').mockResolvedValue({
        result: { retval: nativeToScVal(5000n, { type: 'i128' }) },
      } as any);

      const result = await service.getTotalRaisedOnChain(1);
      expect(result).toBe(5000n);
    });
  });

  describe('getTotalRaisedOnChain — mocked RPC', () => {
    it('returns a bigint from a simulate response', async () => {
      const scVal = nativeToScVal(BigInt(12345), { type: 'i128' });
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(scVal),
      );
      const result = await service.getTotalRaisedOnChain(1);
      expect(result).toBe(12345n);
    });
  });

  describe('getDonorCountOnChain — mocked RPC', () => {
    it('returns a number from a simulate response', async () => {
      const scVal = nativeToScVal(7, { type: 'u32' });
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(scVal),
      );
      const result = await service.getDonorCountOnChain(1);
      expect(result).toBe(7);
    });

    it('returns the donor count from the simulation result', async () => {
      const mockAccount = new Account(Keypair.random().publicKey(), '0');
      jest.spyOn(service['rpcServer'], 'getAccount').mockResolvedValue(mockAccount as any);
      jest.spyOn(service['rpcServer'], 'simulateTransaction').mockResolvedValue({
        result: { retval: nativeToScVal(7, { type: 'u32' }) },
      } as any);

      const result = await service.getDonorCountOnChain(1);
      expect(result).toBe(7);
    });
  });

  describe('getPoolOnChain — mocked RPC', () => {
    it('returns null when retval is missing', async () => {
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce({
        result: { retval: null, auth: [] },
        latestLedger: 1,
      });
      const result = await service.getPoolOnChain(1);
      expect(result).toBeNull();
    });
  });

  // =========================================================================
  // mapError — internal error mapping
  // =========================================================================

  describe('mapError', () => {
    it('maps tx_bad_auth to StellarError with "Bad authentication"', () => {
      const err = service['mapError'](new Error('tx_bad_auth'));
      expect(err).toBeInstanceOf(StellarError);
      expect((err as HttpException).getResponse()).toBe('Bad authentication');
    });

    it('maps op_underfunded to StellarError with "Insufficient balance"', () => {
      const err = service['mapError'](new Error('op_underfunded'));
      expect(err).toBeInstanceOf(StellarError);
      expect((err as HttpException).getResponse()).toBe('Insufficient balance');
    });

    it('maps op_no_source_account to StellarError', () => {
      const err = service['mapError'](new Error('op_no_source_account'));
      expect(err).toBeInstanceOf(StellarError);
      expect((err as HttpException).getResponse()).toBe('Source account does not exist on the network');
    });

    it('maps timeout keyword to StellarError', () => {
      const err = service['mapError'](new Error('timeout'));
      expect(err).toBeInstanceOf(StellarError);
      expect(String((err as HttpException).getResponse())).toMatch(/timed out/i);
    });

    it('wraps unknown errors by message', () => {
      const err = service['mapError'](new Error('some random failure'));
      expect(err).toBeInstanceOf(StellarError);
      expect((err as HttpException).getResponse()).toBe('some random failure');
    });

    it('passes through an existing StellarError unchanged', () => {
      const original = new StellarError('timeout');
      const result = service['mapError'](original);
      expect(result).toBe(original);
    });

    it('handles a non-Error object with a message property', () => {
      const err = service['mapError']({ message: 'op_underfunded' });
      expect(err).toBeInstanceOf(StellarError);
      expect((err as HttpException).getResponse()).toBe('Insufficient balance');
    });

    it('maps op_no_source_account to NOT_FOUND', () => {
      const error = new Error('op_no_source_account');
      const stellarError = service['mapError'](error);
      expect(stellarError).toBeInstanceOf(StellarError);
      expect(stellarError.getStatus()).toBe(HttpStatus.NOT_FOUND);
      expect(stellarError.message).toBe(
        'Source account does not exist on the network',
      );
    });

    it('maps timeout to REQUEST_TIMEOUT', () => {
      const error = new Error('timeout');
      const stellarError = service['mapError'](error);
      expect(stellarError).toBeInstanceOf(StellarError);
      expect(stellarError.getStatus()).toBe(HttpStatus.REQUEST_TIMEOUT);
    });

    it('falls back to INTERNAL_SERVER_ERROR for unrecognized messages', () => {
      const error = new Error('some unrecognized stellar failure');
      const stellarError = service['mapError'](error);
      expect(stellarError).toBeInstanceOf(StellarError);
      expect(stellarError.getStatus()).toBe(HttpStatus.INTERNAL_SERVER_ERROR);
      expect(stellarError.message).toBe('some unrecognized stellar failure');
    });
  });
});
