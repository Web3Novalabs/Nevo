import { Test, TestingModule } from '@nestjs/testing';
import { SyncService, HorizonContractEvent } from './sync.service';
import { PoolsService } from '../pools/pools.service';

describe('SyncService', () => {
  let service: SyncService;
  const upsertFromChain = jest.fn();
  const markCompleted = jest.fn();

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        SyncService,
        { provide: PoolsService, useValue: { upsertFromChain, markCompleted } },
      ],
    }).compile();

    service = module.get(SyncService);
    upsertFromChain.mockReset();
    markCompleted.mockReset();
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
});
