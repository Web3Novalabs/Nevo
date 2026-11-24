import { HttpStatus } from '@nestjs/common';
import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import {
  TransactionBuilder,
  Networks,
  Keypair,
  nativeToScVal,
  scValToNative,
  Account,
  Address,
  xdr,
  rpc as StellarRpc,
} from '@stellar/stellar-sdk';
import { ContractService } from './contract.service.js';
import { StellarError } from './stellar.error.js';
import { StellarConfig } from './stellar.config.js';

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
  };
}

/** Build a SimulateTransactionResponse that carries a return value */
function fakeSimResult(
  scVal: xdr.ScVal,
): StellarRpc.Api.SimulateTransactionSuccessResponse {
  return {
    result: { retval: scVal, auth: [] },
    latestLedger: 1,
    minResourceFee: '100',
    transactionData: '',
    cost: { cpuInsns: '0', memBytes: '0' },
    events: [],
    stateChanges: [],
  };
}

/** Build a SimulateTransactionResponse that carries an error */
function fakeSimError(): StellarRpc.Api.SimulateTransactionErrorResponse {
  return {
    error: 'contract error',
    latestLedger: 1,
    events: [],
  };
}

// ---------------------------------------------------------------------------
// Mock RPC server
// ---------------------------------------------------------------------------

const mockRpcServer = {
  sendTransaction: jest.fn(),
  getTransaction: jest.fn(),
  getAccount: jest.fn(),
  getLedgerEntries: jest.fn(),
  simulateTransaction: jest.fn(),
  prepareTransaction: jest.fn(),
};

