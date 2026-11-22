import { Test, TestingModule } from '@nestjs/testing';
import { SyncService, HorizonContractEvent } from './sync.service';
import { PoolsService } from '../pools/pools.service';
import { DonationsService } from '../donations/donations.service';
import { getRepositoryToken } from '@nestjs/typeorm';
import {
  ADVISORY_LOCK_KEYS,
  AdvisoryLockService,
} from '../common/advisory-lock.service';
import { SyncState } from './sync-state.entity';
import { StellarConfig } from '../contract/stellar.config';
import { Pool, PoolStatus } from '../pools/pool.entity';
import {
  Contract,
  Keypair,
  nativeToScVal,
  rpc as StellarRpc,
  xdr,
} from '@stellar/stellar-sdk';

const STELLAR_CONTRACT_ID =
  'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';

// Valid ed25519 strkeys — nativeToScVal rejects fake addresses.
const CREATOR_WALLET = Keypair.random().publicKey();
const DONOR_WALLET = Keypair.random().publicKey();
const CLOSER_WALLET = Keypair.random().publicKey();

const healthResponse = {
  latestLedger: 1000,
  ledgerRetentionWindow: 100,
  oldestLedger: 1000,
  status: 'healthy' as const,
};

/** Build a minimal `Api.EventResponse` for tests. */
function eventResponse(opts: {
  topic: xdr.ScVal[];
  value: xdr.ScVal;
  txHash?: string;
}): StellarRpc.Api.EventResponse {
  return {
    id: 'evt-fixture',
    type: 'contract',
    ledger: 100,
    ledgerClosedAt: '2026-01-01T00:00:00Z',
    transactionIndex: 0,
    operationIndex: 0,
    inSuccessfulContractCall: true,
    txHash: opts.txHash ?? 'tx-hash-fixture',
    contractId: new Contract(STELLAR_CONTRACT_ID),
    topic: opts.topic,
    value: opts.value,
  };
}

const poolCreatedEvent = eventResponse({
  topic: [
    nativeToScVal('pool_crtd', { type: 'symbol' }),
    nativeToScVal('42', { type: 'u32' }),
  ],
  value: nativeToScVal([
    nativeToScVal(CREATOR_WALLET, { type: 'address' }),
    '50000',
    'My Pool',
    'Description',
  ]),
  txHash: 'tx-pool-created-001',
});

const donationEvent = eventResponse({
  topic: [
    nativeToScVal('donation', { type: 'symbol' }),
    nativeToScVal('42', { type: 'u32' }),
  ],
  value: nativeToScVal([
    nativeToScVal(DONOR_WALLET, { type: 'address' }),
    '500',
    '500',
  ]),
  txHash: 'tx-donation-001',
});

const poolStateEvent = eventResponse({
  topic: [
    nativeToScVal('pool_stat', { type: 'symbol' }),
    nativeToScVal('42', { type: 'u32' }),
  ],
  value: nativeToScVal([['Active'], ['Completed']]),
  txHash: 'tx-pool-stat-001',
});

const poolClosedEvent = eventResponse({
  topic: [
    nativeToScVal('pool_cls', { type: 'symbol' }),
    nativeToScVal('42', { type: 'u32' }),
  ],
  value: nativeToScVal([
    nativeToScVal(CLOSER_WALLET, { type: 'address' }),
    '50000',
  ]),
  txHash: 'tx-close-001',
});

