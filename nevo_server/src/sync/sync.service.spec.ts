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

describe('SyncService', () => {
  let service: SyncService;
  const upsertFromChain = jest.fn();
  const markCompleted = jest.fn();
  const isTxProcessed = jest.fn();
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

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        SyncService,
        { provide: PoolsService, useValue: { upsertFromChain, markCompleted } },
        { provide: DonationsService, useValue: { isTxProcessed } },
        { provide: getRepositoryToken(SyncState), useValue: syncStateRepo },
        { provide: AdvisoryLockService, useValue: advisoryLock },
      ],
    }).compile();

    service = module.get(SyncService);
    upsertFromChain.mockReset();
    markCompleted.mockReset();
    isTxProcessed.mockReset().mockResolvedValue(false);
    syncStateRepo.findOne.mockReset();
    syncStateRepo.save.mockReset();
    advisoryLock.runExclusive.mockClear();
  });

  describe('pollHorizonEvents', () => {
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
  });

  describe('processPoolClosedEvent', () => {
    it('extracts contractPoolId from topic and calls markCompleted', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_cls', 'pool-99'],
        value: [],
      };

      await service.processPoolClosedEvent(event);

      expect(markCompleted).toHaveBeenCalledWith('pool-99');
    });

    it('calls markCompleted with the correct id for different pool ids', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_cls', 'pool-7'],
        value: [],
      };

      await service.processPoolClosedEvent(event);

      expect(markCompleted).toHaveBeenCalledTimes(1);
      expect(markCompleted).toHaveBeenCalledWith('pool-7');
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

      expect(markCompleted).not.toHaveBeenCalled();
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
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
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
});
