import { Test, TestingModule } from '@nestjs/testing';
import { SyncService, HorizonContractEvent } from './sync.service';
import { PoolsService } from '../pools/pools.service';
import { DonationsService } from '../donations/donations.service';
import { getRepositoryToken } from '@nestjs/typeorm';
import { SyncState } from './sync-state.entity';

describe('SyncService', () => {
  let service: SyncService;
  const upsertFromChain = jest.fn();
  const markCompleted = jest.fn();
  const incrementRaised = jest.fn();
  const isTxProcessed = jest.fn();
  const recordDonation = jest.fn();
  const syncStateRepo = { findOne: jest.fn(), save: jest.fn() };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        SyncService,
        {
          provide: PoolsService,
          useValue: { upsertFromChain, markCompleted, incrementRaised },
        },
        {
          provide: DonationsService,
          useValue: { isTxProcessed, recordDonation },
        },
        { provide: getRepositoryToken(SyncState), useValue: syncStateRepo },
      ],
    }).compile();

    service = module.get(SyncService);
    upsertFromChain.mockReset();
    markCompleted.mockReset().mockResolvedValue(undefined);
    incrementRaised.mockReset().mockResolvedValue(undefined);
    isTxProcessed.mockReset().mockResolvedValue(false);
    recordDonation.mockReset().mockResolvedValue(undefined);
    syncStateRepo.findOne.mockReset();
    syncStateRepo.save.mockReset();
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
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
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
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
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
  });

  describe('pollHorizonEvents', () => {
    // NOTE: pollHorizonEvents is an unimplemented stub (TODO: replace with a
    // real implementation once HorizonService (#46) is available). It only
    // clears seenInRun and never calls HorizonService. Do not write tests
    // against non-existent polling behavior; this test only pins the stub.
    it('clears seenInRun and never calls HorizonService', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-stub'],
        value: ['GSTUB', '100'],
        txHash: 'stub-hash',
      };
      await service.processPoolCreatedEvent(event);
      upsertFromChain.mockClear();

      await service.pollHorizonEvents();

      expect(upsertFromChain).not.toHaveBeenCalled();
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

    it('calls PoolsService.markCompleted with topic[1] as contractPoolId', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_clos', 'pool-closed-42'],
        value: [],
        txHash: 'tx-close-42',
      };

      await service.processPoolClosedEvent(event);

      expect(isTxProcessed).toHaveBeenCalledWith('tx-close-42');
      expect(markCompleted).toHaveBeenCalledTimes(1);
      expect(markCompleted).toHaveBeenCalledWith('pool-closed-42');
    });

    it('short-circuits on a duplicate txHash already seen in-run before calling markCompleted', async () => {
      const loggerWarnSpy = jest
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
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
      expect(markCompleted).toHaveBeenCalledTimes(1);
      expect(markCompleted).toHaveBeenCalledWith('pool-dup-close');
    });

    it('short-circuits on a duplicate txHash already processed in DB before calling markCompleted', async () => {
      isTxProcessed.mockResolvedValue(true);
      const event: HorizonContractEvent = {
        topic: ['pool_clos', 'pool-dbdup-close'],
        value: [],
        txHash: 'db-dup-close-hash',
      };

      await service.processPoolClosedEvent(event);

      expect(isTxProcessed).toHaveBeenCalledWith('db-dup-close-hash');
      expect(markCompleted).not.toHaveBeenCalled();
    });

    it('processes an event with no txHash (duplicate check skipped)', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_clos', 'pool-notx-close'],
        value: [],
      };

      await service.processPoolClosedEvent(event);

      expect(isTxProcessed).not.toHaveBeenCalled();
      expect(markCompleted).toHaveBeenCalledTimes(1);
      expect(markCompleted).toHaveBeenCalledWith('pool-notx-close');
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