describe('SyncService', () => {
  let service: SyncService;
  const upsertFromChain = jest.fn();
  const markStatus = jest.fn();
  const incrementRaised = jest.fn();
  const findByContractId = jest.fn();
  const isTxProcessed = jest.fn();
  const recordDonation = jest.fn();
  const syncStateRepo = { findOne: jest.fn(), save: jest.fn() };
  const advisoryLock = {
    runExclusive: jest.fn(
      (
        _key: number,
        _job: string,
        callback: () => Promise<unknown>,
      ): Promise<unknown> => callback(),
    ),
  };
  const rpcServer = {
    getEvents: jest.fn(),
    getHealth: jest.fn(),
  };
  const stellarConfig = {
    contractId: STELLAR_CONTRACT_ID,
    rpcUrl: 'https://soroban-testnet.stellar.org',
    syncStartLedger: 0,
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        SyncService,
        {
          provide: PoolsService,
          useValue: { upsertFromChain, markStatus, incrementRaised, findByContractId },
        },
        {
          provide: DonationsService,
          useValue: { isTxProcessed, recordDonation },
        },
        { provide: getRepositoryToken(SyncState), useValue: syncStateRepo },
        { provide: AdvisoryLockService, useValue: advisoryLock },
        { provide: StellarConfig, useValue: stellarConfig },
      ],
    }).compile();

    service = module.get(SyncService);
    (service as any).rpcServer = rpcServer;
    upsertFromChain.mockReset();
    markStatus.mockReset().mockResolvedValue(null);
    incrementRaised.mockReset().mockResolvedValue(null);
    findByContractId.mockReset().mockResolvedValue(null);
    isTxProcessed.mockReset().mockResolvedValue(false);
    recordDonation.mockReset().mockResolvedValue(undefined);
    syncStateRepo.findOne.mockReset();
    syncStateRepo.save.mockReset();
    advisoryLock.runExclusive.mockClear();
    rpcServer.getEvents.mockReset().mockResolvedValue({ events: [], cursor: '' });
    rpcServer.getHealth.mockReset().mockResolvedValue(healthResponse);
  });

  describe('pollHorizonEvents', () => {
    const contractFilter = [
      { type: 'contract', contractIds: [STELLAR_CONTRACT_ID] },
    ] as StellarRpc.Api.EventFilter[];

    it('dispatches contract events from RPC getEvents to the process* handlers', async () => {
      rpcServer.getEvents.mockResolvedValueOnce({
        events: [poolCreatedEvent],
        cursor: '',
      });

      await service.pollHorizonEvents();

      expect(rpcServer.getEvents).toHaveBeenCalledWith({
        filters: contractFilter,
        startLedger: healthResponse.oldestLedger,
      });
      expect(upsertFromChain).toHaveBeenCalledWith({
        contractPoolId: '42',
        creatorWallet: CREATOR_WALLET,
        goal: '50000',
      });
    });

    it('persists the cursor after each processed page and continues from it', async () => {
      rpcServer.getEvents
        .mockResolvedValueOnce({ events: [donationEvent], cursor: '111' })
        .mockResolvedValueOnce({ events: [], cursor: '' });

      await service.pollHorizonEvents();

      expect(rpcServer.getEvents).toHaveBeenNthCalledWith(1, {
        filters: contractFilter,
        startLedger: healthResponse.oldestLedger,
      });
      expect(rpcServer.getEvents).toHaveBeenNthCalledWith(2, {
        filters: contractFilter,
        cursor: '111',
      });
      expect(recordDonation).toHaveBeenCalledTimes(1);
      expect(syncStateRepo.save).toHaveBeenCalledWith({
        key: 'horizon_cursor',
        value: '111',
      });
    });

    it('does not persist a cursor when no events are returned', async () => {
      await service.pollHorizonEvents();

      expect(recordDonation).not.toHaveBeenCalled();
      expect(upsertFromChain).not.toHaveBeenCalled();
      expect(syncStateRepo.save).not.toHaveBeenCalled();
    });

    it('restarts from the RPC-oldest ledger when the saved cursor is stale', async () => {
      service['currentCursor'] = '500';

      await service.pollHorizonEvents();

      expect(rpcServer.getEvents).toHaveBeenCalledWith({
        filters: contractFilter,
        startLedger: healthResponse.oldestLedger,
      });
    });

    it('runs the poll inside the advisory lock', async () => {
      const seenInRun = (service as unknown as { seenInRun: Set<string> })
        .seenInRun;
      const clearSpy = jest.spyOn(seenInRun, 'clear');

      await service.pollHorizonEvents();

      expect(advisoryLock.runExclusive).toHaveBeenCalledWith(
        ADVISORY_LOCK_KEYS.syncHorizonEvents,
        'SyncService.pollHorizonEvents',
        expect.any(Function),
      );
      expect(clearSpy).toHaveBeenCalledTimes(1);
    });

    it('does not run the poll when another instance holds the lock', async () => {
      advisoryLock.runExclusive.mockImplementationOnce(() =>
        Promise.resolve(undefined),
      );
      const seenInRun = (service as unknown as { seenInRun: Set<string> })
        .seenInRun;
      const clearSpy = jest.spyOn(seenInRun, 'clear');

      await service.pollHorizonEvents();

      expect(clearSpy).not.toHaveBeenCalled();
    });
  });

  describe('processPoolCreatedEvent', () => {
    it('extracts contractPoolId, creatorWallet, and goal and calls upsertFromChain', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-42'],
        value: ['GABC123', '50000', 'My Pool', 'A great pool'],
      };

      await service.processPoolCreatedEvent(event);

      expect(upsertFromChain).toHaveBeenCalledWith({
        contractPoolId: 'pool-42',
        creatorWallet: 'GABC123',
        goal: '50000',
      });
    });

    it('reads contractPoolId from topic[1], creatorWallet from value[0], and goal from value[1]', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-xyz'],
        value: ['GCREATOR', '12345', 'ignored-name'],
      };

      await service.processPoolCreatedEvent(event);

      expect(upsertFromChain).toHaveBeenCalledTimes(1);
      expect(upsertFromChain).toHaveBeenCalledWith({
        contractPoolId: 'pool-xyz',
        creatorWallet: 'GCREATOR',
        goal: '12345',
      });
    });

    it('short-circuits on a duplicate txHash already seen in-run', async () => {
      const loggerWarnSpy = jest
        .spyOn((service as any).logger, 'warn')
        .mockImplementation(() => {});
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-dup'],
        value: ['GDUP', '300'],
        txHash: 'dup-hash',
      };

      await service.processPoolCreatedEvent(event);
      await service.processPoolCreatedEvent(event);

      expect(loggerWarnSpy).toHaveBeenCalledWith(
        expect.stringContaining('dup-hash'),
      );
      expect(upsertFromChain).toHaveBeenCalledTimes(1);
    });

    it('short-circuits on a duplicate txHash already processed in DB', async () => {
      isTxProcessed.mockResolvedValue(true);
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-dbdup'],
        value: ['GDBDUP', '400'],
        txHash: 'db-dup-hash',
      };

      await service.processPoolCreatedEvent(event);

      expect(isTxProcessed).toHaveBeenCalledWith('db-dup-hash');
      expect(upsertFromChain).not.toHaveBeenCalled();
    });

    it('processes an event with no txHash (duplicate check skipped)', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-notx'],
        value: ['GNOTX', '500'],
      };

      await service.processPoolCreatedEvent(event);

      expect(isTxProcessed).not.toHaveBeenCalled();
      expect(upsertFromChain).toHaveBeenCalledTimes(1);
    });
  });

  describe('processDonationEvent', () => {
    it('logs a warning and skips processing when event.txHash is missing', async () => {
      const loggerWarnSpy = jest
        .spyOn((service as any).logger, 'warn')
        .mockImplementation(() => {});
      const event: HorizonContractEvent = {
        topic: ['donation', 'pool-1'],
        value: ['GDONOR', '100'],
      };

      await service.processDonationEvent(event);

      expect(loggerWarnSpy).toHaveBeenCalled();
      expect(recordDonation).not.toHaveBeenCalled();
      expect(incrementRaised).not.toHaveBeenCalled();
    });

    it("defaults asset to 'XLM' when value[2] is omitted", async () => {
      const event: HorizonContractEvent = {
        topic: ['donation', 'pool-2'],
        value: ['GDONOR', '250'],
        txHash: 'donation-hash-1',
      };

      await service.processDonationEvent(event);

      expect(recordDonation).toHaveBeenCalledTimes(1);
      expect(recordDonation).toHaveBeenCalledWith(
        expect.objectContaining({ asset: 'XLM' }),
      );
    });

    it('calls recordDonation and incrementRaised with matching poolId and amount', async () => {
      const event: HorizonContractEvent = {
        topic: ['donation', 'pool-3'],
        value: ['GDONOR', '750', 'USDC'],
        txHash: 'donation-hash-2',
      };

      await service.processDonationEvent(event);

      expect(recordDonation).toHaveBeenCalledTimes(1);
      expect(incrementRaised).toHaveBeenCalledTimes(1);
      expect(recordDonation).toHaveBeenCalledWith(
        expect.objectContaining({ poolId: 'pool-3', amount: '750' }),
      );
      expect(incrementRaised).toHaveBeenCalledWith('pool-3', '750');
    });

    it('marks the pool Completed when donations reach the goal', async () => {
      findByContractId.mockResolvedValueOnce({
        contractPoolId: 'pool-3',
        creatorWallet: 'GCREATOR',
        goal: '750',
        raised: '750',
        status: PoolStatus.Active,
      } as Pool);

      const event: HorizonContractEvent = {
        topic: ['donation', 'pool-3'],
        value: ['GDONOR', '750', '750'],
        txHash: 'donation-goal-1',
      };

      await service.processDonationEvent(event);

      expect(recordDonation).toHaveBeenCalledTimes(1);
      expect(incrementRaised).toHaveBeenCalledWith('pool-3', '750');
      expect(markStatus).toHaveBeenCalledWith('pool-3', PoolStatus.Completed);
    });

    it('does not mark the pool Completed when donations fall short of the goal', async () => {
      findByContractId.mockResolvedValueOnce({
        contractPoolId: 'pool-3',
        creatorWallet: 'GCREATOR',
        goal: '1000',
        raised: '750',
        status: PoolStatus.Active,
      } as Pool);

      const event: HorizonContractEvent = {
        topic: ['donation', 'pool-3'],
        value: ['GDONOR', '750', '750'],
        txHash: 'donation-nogoal-1',
      };

      await service.processDonationEvent(event);

      expect(markStatus).not.toHaveBeenCalled();
    });
  });

  describe('cursor persistence', () => {
    it('getCursor returns null when no cursor has been set', () => {
      expect(service.getCursor()).toBeNull();
    });

    it('saveCursor persists the cursor value and updates in-memory state', async () => {
      syncStateRepo.save.mockResolvedValue({
        key: 'horizon_cursor',
        value: 'cursor-42',
      });

      await service.saveCursor('cursor-42');

      expect(service.getCursor()).toBe('cursor-42');
      expect(syncStateRepo.save).toHaveBeenCalledWith({
        key: 'horizon_cursor',
        value: 'cursor-42',
      });
    });

    it('saveCursor overwrites a previously set cursor', async () => {
      syncStateRepo.save.mockResolvedValue({
        key: 'horizon_cursor',
        value: 'cursor-first',
      });
      await service.saveCursor('cursor-first');

      syncStateRepo.save.mockResolvedValue({
        key: 'horizon_cursor',
        value: 'cursor-second',
      });
      await service.saveCursor('cursor-second');

      expect(service.getCursor()).toBe('cursor-second');
      expect(syncStateRepo.save).toHaveBeenCalledTimes(2);
    });

    it('onModuleInit restores the cursor from the database on startup', async () => {
      syncStateRepo.findOne.mockResolvedValue({
        key: 'horizon_cursor',
        value: 'restored-cursor',
      });

      await service.onModuleInit();

      expect(syncStateRepo.findOne).toHaveBeenCalledWith({
        where: { key: 'horizon_cursor' },
      });
      expect(service.getCursor()).toBe('restored-cursor');
    });

    it('onModuleInit leaves cursor null when no state exists in the database', async () => {
      syncStateRepo.findOne.mockResolvedValue(null);

      await service.onModuleInit();

      expect(service.getCursor()).toBeNull();
    });

    it('onModuleInit queries findOne with the horizon_cursor key and restores the value', async () => {
      syncStateRepo.findOne.mockResolvedValue({
        key: 'horizon_cursor',
        value: 'restored-cursor',
      });

      await service.onModuleInit();

      expect(syncStateRepo.findOne).toHaveBeenCalledWith({
        where: { key: 'horizon_cursor' },
      });
      expect(service.getCursor()).toBe('restored-cursor');
    });

    it('onModuleInit keeps the cursor null when findOne resolves null', async () => {
      syncStateRepo.findOne.mockResolvedValue(null);

      await service.onModuleInit();

      expect(syncStateRepo.findOne).toHaveBeenCalledWith({
        where: { key: 'horizon_cursor' },
      });
      expect(service.getCursor()).toBeNull();
    });

    it('onModuleInit does not overwrite an already-set cursor incorrectly', async () => {
      syncStateRepo.save.mockResolvedValue({
        key: 'horizon_cursor',
        value: 'existing-cursor',
      });
      await service.saveCursor('existing-cursor');

      syncStateRepo.findOne.mockResolvedValue({
        key: 'horizon_cursor',
        value: 'restored-cursor',
      });
      await service.onModuleInit();

      expect(service.getCursor()).toBe('restored-cursor');
    });

    it('onModuleInit ignores a row with a different key', async () => {
      syncStateRepo.findOne.mockResolvedValue(null);

      await service.onModuleInit();

      expect(syncStateRepo.findOne).toHaveBeenCalledWith({
        where: { key: 'horizon_cursor' },
      });
      expect(service.getCursor()).toBeNull();
    });
  });

  describe('processPoolClosedEvent', () => {
    it('extracts contractPoolId from topic and calls markStatus', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_cls', 'pool-99'],
        value: [],
      };

      await service.processPoolClosedEvent(event);

      expect(markStatus).toHaveBeenCalledWith('pool-99', PoolStatus.Closed);
    });

    it('calls markStatus with the correct id for different pool ids', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_cls', 'pool-7'],
        value: [],
      };

      await service.processPoolClosedEvent(event);

      expect(markStatus).toHaveBeenCalledTimes(1);
      expect(markStatus).toHaveBeenCalledWith('pool-7', PoolStatus.Closed);
    });

    it('calls PoolsService.markStatus with topic[1] as contractPoolId', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_clos', 'pool-closed-42'],
        value: [],
        txHash: 'tx-close-42',
      };

      await service.processPoolClosedEvent(event);

      expect(isTxProcessed).toHaveBeenCalledWith('tx-close-42');
      expect(markStatus).toHaveBeenCalledTimes(1);
      expect(markStatus).toHaveBeenCalledWith('pool-closed-42', PoolStatus.Closed);
    });

    it('short-circuits on a duplicate txHash already seen in-run before calling markStatus', async () => {
      const loggerWarnSpy = jest
        .spyOn((service as any).logger, 'warn')
        .mockImplementation(() => {});
      const event: HorizonContractEvent = {
        topic: ['pool_clos', 'pool-dup-close'],
        value: [],
        txHash: 'dup-close-hash',
      };

      await service.processPoolClosedEvent(event);
      await service.processPoolClosedEvent(event);

      expect(loggerWarnSpy).toHaveBeenCalledWith(
        expect.stringContaining('dup-close-hash'),
      );
      expect(markStatus).toHaveBeenCalledTimes(1);
      expect(markStatus).toHaveBeenCalledWith('pool-dup-close', PoolStatus.Closed);
    });

    it('short-circuits on a duplicate txHash already processed in DB before calling markStatus', async () => {
      isTxProcessed.mockResolvedValue(true);
      const event: HorizonContractEvent = {
        topic: ['pool_clos', 'pool-dbdup-close'],
        value: [],
        txHash: 'db-dup-close-hash',
      };

      await service.processPoolClosedEvent(event);

      expect(isTxProcessed).toHaveBeenCalledWith('db-dup-close-hash');
      expect(markStatus).not.toHaveBeenCalled();
    });

    it('processes an event with no txHash (duplicate check skipped)', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_clos', 'pool-notx-close'],
        value: [],
      };

      await service.processPoolClosedEvent(event);

      expect(isTxProcessed).not.toHaveBeenCalled();
      expect(markStatus).toHaveBeenCalledTimes(1);
      expect(markStatus).toHaveBeenCalledWith('pool-notx-close', PoolStatus.Closed);
    });
  });

  describe('processPoolStateEvent', () => {
    it('maps the new on-chain state name to the DB PoolStatus', async () => {
      const event = service['decodeEvent'](poolStateEvent);
      await service.processPoolStateEvent(event);
      expect(markStatus).toHaveBeenCalledWith('42', PoolStatus.Completed);
    });

    it('ignores pool_stat events with an unknown state name', async () => {
      await service.processPoolStateEvent({
        topic: ['pool_stat', '42'],
        value: ['Active', 'Bogus'],
      });
      expect(markStatus).not.toHaveBeenCalled();
    });
  });

  describe('decodeEvent', () => {
    it('decodes pool_crtd topic and value to strings', () => {
      const decoded = service['decodeEvent'](poolCreatedEvent);
      expect(decoded.topic).toEqual(['pool_crtd', '42']);
      expect(decoded.value).toEqual([
        CREATOR_WALLET,
        '50000',
        'My Pool',
        'Description',
      ]);
    });

    it('decodes pool_stat state names as strings', () => {
      const decoded = service['decodeEvent'](poolStateEvent);
      expect(decoded.topic[0]).toBe('pool_stat');
      expect(decoded.value[0]).toBe('Active');
      expect(decoded.value[1]).toBe('Completed');
    });
  });

  describe('idempotency', () => {
    it('skips processPoolCreatedEvent when tx is already in DB', async () => {
      isTxProcessed.mockResolvedValue(true);
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-1'],
        value: ['GABC', '100'],
        txHash: 'abc123',
      };

      await service.processPoolCreatedEvent(event);

      expect(upsertFromChain).not.toHaveBeenCalled();
    });

    it('skips processPoolClosedEvent when tx is already in DB', async () => {
      isTxProcessed.mockResolvedValue(true);
      const event: HorizonContractEvent = {
        topic: ['pool_cls', 'pool-1'],
        value: [],
        txHash: 'abc123',
      };

      await service.processPoolClosedEvent(event);

      expect(markStatus).not.toHaveBeenCalled();
    });

    it('processes event when txHash is new', async () => {
      isTxProcessed.mockResolvedValue(false);
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-2'],
        value: ['GXYZ', '200'],
        txHash: 'newhash',
      };

      await service.processPoolCreatedEvent(event);

      expect(upsertFromChain).toHaveBeenCalledTimes(1);
    });

    it('warns and skips on duplicate txHash within the same run', async () => {
      const loggerWarnSpy = jest
        .spyOn((service as any).logger, 'warn')
        .mockImplementation(() => {});
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-3'],
        value: ['GDUP', '300'],
        txHash: 'dup-hash',
      };

      await service.processPoolCreatedEvent(event);
      await service.processPoolCreatedEvent(event);

      expect(loggerWarnSpy).toHaveBeenCalledWith(
        expect.stringContaining('dup-hash'),
      );
      expect(upsertFromChain).toHaveBeenCalledTimes(1);
    });

    it('processes event without txHash (no idempotency check)', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-4'],
        value: ['GNOHASH', '400'],
      };

      await service.processPoolCreatedEvent(event);

      expect(isTxProcessed).not.toHaveBeenCalled();
      expect(upsertFromChain).toHaveBeenCalledTimes(1);
    });
  });

  describe('processDonationEvent', () => {
    it('skips when txHash is missing', async () => {
      const event: HorizonContractEvent = {
        topic: ['donation', 'pool-1'],
        value: ['GDONOR', '100'],
      };

      await service.processDonationEvent(event);

      expect(isTxProcessed).not.toHaveBeenCalled();
      expect(recordDonation).not.toHaveBeenCalled();
      expect(incrementRaised).not.toHaveBeenCalled();
    });

    it('skips when isTxDuplicate returns true', async () => {
      isTxProcessed.mockResolvedValue(true);
      const event: HorizonContractEvent = {
        topic: ['donation', 'pool-1'],
        value: ['GDONOR', '100'],
        txHash: 'dup-donation-hash',
      };

      await service.processDonationEvent(event);

      expect(recordDonation).not.toHaveBeenCalled();
      expect(incrementRaised).not.toHaveBeenCalled();
    });

    it('handles a DB-level unique violation gracefully when a race slips past isTxDuplicate', async () => {
      isTxProcessed.mockResolvedValue(false);
      const uniqueViolation = Object.assign(
        new Error('duplicate key value violates unique constraint'),
        { code: '23505' },
      );
      recordDonation.mockRejectedValue(uniqueViolation);
      const event: HorizonContractEvent = {
        topic: ['donation', 'pool-1'],
        value: ['GDONOR', '100'],
        txHash: 'race-hash',
      };

      await expect(service.processDonationEvent(event)).resolves.not.toThrow();

      expect(recordDonation).toHaveBeenCalledTimes(1);
      expect(incrementRaised).not.toHaveBeenCalled();
    });
  });
});