// Patch the constructor so every `new StellarRpc.Server(...)` returns our mock
jest.mock('@stellar/stellar-sdk', () => {
  const actual = jest.requireActual('@stellar/stellar-sdk');
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
    // Default RPC behaviour: a funded account and a pass-through prepare
    mockRpcServer.getAccount.mockImplementation(async (publicKey: string) =>
      fakeAccount(publicKey),
    );
    mockRpcServer.prepareTransaction.mockImplementation(async (tx: any) => tx);
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        ContractService,
        StellarConfig,
        { provide: ConfigService, useValue: mockConfigService },
      ],
    }).compile();
    service = module.get(ContractService);
  });

  // =========================================================================
  // 1. Transaction envelopes — correct source account and sequence number
  // =========================================================================

  it('reads application deadlines from the persistent on-chain pool records', async () => {
    const contract = Address.fromString(
      mockConfigService.get('CONTRACT_ID')!,
    ).toScAddress();
    const key = xdr.ScVal.scvU32(42);
    const durability = xdr.ContractDataDurability.persistent();
    const deadline = 18446744073709551615n;

    mockRpcServer.getLedgerEntries.mockResolvedValue({
      entries: [
        {
          val: xdr.LedgerEntryData.contractData(
            new xdr.ContractDataEntry({
              ext: new xdr.ExtensionPoint(0),
              contract,
              key,
              durability,
              val: xdr.ScVal.scvMap([
                new xdr.ScMapEntry({
                  key: xdr.ScVal.scvSymbol('application_deadline'),
                  val: nativeToScVal(deadline, { type: 'u64' }),
                }),
              ]),
            }),
          ),
        },
      ],
    });

    await expect(
      service.getPoolApplicationDeadlines(['42', '4294967296', 'bad']),
    ).resolves.toEqual(new Map([['42', deadline.toString()]]));
    expect(mockRpcServer.getLedgerEntries).toHaveBeenCalledWith(
      xdr.LedgerKey.contractData(
        new xdr.LedgerKeyContractData({ contract, key, durability }),
      ),
    );
  });

  describe('Transaction envelope construction', () => {
    it('buildCreatePoolTransaction — source account matches the creator', async () => {
      const xdrStr = await service.buildCreatePoolTransaction({
        creator: SOURCE,
        goal: '1000',
        token: TOKEN,
        title: 'My Pool',
        description: 'desc',
        applicationDeadline: '2000000000',
      });

      expect(typeof xdrStr).toBe('string');
      expect(xdrStr.length).toBeGreaterThan(0);
      // Must be parseable back into a transaction
      expect(() => TransactionBuilder.fromXDR(xdrStr, NETWORK)).not.toThrow();
    });

    it('buildCreatePoolTransaction — sequence number comes from the loaded account', async () => {
      const xdrStr = await service.buildCreatePoolTransaction({
        creator: SOURCE,
        goal: '1000',
        token: TOKEN,
        title: 'My Pool',
        description: 'desc',
        applicationDeadline: '2000000000',
      });
      const tx = TransactionBuilder.fromXDR(xdrStr, NETWORK) as any;
      // Account seeded with sequence 100 → builder increments to 101
      expect(tx.sequence).toBe('101');
    });

    it('buildDonateTransaction — source account matches the caller', async () => {
      const xdrStr = await service.buildDonateTransaction(SOURCE, 5, '250');
      const tx = TransactionBuilder.fromXDR(xdrStr, NETWORK) as any;
      expect(tx.source).toBe(SOURCE);
    });

    it('buildDonateTransaction — sequence number comes from the loaded account', async () => {
      const xdrStr = await service.buildDonateTransaction(SOURCE, 5, '250');
      const tx = TransactionBuilder.fromXDR(xdrStr, NETWORK) as any;
      expect(tx.sequence).toBe('101');
    });

    it('buildWithdrawTransaction — source account matches the caller', async () => {
      const xdrStr = await service.buildWithdrawTransaction(SOURCE, 3, TOKEN);
      const tx = TransactionBuilder.fromXDR(xdrStr, NETWORK) as any;
      expect(tx.source).toBe(SOURCE);
    });

    it('buildClosePoolTransaction — source account matches the caller', async () => {
      const xdrStr = await service.buildClosePoolTransaction(SOURCE, 7);
      const tx = TransactionBuilder.fromXDR(xdrStr, NETWORK) as any;
      expect(tx.source).toBe(SOURCE);
    });

    it('all build methods produce parseable XDR', async () => {
      const cases = [
        () =>
          service.buildCreatePoolTransaction({
            creator: SOURCE,
            goal: '500',
            token: TOKEN,
            title: 'T',
            description: 'D',
            applicationDeadline: '2000000000',
          }),
        () => service.buildDonateTransaction(SOURCE, 1, '100'),
        () => service.buildWithdrawTransaction(SOURCE, 1, TOKEN),
        () => service.buildClosePoolTransaction(SOURCE, 1),
      ];
      for (const build of cases) {
        expect(() =>
          TransactionBuilder.fromXDR(await build(), NETWORK),
        ).not.toThrow();
      }
    });
  });

  // =========================================================================
  // 2. Contract invocation parameters are correctly encoded
  // =========================================================================

  describe('Contract invocation parameter encoding', () => {
    it('buildCreatePoolTransaction invokes create_pool with (creator, title, description, goal, application_deadline)', async () => {
      const xdrStr = await service.buildCreatePoolTransaction({
        creator: SOURCE,
        goal: '9999',
        token: TOKEN,
        title: 'Pool',
        description: 'desc',
        applicationDeadline: '2000000000',
      });
      const op = decodeFirstOp(xdrStr);
      const invoke = op.func?.invokeContract?.();
      expect(invoke?.functionName?.().toString()).toBe('create_pool');
      const args: xdr.ScVal[] = invoke?.args?.() ?? [];
      expect(args).toHaveLength(5);
      // arg[0] creator: Address
      expect(args[0].switch().name).toBe('scvAddress');
      expect(scValToNative(args[0])).toBe(SOURCE);
      // arg[1] title: String
      expect(args[1].switch().name).toBe('scvString');
      expect(args[1].str().toString()).toBe('Pool');
      // arg[2] description: String
      expect(args[2].switch().name).toBe('scvString');
      expect(args[2].str().toString()).toBe('desc');
      // arg[3] goal: u128
      expect(args[3].switch().name).toBe('scvU128');
      expect(scValToNative(args[3])).toBe(9999n);
      // arg[4] application_deadline: u64
      expect(args[4].switch().name).toBe('scvU64');
      expect(scValToNative(args[4])).toBe(2000000000n);
    });

    it('buildDonateTransaction invokes donate_with_token with (pool_id, donor, token, amount)', async () => {
      const xdrStr = await service.buildDonateTransaction(SOURCE, 42, '777');
      const args: xdr.ScVal[] = op.func?.invokeContract?.()?.args?.() ?? [];
      // arg[1] is the goal
      expect(args[1].switch().name).toBe('scvI128');
      expect(args[1].switch().name).not.toBe('scvU128');
      expect(scValToNative(args[1])).toBe(9999n);
    });

    it('buildCreatePoolTransaction rejects a goal above i128::MAX with a typed error', () => {
      const tooLarge = (BigInt('1') << BigInt(127)).toString();
      expect(() =>
        service.buildCreatePoolTransaction({
          creator: SOURCE,
          goal: tooLarge,
          token: TOKEN,
          title: 'Pool',
          description: 'desc',
        }),
      ).toThrow(StellarError);
    });

    it('buildCreatePoolTransaction rejects a negative goal with a typed error', () => {
      expect(() =>
        service.buildCreatePoolTransaction({
          creator: SOURCE,
          goal: '-1',
          token: TOKEN,
          title: 'Pool',
          description: 'desc',
        }),
      ).toThrow(StellarError);
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
      const invoke = op.func?.invokeContract?.();
      expect(invoke?.functionName?.().toString()).toBe('donate_with_token');
      const args: xdr.ScVal[] = invoke?.args?.() ?? [];
      expect(args).toHaveLength(4);
      // arg[0] pool_id: u32
      expect(args[0].switch().name).toBe('scvU32');
      expect(args[0].u32()).toBe(42);
      // arg[1] donor: Address
      expect(args[1].switch().name).toBe('scvAddress');
      expect(scValToNative(args[1])).toBe(SOURCE);
      // arg[2] token: Address
      expect(args[2].switch().name).toBe('scvAddress');
      // arg[3] amount: i128
      expect(args[3].switch().name).toBe('scvI128');
      expect(scValToNative(args[3])).toBe(777n);
    });

    it('buildWithdrawTransaction encodes poolId as u32 and token as address', async () => {
      const xdrStr = await service.buildWithdrawTransaction(SOURCE, 9, TOKEN);
    it('buildDonateTransaction encodes amount as i128', () => {
      const xdrStr = service.buildDonateTransaction(SOURCE, 1, '777');
      const op = decodeFirstOp(xdrStr);
      const args: xdr.ScVal[] = op.func?.invokeContract?.()?.args?.() ?? [];
      expect(args[1].switch().name).toBe('scvI128');
      expect(scValToNative(args[1])).toBe(777n);
    });

    it('buildDonateTransaction rejects an amount above i128::MAX with a typed error', () => {
      const tooLarge = (BigInt('1') << BigInt(127)).toString();
      expect(() =>
        service.buildDonateTransaction(SOURCE, 1, tooLarge),
      ).toThrow(StellarError);
    });

    it('buildDonateTransaction rejects a negative amount with a typed error', () => {
      expect(() => service.buildDonateTransaction(SOURCE, 1, '-5')).toThrow(
        StellarError,
      );
    });

    it('buildWithdrawTransaction encodes poolId as u32 and token as address', () => {
      const xdrStr = service.buildWithdrawTransaction(SOURCE, 9, TOKEN);
      const op = decodeFirstOp(xdrStr);
      const args: xdr.ScVal[] = op.func?.invokeContract?.()?.args?.() ?? [];
      expect(args[0].switch().name).toBe('scvU32');
      expect(args[0].u32()).toBe(9);
      expect(args[1].switch().name).toBe('scvAddress');
    });

    it('buildClosePoolTransaction encodes poolId as u32 and passes no extra address argument', () => {
      const xdrStr = service.buildClosePoolTransaction(SOURCE, 11);
    it('buildClosePoolTransaction encodes poolId as u32 and caller as address', async () => {
      const xdrStr = await service.buildClosePoolTransaction(SOURCE, 11);
      const op = decodeFirstOp(xdrStr);
      const args: xdr.ScVal[] = op.func?.invokeContract?.()?.args?.() ?? [];
      expect(args).toHaveLength(1);
      expect(args[0].switch().name).toBe('scvU32');
      expect(args[0].u32()).toBe(11);
    });
  });

  describe('buildTransaction', () => {
    it('loads the source account and returns the prepared transaction XDR', async () => {
      const operation = service['contract'].call(
        'close_pool',
        nativeToScVal(1, { type: 'u32' }),
      );

      const xdr = await service['buildTransaction'](SOURCE, operation);
      expect(mockRpcServer.getAccount).toHaveBeenCalledWith(SOURCE);
      expect(mockRpcServer.prepareTransaction).toHaveBeenCalledTimes(1);
      expect(() => TransactionBuilder.fromXDR(xdr, NETWORK)).not.toThrow();
    });

    it('returns exactly the XDR produced by prepareTransaction', async () => {
      const operation = service['contract'].call(
        'close_pool',
        nativeToScVal(1, { type: 'u32' }),
        nativeToScVal(SOURCE, { type: 'address' }),
      );
      const prepared = new TransactionBuilder(fakeAccount(SOURCE) as any, {
        fee: '100',
        networkPassphrase: NETWORK,
      })
        .addOperation(operation)
        .setTimeout(30)
        .build();
      mockRpcServer.prepareTransaction.mockResolvedValueOnce(prepared);

      const xdr = await service['buildTransaction'](SOURCE, operation);
      expect(xdr).toBe(prepared.toXDR());
    });

    it('maps a getAccount rejection to a StellarError', async () => {
      mockRpcServer.getAccount.mockRejectedValueOnce(
        new Error('op_no_source_account'),
      );
      const operation = service['contract'].call(
        'close_pool',
        nativeToScVal(1, { type: 'u32' }),
        nativeToScVal(SOURCE, { type: 'address' }),
      );
      await expect(
        service['buildTransaction'](SOURCE, operation),
      ).rejects.toBeInstanceOf(StellarError);
      expect(mockRpcServer.prepareTransaction).not.toHaveBeenCalled();
    });

    it('maps a prepareTransaction rejection to a StellarError', async () => {
      mockRpcServer.prepareTransaction.mockRejectedValueOnce(
        new Error('simulation failed'),
      );
      const operation = service['contract'].call(
        'close_pool',
        nativeToScVal(1, { type: 'u32' }),
        nativeToScVal(SOURCE, { type: 'address' }),
      );
      await expect(
        service['buildTransaction'](SOURCE, operation),
      ).rejects.toBeInstanceOf(StellarError);
    });
  });

  describe('submitSignedXdr', () => {
    it('throws StellarError when given invalid XDR', async () => {
      await expect(
        service.submitSignedXdr('not-valid-xdr'),
      ).rejects.toBeInstanceOf(StellarError);
    });

    it('returns completed with the decoded return value when the tx is confirmed SUCCESS', async () => {
      mockRpcServer.sendTransaction.mockResolvedValueOnce({
        hash: 'abc123txhash',
        status: 'PENDING',
        latestLedger: 0,
        latestLedgerCloseTime: 0,
      });
      mockRpcServer.getTransaction.mockResolvedValueOnce({
        status: StellarRpc.Api.GetTransactionStatus.SUCCESS,
        txHash: 'abc123txhash',
        ledger: 1,
        createdAt: 0,
        applicationOrder: 1,
        feeBump: false,
        envelopeXdr: 'AAAA',
        resultXdr: 'AAAA',
        resultMetaXdr: 'AAAA',
        returnValue: xdr.ScVal.scvU32(7),
        events: { transactionEventsXdr: [], contractEventsXdr: [] },
      } as StellarRpc.Api.GetSuccessfulTransactionResponse);

      const validXdr = await service.buildDonateTransaction(SOURCE, 1, '100');
      const result = await service.submitSignedXdr(validXdr);

      expect(result.txHash).toBe('abc123txhash');
      expect(result.status).toBe('completed');
      expect(result.returnValue).toEqual(scValToNative(xdr.ScVal.scvU32(7)));
    });

    it('returns failed when the tx is confirmed FAILED', async () => {
      mockRpcServer.sendTransaction.mockResolvedValueOnce({
        hash: 'failtx',
        status: 'PENDING',
        latestLedger: 0,
        latestLedgerCloseTime: 0,
      });
      mockRpcServer.getTransaction.mockResolvedValueOnce({
        status: StellarRpc.Api.GetTransactionStatus.FAILED,
        txHash: 'failtx',
        ledger: 1,
        createdAt: 0,
        applicationOrder: 1,
        feeBump: false,
        envelopeXdr: 'AAAA',
        resultXdr: 'AAAA',
        resultMetaXdr: 'AAAA',
        events: { transactionEventsXdr: [], contractEventsXdr: [] },
      });

      const validXdr = await service.buildDonateTransaction(SOURCE, 1, '100');
      const result = await service.submitSignedXdr(validXdr);
      expect(result).toEqual({ txHash: 'failtx', status: 'failed' });
    });

    it('throws a mapped StellarError when sendTransaction returns ERROR', async () => {
      mockRpcServer.sendTransaction.mockResolvedValueOnce({
        hash: 'errtx',
        status: 'ERROR',
        latestLedger: 0,
        latestLedgerCloseTime: 0,
      });

      const validXdr = await service.buildDonateTransaction(SOURCE, 1, '100');
      await expect(service.submitSignedXdr(validXdr)).rejects.toBeInstanceOf(
        StellarError,
      );
      expect(mockRpcServer.getTransaction).not.toHaveBeenCalled();
    });

    it('throws a mapped StellarError when sendTransaction returns TRY_AGAIN_LATER', async () => {
      mockRpcServer.sendTransaction.mockResolvedValueOnce({
        hash: 'retrytx',
        status: 'TRY_AGAIN_LATER',
        latestLedger: 0,
        latestLedgerCloseTime: 0,
      });

      const validXdr = await service.buildDonateTransaction(SOURCE, 1, '100');
      await expect(service.submitSignedXdr(validXdr)).rejects.toBeInstanceOf(
        StellarError,
      );
      expect(mockRpcServer.getTransaction).not.toHaveBeenCalled();
    });

    it('returns pending when no confirmation arrives within the timeout', async () => {
      service['confirmTimeoutMs'] = 0;
      mockRpcServer.sendTransaction.mockResolvedValueOnce({
        hash: 'pendtx',
        status: 'PENDING',
        latestLedger: 0,
        latestLedgerCloseTime: 0,
      });
      mockRpcServer.getTransaction.mockResolvedValue({
        status: StellarRpc.Api.GetTransactionStatus.NOT_FOUND,
        txHash: 'pendtx',
        latestLedger: 0,
        latestLedgerCloseTime: 0,
        oldestLedger: 0,
        oldestLedgerCloseTime: 0,
      });

      const validXdr = await service.buildDonateTransaction(SOURCE, 1, '100');
      const result = await service.submitSignedXdr(validXdr);
      expect(result).toEqual({ txHash: 'pendtx', status: 'pending' });
    });
  });

  // =========================================================================
  // 4. Retries / timeouts handled per configured policy
  // =========================================================================

  describe('Timeout and error handling policy', () => {
    it('getContributionOnChain does not call getAccount (simulates from in-memory account)', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(nativeToScVal(BigInt(500), { type: 'i128' })),
      );
      const result = await service.getContributionOnChain(1, DONOR);
      expect(result).toBe(500n);
      expect(mockRpcServer.getAccount).not.toHaveBeenCalled();
    });

    it('getContributionOnChain simulates with an in-memory Account', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(nativeToScVal(BigInt(1), { type: 'i128' })),
      );
      await service.getContributionOnChain(1, DONOR);
      const tx = mockRpcServer.simulateTransaction.mock.calls[0][0];
      expect(tx.source).toBeInstanceOf(Account);
      expect(tx.source.sequenceNumber()).toBe('0');
    });

    it('getContributionOnChain throws StellarError on RPC failure (not 0n)', async () => {
      mockRpcServer.simulateTransaction.mockRejectedValueOnce(
        new Error('timeout'),
      );
      await expect(
        service.getContributionOnChain(1, DONOR),
      ).rejects.toBeInstanceOf(StellarError);
    });

    it('getContributionOnChain throws StellarError when simulateTransaction returns error shape', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(fakeSimError());
      await expect(
        service.getContributionOnChain(1, DONOR),
      ).rejects.toBeInstanceOf(StellarError);
    });

    it('getContributionOnChain returns 0n when retval is missing', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce({
        result: { retval: null, auth: [] },
        latestLedger: 1,
      });
      const result = await service.getContributionOnChain(1, DONOR);
      expect(result).toBe(0n);
    });

    it('getTotalRaisedOnChain does not call getAccount', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(nativeToScVal(BigInt(12345), { type: 'i128' })),
      );
      const result = await service.getTotalRaisedOnChain(1);
      expect(result).toBe(12345n);
      expect(mockRpcServer.getAccount).not.toHaveBeenCalled();
    });

    it('getTotalRaisedOnChain throws StellarError on RPC failure (not 0n)', async () => {
      mockRpcServer.simulateTransaction.mockRejectedValueOnce(
        new Error('connection refused'),
      );
      await expect(service.getTotalRaisedOnChain(1)).rejects.toBeInstanceOf(
        StellarError,
      );
    });

    it('getTotalRaisedOnChain throws StellarError when simulate returns error shape', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(fakeSimError());
      await expect(service.getTotalRaisedOnChain(1)).rejects.toBeInstanceOf(
        StellarError,
      );
    });

    it('decodes scvI128 return value correctly', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(nativeToScVal(BigInt(123), { type: 'i128' })),
      );

      const donor = Keypair.random().publicKey();
      const contribution = await service.getContributionOnChain(1, donor);
      expect(contribution).toBe(123n);
    });

    it('getDonorCountOnChain does not call getAccount', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(nativeToScVal(7, { type: 'u32' })),
      );
      const result = await service.getDonorCountOnChain(1);
      expect(result).toBe(7);
      expect(mockRpcServer.getAccount).not.toHaveBeenCalled();
    });

    it('getDonorCountOnChain throws StellarError on RPC failure (not 0)', async () => {
      mockRpcServer.simulateTransaction.mockRejectedValueOnce(
        new Error('timeout'),
      );
      await expect(service.getDonorCountOnChain(1)).rejects.toBeInstanceOf(
        StellarError,
      );
    });

    it('getDonorCountOnChain throws StellarError when simulate returns error shape', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(fakeSimError());
      await expect(service.getDonorCountOnChain(1)).rejects.toBeInstanceOf(
        StellarError,
      );
    });

    it('getPoolOnChain does not call getAccount', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(nativeToScVal({ id: 1 }, { type: 'map' })),
      );
      await service.getPoolOnChain(1);
      expect(mockRpcServer.getAccount).not.toHaveBeenCalled();
    });

    it('getPoolOnChain throws StellarError on RPC failure (not null)', async () => {
      mockRpcServer.simulateTransaction.mockRejectedValueOnce(
        new Error('timeout'),
      );
      await expect(service.getPoolOnChain(1)).rejects.toBeInstanceOf(
        StellarError,
      );
    });

    it('getPoolOnChain throws StellarError when simulate returns error shape', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(fakeSimError());
      await expect(service.getPoolOnChain(1)).rejects.toBeInstanceOf(
        StellarError,
      );
    });
  });

  // =========================================================================
  // 5. Service methods with mocked Soroban RPC responses
  // =========================================================================

  describe('getContributionOnChain — mocked RPC', () => {
    it('decodes a non-zero i128 return value', async () => {
      const scVal = nativeToScVal(BigInt(500), { type: 'i128' });
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(scVal),
      );
      const result = await service.getContributionOnChain(1, DONOR);
      expect(result).toBe(500n);
    });

    it('returns 0n when retval switch is neither i128 nor u128', async () => {
      const scVal = nativeToScVal(42, { type: 'u32' });
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(scVal),
      );
      const result = await service.getContributionOnChain(1, DONOR);
      expect(result).toBe(0n);
    });

    it('returns the total raised from the simulation result', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(nativeToScVal(5000n, { type: 'i128' })),
      );

      const result = await service.getTotalRaisedOnChain(1);
      expect(result).toBe(5000n);
    });
  });

  describe('getTotalRaisedOnChain — mocked RPC', () => {
    it('returns a bigint from a simulate response', async () => {
      const scVal = nativeToScVal(BigInt(12345), { type: 'i128' });
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
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(scVal),
      );
      const result = await service.getDonorCountOnChain(1);
      expect(result).toBe(7);
    });

    it('returns the donor count from the simulation result', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(nativeToScVal(7, { type: 'u32' })),
      );

      const result = await service.getDonorCountOnChain(1);
      expect(result).toBe(7);
    });
  });

  describe('getPoolOnChain — mocked RPC', () => {
    it('returns null when retval is missing', async () => {
      mockRpcServer.simulateTransaction.mockResolvedValueOnce({
        result: { retval: null, auth: [] },
        latestLedger: 1,
      });
      const result = await service.getPoolOnChain(1);
      expect(result).toBeNull();
    });
  });

  // =========================================================================
  // Application status — enum-based decoding
  // =========================================================================

  describe('getApplicationStatusOnChain — mocked RPC', () => {
    it('decodes the Pending enum variant', async () => {
      const scVal = nativeToScVal('Pending', { type: 'symbol' });
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(scVal),
      );
      const result = await service.getApplicationStatusOnChain(1, DONOR);
      expect(result).toBe('Pending');
    });

    it('decodes the Approved enum variant', async () => {
      const scVal = nativeToScVal('Approved', { type: 'symbol' });
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(scVal),
      );
      const result = await service.getApplicationStatusOnChain(1, DONOR);
      expect(result).toBe('Approved');
    });

    it('decodes the Rejected enum variant', async () => {
      const scVal = nativeToScVal('Rejected', { type: 'symbol' });
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(scVal),
      );
      const result = await service.getApplicationStatusOnChain(1, DONOR);
      expect(result).toBe('Rejected');
    });

    it('returns null when retval is missing', async () => {
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce({
        result: { retval: null, auth: [] },
        latestLedger: 1,
      });
      const result = await service.getApplicationStatusOnChain(1, DONOR);
      expect(result).toBeNull();
    });

    it('returns null when getAccount throws', async () => {
      mockRpcServer.getAccount.mockRejectedValueOnce(new Error('timeout'));
      const result = await service.getApplicationStatusOnChain(1, DONOR);
      expect(result).toBeNull();
    });

    it('returns null when simulate returns error shape', async () => {
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(fakeSimError());
      const result = await service.getApplicationStatusOnChain(1, DONOR);
      expect(result).toBeNull();
    });

    it('returns null for an unrecognized enum variant', async () => {
      const scVal = nativeToScVal('Unknown', { type: 'symbol' });
      mockRpcServer.getAccount.mockResolvedValue(fakeAccount(SOURCE));
      mockRpcServer.simulateTransaction.mockResolvedValueOnce(
        fakeSimResult(scVal),
      );
      const result = await service.getApplicationStatusOnChain(1, DONOR);
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
      expect((err as HttpException).getResponse()).toBe(
        'Source account does not exist on the network',
      );
    });

    it('maps timeout keyword to StellarError', () => {
      const err = service['mapError'](new Error('timeout'));
      expect(err).toBeInstanceOf(StellarError);
      expect(String((err as HttpException).getResponse())).toMatch(
        /timed out/i,
      );
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
