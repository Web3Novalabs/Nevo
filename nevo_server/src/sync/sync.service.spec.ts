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
  const isTxProcessed = jest.fn();

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        SyncService,
        { provide: PoolsService, useValue: { upsertFromChain, markCompleted } },
        { provide: DonationsService, useValue: { isTxProcessed } },
        { provide: getRepositoryToken(SyncState), useValue: { findOne: jest.fn(), save: jest.fn() } },
      ],
    }).compile();

    service = module.get(SyncService);
    upsertFromChain.mockReset();
    markCompleted.mockReset();
    isTxProcessed.mockReset().mockResolvedValue(false);
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
      const loggerWarnSpy = jest.spyOn((service as any).logger, 'warn').mockImplementation(() => {});
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-3'],
        value: ['GDUP', '300'],
        txHash: 'dup-hash',
      };

      await service.processPoolCreatedEvent(event);
      await service.processPoolCreatedEvent(event);

      expect(loggerWarnSpy).toHaveBeenCalledWith(expect.stringContaining('dup-hash'));
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
