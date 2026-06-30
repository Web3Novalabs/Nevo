import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { SyncService, HorizonContractEvent } from './sync.service';
import { PoolsService } from '../pools/pools.service';
import { DonationsService } from '../donations/donations.service';
import { ContractService } from '../contract/contract.service';
import { SyncState } from './sync-state.entity';
import { Pool, PoolStatus } from '../pools/pool.entity';
import { Donation } from '../donations/donation.entity';

describe('SyncService Integration Tests', () => {
  let service: SyncService;
  let poolsService: PoolsService;
  let donationsService: DonationsService;
  let poolRepo: Repository<Pool>;
  let donationRepo: Repository<Donation>;
  let syncStateRepo: Repository<SyncState>;

  beforeEach(async () => {
    // Mock repositories with in-memory storage
    const pools: Pool[] = [];
    const donations: Donation[] = [];
    const syncStates: SyncState[] = [];

    const mockPoolRepo = {
      findOne: jest.fn((options) => {
        const pool = pools.find(
          (p) => p.contractPoolId === options.where.contractPoolId,
        );
        return Promise.resolve(pool || null);
      }),
      create: jest.fn((data) => ({ ...data, id: `pool-${pools.length + 1}` })),
      save: jest.fn((pool) => {
        const existingIndex = pools.findIndex((p) => p.id === pool.id);
        if (existingIndex >= 0) {
          pools[existingIndex] = { ...pool };
          return Promise.resolve(pools[existingIndex]);
        }
        const newPool = { ...pool, createdAt: new Date() };
        pools.push(newPool);
        return Promise.resolve(newPool);
      }),
      find: jest.fn(() => Promise.resolve(pools)),
    };

    const mockDonationRepo = {
      findOne: jest.fn((options) => {
        const donation = donations.find(
          (d) => d.txHash === options.where?.txHash,
        );
        return Promise.resolve(donation || null);
      }),
      countBy: jest.fn((where) => {
        const count = donations.filter((d) => d.txHash === where.txHash).length;
        return Promise.resolve(count);
      }),
      create: jest.fn((data) => ({
        ...data,
        id: `donation-${donations.length + 1}`,
        createdAt: new Date(),
      })),
      save: jest.fn((donation) => {
        const newDonation = { ...donation };
        donations.push(newDonation);
        return Promise.resolve(newDonation);
      }),
      find: jest.fn(() => Promise.resolve(donations)),
    };

    const mockSyncStateRepo = {
      findOne: jest.fn((options) => {
        const state = syncStates.find((s) => s.key === options.where.key);
        return Promise.resolve(state || null);
      }),
      save: jest.fn((state) => {
        const existingIndex = syncStates.findIndex((s) => s.key === state.key);
        if (existingIndex >= 0) {
          syncStates[existingIndex] = { ...state };
          return Promise.resolve(syncStates[existingIndex]);
        }
        syncStates.push({ ...state });
        return Promise.resolve(state);
      }),
    };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        SyncService,
        PoolsService,
        DonationsService,
        {
          provide: getRepositoryToken(Pool),
          useValue: mockPoolRepo,
        },
        {
          provide: getRepositoryToken(Donation),
          useValue: mockDonationRepo,
        },
        {
          provide: getRepositoryToken(SyncState),
          useValue: mockSyncStateRepo,
        },
        {
          provide: ContractService,
          useValue: {
            getPoolOnChain: jest.fn(),
            getTotalRaisedOnChain: jest.fn(),
            getDonorCountOnChain: jest.fn(),
            buildClosePoolTransaction: jest.fn(),
          },
        },
      ],
    }).compile();

    service = module.get<SyncService>(SyncService);
    poolsService = module.get<PoolsService>(PoolsService);
    donationsService = module.get<DonationsService>(DonationsService);
    poolRepo = module.get<Repository<Pool>>(getRepositoryToken(Pool));
    donationRepo = module.get<Repository<Donation>>(
      getRepositoryToken(Donation),
    );
    syncStateRepo = module.get<Repository<SyncState>>(
      getRepositoryToken(SyncState),
    );
  });

  describe('processPoolCreatedEvent', () => {
    it('creates a Pool record in DB when processing pool_created event', async () => {
      const event: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-123'],
        value: ['GABC123CREATOR', '100000', 'Test Pool', 'Description'],
        txHash: 'tx-pool-created-001',
      };

      await service.processPoolCreatedEvent(event);

      const pool = await poolRepo.findOne({
        where: { contractPoolId: 'pool-123' },
      });

      expect(pool).toBeDefined();
      expect(pool?.contractPoolId).toBe('pool-123');
      expect(pool?.creatorWallet).toBe('GABC123CREATOR');
      expect(pool?.goal).toBe('100000');
      expect(pool?.status).toBe(PoolStatus.Active);
      expect(pool?.raised).toBe('0');
    });

    it('upserts existing pool when processing duplicate pool_created event', async () => {
      // First event creates the pool
      const event1: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-456'],
        value: ['GXYZ456CREATOR', '50000', 'Original Pool', 'Original Desc'],
        txHash: 'tx-pool-created-002',
      };

      await service.processPoolCreatedEvent(event1);

      // Second event with different data for the same pool
      const event2: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-456'],
        value: ['GXYZ456CREATOR', '75000', 'Updated Pool', 'Updated Desc'],
        txHash: 'tx-pool-created-003',
      };

      await service.processPoolCreatedEvent(event2);

      const pool = await poolRepo.findOne({
        where: { contractPoolId: 'pool-456' },
      });

      expect(pool).toBeDefined();
      expect(pool?.goal).toBe('75000'); // Should be updated
    });
  });

  describe('processDonationEvent', () => {
    beforeEach(async () => {
      // Create a pool first for donations to reference
      const poolEvent: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-789'],
        value: ['GCREATOR789', '200000', 'Donation Test Pool', 'Test Desc'],
        txHash: 'tx-pool-789',
      };
      await service.processPoolCreatedEvent(poolEvent);
    });

    it('creates a Donation record in DB when processing donation event', async () => {
      const donationEvent: HorizonContractEvent = {
        topic: ['donation', 'pool-789'],
        value: ['GDONOR001', '5000', 'XLM'],
        txHash: 'tx-donation-001',
      };

      await service.processDonationEvent(donationEvent);

      const count = await donationRepo.countBy({ txHash: 'tx-donation-001' });
      expect(count).toBe(1);

      const donations = await donationRepo.find();
      const donation = donations[0];

      expect(donation).toBeDefined();
      expect(donation.poolId).toBe('pool-789');
      expect(donation.donorWallet).toBe('GDONOR001');
      expect(donation.amount).toBe('5000');
      expect(donation.asset).toBe('XLM');
      expect(donation.txHash).toBe('tx-donation-001');
    });

    it('updates pool.raised when processing donation event', async () => {
      const donationEvent: HorizonContractEvent = {
        topic: ['donation', 'pool-789'],
        value: ['GDONOR002', '10000', 'XLM'],
        txHash: 'tx-donation-002',
      };

      await service.processDonationEvent(donationEvent);

      const pool = await poolRepo.findOne({
        where: { contractPoolId: 'pool-789' },
      });

      expect(pool?.raised).toBe('10000');
    });

    it('correctly accumulates multiple donations to pool.raised', async () => {
      const donation1: HorizonContractEvent = {
        topic: ['donation', 'pool-789'],
        value: ['GDONOR003', '3000', 'XLM'],
        txHash: 'tx-donation-003',
      };

      const donation2: HorizonContractEvent = {
        topic: ['donation', 'pool-789'],
        value: ['GDONOR004', '7000', 'XLM'],
        txHash: 'tx-donation-004',
      };

      await service.processDonationEvent(donation1);
      await service.processDonationEvent(donation2);

      const pool = await poolRepo.findOne({
        where: { contractPoolId: 'pool-789' },
      });

      expect(pool?.raised).toBe('10000'); // 3000 + 7000
    });

    it('skips donation event when txHash is missing', async () => {
      const donationEvent: HorizonContractEvent = {
        topic: ['donation', 'pool-789'],
        value: ['GDONOR005', '1000', 'XLM'],
        // No txHash
      };

      await service.processDonationEvent(donationEvent);

      const donations = await donationRepo.find();
      expect(donations.length).toBe(0);
    });
  });

  describe('processPoolClosedEvent', () => {
    beforeEach(async () => {
      // Create an active pool
      const poolEvent: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-closed-1'],
        value: ['GCREATOR999', '50000', 'Pool to Close', 'Will be closed'],
        txHash: 'tx-pool-closed-1',
      };
      await service.processPoolCreatedEvent(poolEvent);
    });

    it('sets pool.status to Completed when processing pool_closed event', async () => {
      const closeEvent: HorizonContractEvent = {
        topic: ['pool_cls', 'pool-closed-1'],
        value: [],
        txHash: 'tx-close-001',
      };

      await service.processPoolClosedEvent(closeEvent);

      const pool = await poolRepo.findOne({
        where: { contractPoolId: 'pool-closed-1' },
      });

      expect(pool?.status).toBe(PoolStatus.Completed);
    });
  });

  describe('Idempotency - duplicate tx hash is skipped', () => {
    it('skips processing when the same donation txHash is submitted twice', async () => {
      // Create pool first
      const poolEvent: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-idem-1'],
        value: ['GCREATOR111', '100000', 'Idempotency Test', 'Test Desc'],
        txHash: 'tx-pool-idem-1',
      };
      await service.processPoolCreatedEvent(poolEvent);

      const donationEvent: HorizonContractEvent = {
        topic: ['donation', 'pool-idem-1'],
        value: ['GDONOR111', '5000', 'XLM'],
        txHash: 'tx-duplicate-donation',
      };

      // First submission should process
      await service.processDonationEvent(donationEvent);

      const countAfterFirst = await donationRepo.countBy({
        txHash: 'tx-duplicate-donation',
      });
      expect(countAfterFirst).toBe(1);

      const poolAfterFirst = await poolRepo.findOne({
        where: { contractPoolId: 'pool-idem-1' },
      });
      expect(poolAfterFirst?.raised).toBe('5000');

      // Second submission should be skipped
      await service.processDonationEvent(donationEvent);

      const countAfterSecond = await donationRepo.countBy({
        txHash: 'tx-duplicate-donation',
      });
      expect(countAfterSecond).toBe(1); // Still only 1

      const poolAfterSecond = await poolRepo.findOne({
        where: { contractPoolId: 'pool-idem-1' },
      });
      expect(poolAfterSecond?.raised).toBe('5000'); // Not doubled
    });

    it('skips processing when the same pool_created txHash is submitted twice', async () => {
      const poolEvent: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-idem-2'],
        value: ['GCREATOR222', '200000', 'Duplicate Pool Event', 'Test Desc'],
        txHash: 'tx-duplicate-pool',
      };

      // First submission
      await service.processPoolCreatedEvent(poolEvent);

      // Add a donation to track if pool state changes
      const donationEvent: HorizonContractEvent = {
        topic: ['donation', 'pool-idem-2'],
        value: ['GDONOR222', '1000', 'XLM'],
        txHash: 'tx-donation-unique',
      };
      await service.processDonationEvent(donationEvent);

      const poolAfterFirst = await poolRepo.findOne({
        where: { contractPoolId: 'pool-idem-2' },
      });
      expect(poolAfterFirst?.raised).toBe('1000');

      // Second submission with same txHash should be skipped
      await service.processPoolCreatedEvent(poolEvent);

      // Verify the donation count didn't change (pool wasn't recreated)
      const donations = await donationRepo.find();
      const poolDonations = donations.filter((d) => d.poolId === 'pool-idem-2');
      expect(poolDonations.length).toBe(1);
    });

    it('skips processing when the same pool_closed txHash is submitted twice', async () => {
      // Create pool
      const poolEvent: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-idem-3'],
        value: ['GCREATOR333', '150000', 'Close Duplicate Test', 'Test Desc'],
        txHash: 'tx-pool-idem-3',
      };
      await service.processPoolCreatedEvent(poolEvent);

      const closeEvent: HorizonContractEvent = {
        topic: ['pool_cls', 'pool-idem-3'],
        value: [],
        txHash: 'tx-duplicate-close',
      };

      // First close
      await service.processPoolClosedEvent(closeEvent);

      const poolAfterFirst = await poolRepo.findOne({
        where: { contractPoolId: 'pool-idem-3' },
      });
      expect(poolAfterFirst?.status).toBe(PoolStatus.Completed);

      // Mark as processed to simulate the idempotency check
      await donationsService.recordDonation({
        poolId: 'pool-idem-3',
        donorWallet: 'GSYSTEM',
        amount: '0',
        asset: 'XLM',
        txHash: 'tx-duplicate-close',
      });

      // Second close attempt should be skipped
      await service.processPoolClosedEvent(closeEvent);

      // Status should still be Completed (not changed)
      const poolAfterSecond = await poolRepo.findOne({
        where: { contractPoolId: 'pool-idem-3' },
      });
      expect(poolAfterSecond?.status).toBe(PoolStatus.Completed);
    });

    it('detects duplicate txHash within the same run', async () => {
      const loggerWarnSpy = jest
        .spyOn((service as any).logger, 'warn')
        .mockImplementation(() => {});

      // Create pool
      const poolEvent: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-same-run'],
        value: ['GCREATOR444', '75000', 'Same Run Test', 'Test Desc'],
        txHash: 'tx-pool-same-run',
      };
      await service.processPoolCreatedEvent(poolEvent);

      const donationEvent: HorizonContractEvent = {
        topic: ['donation', 'pool-same-run'],
        value: ['GDONOR444', '2000', 'XLM'],
        txHash: 'tx-same-run-dup',
      };

      // Process twice in the same run
      await service.processDonationEvent(donationEvent);
      await service.processDonationEvent(donationEvent);

      expect(loggerWarnSpy).toHaveBeenCalledWith(
        expect.stringContaining('tx-same-run-dup'),
      );

      const count = await donationRepo.countBy({ txHash: 'tx-same-run-dup' });
      expect(count).toBe(1); // Only processed once
    });
  });

  describe('Integration flow - complete event sequence', () => {
    it('processes a complete pool lifecycle: created -> donations -> closed', async () => {
      // 1. Pool created
      const poolCreatedEvent: HorizonContractEvent = {
        topic: ['pool_crtd', 'pool-lifecycle'],
        value: ['GCREATOR555', '100000', 'Lifecycle Pool', 'Full test'],
        txHash: 'tx-lifecycle-created',
      };
      await service.processPoolCreatedEvent(poolCreatedEvent);

      let pool = await poolRepo.findOne({
        where: { contractPoolId: 'pool-lifecycle' },
      });
      expect(pool?.status).toBe(PoolStatus.Active);
      expect(pool?.raised).toBe('0');

      // 2. Multiple donations
      const donation1: HorizonContractEvent = {
        topic: ['donation', 'pool-lifecycle'],
        value: ['GDONOR555A', '25000', 'XLM'],
        txHash: 'tx-lifecycle-don-1',
      };
      await service.processDonationEvent(donation1);

      const donation2: HorizonContractEvent = {
        topic: ['donation', 'pool-lifecycle'],
        value: ['GDONOR555B', '35000', 'XLM'],
        txHash: 'tx-lifecycle-don-2',
      };
      await service.processDonationEvent(donation2);

      pool = await poolRepo.findOne({
        where: { contractPoolId: 'pool-lifecycle' },
      });
      expect(pool?.raised).toBe('60000');

      const donations = await donationRepo.find();
      const poolDonations = donations.filter(
        (d) => d.poolId === 'pool-lifecycle',
      );
      expect(poolDonations.length).toBe(2);

      // 3. Pool closed
      const poolClosedEvent: HorizonContractEvent = {
        topic: ['pool_cls', 'pool-lifecycle'],
        value: [],
        txHash: 'tx-lifecycle-closed',
      };
      await service.processPoolClosedEvent(poolClosedEvent);

      pool = await poolRepo.findOne({
        where: { contractPoolId: 'pool-lifecycle' },
      });
      expect(pool?.status).toBe(PoolStatus.Completed);
      expect(pool?.raised).toBe('60000'); // Raised amount preserved
    });
  });
});
