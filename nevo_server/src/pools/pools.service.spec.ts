import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { PoolsService, ChainPoolData } from './pools.service';
import { Pool, PoolStatus } from './pool.entity';
import { ContractService } from '../contract/contract.service';

describe('PoolsService', () => {
  const chainData: ChainPoolData = {
    contractPoolId: '1',
    creatorWallet: 'GWALLET',
    goal: '10000',
  };

  it('creates a new pool when none exists', async () => {
    const { service, savedArg } = await buildService(null);

    await service.upsertFromChain(chainData);

    expect(savedArg()).toMatchObject(chainData);
  });

  it('updates chain fields without overwriting off-chain metadata', async () => {
    const existing: Pool = {
      id: 'uuid-1',
      contractPoolId: '1',
      creatorWallet: 'GOLD',
      goal: '5000',
      raised: '0',
      status: PoolStatus.Active,
      category: '',
      title: 'Existing Title',
      description: 'Existing description',
      imageUrl: 'https://example.com/img.png',
      createdAt: new Date(),
      updatedAt: new Date(),
    };
    const { service, savedArg } = await buildService(existing);

    await service.upsertFromChain(chainData);

    const saved = savedArg();
    expect(saved.creatorWallet).toBe('GWALLET');
    expect(saved.goal).toBe('10000');
    expect(saved.title).toBe('Existing Title');
    expect(saved.description).toBe('Existing description');
    expect(saved.imageUrl).toBe('https://example.com/img.png');
  });

  describe('findOneMerged', () => {
    it('returns null if the pool is not found in the DB', async () => {
      const { service } = await buildService(null);
      const result = await service.findOneMerged('1');
      expect(result).toBeNull();
    });

    it('returns merged data if the pool is found and contract service returns data', async () => {
      const existing: Pool = {
        id: 'uuid-1',
        contractPoolId: '1',
        creatorWallet: 'GOLD',
        goal: '5000',
        raised: '0',
        status: PoolStatus.Active,
        category: '',
        title: 'Existing Title',
        description: 'Existing description',
        imageUrl: 'https://example.com/img.png',
        createdAt: new Date(),
        updatedAt: new Date(),
      };
      const { service, contractService } = await buildService(existing);

      contractService.getPoolOnChain.mockResolvedValue({
        id: 1,
        creator: 'GOLD',
        goal: 5000n,
        collected: 2500n,
        closed: true,
      });
      contractService.getDonorCountOnChain.mockResolvedValue(10);

      const result = await service.findOneMerged('1');
      expect(result).toEqual({
        ...existing,
        raisedOnChain: '2500',
        closedOnChain: true,
        donorCount: 10,
      });
    });

    it('falls back gracefully to DB raised value if getPoolOnChain returns null but total raised returns value', async () => {
      const existing: Pool = {
        id: 'uuid-1',
        contractPoolId: '1',
        creatorWallet: 'GOLD',
        goal: '5000',
        raised: '0',
        status: PoolStatus.Active,
        category: '',
        title: 'Existing Title',
        description: 'Existing description',
        imageUrl: 'https://example.com/img.png',
        createdAt: new Date(),
        updatedAt: new Date(),
      };
      const { service, contractService } = await buildService(existing);

      contractService.getPoolOnChain.mockResolvedValue(null);
      contractService.getTotalRaisedOnChain.mockResolvedValue(1500n);
      contractService.getDonorCountOnChain.mockResolvedValue(5);

      const result = await service.findOneMerged('1');
      expect(result).toEqual({
        ...existing,
        raisedOnChain: '1500',
        closedOnChain: false,
        donorCount: 5,
      });
    });
  });

  describe('markCompleted', () => {
    it('sets pool status to Completed and saves', async () => {
      const existing: Pool = {
        id: 'uuid-1',
        contractPoolId: 'pool-1',
        creatorWallet: 'GWALLET',
        goal: '5000',
        raised: '0',
        status: PoolStatus.Active,
        category: '',
        title: 'My Pool',
        description: '',
        imageUrl: null,
        createdAt: new Date(),
        updatedAt: new Date(),
      };
      const { service, savedArg } = await buildService(existing);

      await service.markCompleted('pool-1');

      expect(savedArg().status).toBe(PoolStatus.Completed);
    });

    it('returns null if pool is not found', async () => {
      const { service } = await buildService(null);
      const result = await service.markCompleted('nonexistent');
      expect(result).toBeNull();
    });
  });
});

async function buildService(existing: Pool | null) {
  let lastSaved: Pool | undefined;

  const repo = {
    findOne: jest.fn().mockResolvedValue(existing),
    save: jest.fn().mockImplementation((p: Pool) => {
      lastSaved = p;
      return Promise.resolve(p);
    }),
    create: jest
      .fn()
      .mockImplementation((d: Partial<Pool>) => ({ ...d }) as Pool),
  };

  const mockContractService = {
    getPoolOnChain: jest.fn().mockResolvedValue(null),
    getTotalRaisedOnChain: jest.fn().mockResolvedValue(0n),
    getDonorCountOnChain: jest.fn().mockResolvedValue(0),
  };

  const module: TestingModule = await Test.createTestingModule({
    providers: [
      PoolsService,
      { provide: getRepositoryToken(Pool), useValue: repo },
      { provide: ContractService, useValue: mockContractService },
    ],
  }).compile();

  return {
    service: module.get(PoolsService),
    contractService: mockContractService,
    savedArg: () => lastSaved as Pool,
  };
}
