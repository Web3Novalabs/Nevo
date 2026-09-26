import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { ConfigService } from '@nestjs/config';
import { PoolsService, ChainPoolData } from './pools.service';
import { Pool, PoolStatus } from './pool.entity';
import { ContractService } from '../contract/contract.service';

// ---------------------------------------------------------------------------
// Helper types
// ---------------------------------------------------------------------------

type MockQueryBuilder = {
  andWhere: jest.Mock;
  orderBy: jest.Mock;
  skip: jest.Mock;
  take: jest.Mock;
  getManyAndCount: jest.Mock;
};

// ---------------------------------------------------------------------------
// Core builder — extended to support createQueryBuilder for findAll
// ---------------------------------------------------------------------------

async function buildService(
  existing: Pool | null,
  queryBuilderOverrides?: Partial<MockQueryBuilder>,
) {
  let lastSaved: Pool | undefined;

  const qb: MockQueryBuilder = {
    andWhere: jest.fn().mockReturnThis(),
    orderBy: jest.fn().mockReturnThis(),
    skip: jest.fn().mockReturnThis(),
    take: jest.fn().mockReturnThis(),
    getManyAndCount: jest
      .fn()
      .mockResolvedValue(existing ? [[existing], 1] : [[], 0]),
    ...queryBuilderOverrides,
  };

  const repo = {
    findOne: jest.fn().mockResolvedValue(existing),
    save: jest.fn().mockImplementation((p: Pool) => {
      lastSaved = p;
      return Promise.resolve(p);
    }),
    create: jest
      .fn()
      .mockImplementation((d: Partial<Pool>) => ({ ...d }) as Pool),
    createQueryBuilder: jest.fn().mockReturnValue(qb),
  };

  const mockContractService = {
    getPoolOnChain: jest.fn().mockResolvedValue(null),
    getTotalRaisedOnChain: jest.fn().mockResolvedValue(0n),
    getDonorCountOnChain: jest.fn().mockResolvedValue(0),
    buildClosePoolTransaction: jest.fn().mockReturnValue('close-xdr-string'),
    buildWithdrawTransaction: jest
      .fn()
      .mockReturnValue('withdraw-xdr-string'),
  };

  const mockConfigService = {
    getOrThrow: jest.fn().mockImplementation((key: string) => {
      if (key === 'TOKEN_ADDRESS') return 'CTOKENADDRESS';
      throw new Error(`Configuration key "${key}" does not exist`);
    }),
  };

  const module: TestingModule = await Test.createTestingModule({
    providers: [
      PoolsService,
      { provide: getRepositoryToken(Pool), useValue: repo },
      { provide: ContractService, useValue: mockContractService },
      { provide: ConfigService, useValue: mockConfigService },
    ],
  }).compile();

  return {
    service: module.get(PoolsService),
    contractService: mockContractService,
    config: mockConfigService,
    repo,
    qb,
    savedArg: () => lastSaved as Pool,
  };
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const chainData: ChainPoolData = {
  contractPoolId: '1',
  creatorWallet: 'GWALLET',
  goal: '10000',
};

const makePool = (overrides: Partial<Pool> = {}): Pool => ({
  id: 'uuid-1',
  contractPoolId: '1',
  creatorWallet: 'GWALLET',
  goal: '10000',
  raised: '0',
  status: PoolStatus.Active,
  category: 'charity',
  title: 'Test Pool',
  description: 'A test pool',
  imageUrl: null,
  createdAt: new Date(),
  updatedAt: new Date(),
  ...overrides,
});

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('PoolsService', () => {
  // ── upsertFromChain ──────────────────────────────────────────────────────

  it('creates a new pool when none exists', async () => {
    const { service, savedArg } = await buildService(null);

    await service.upsertFromChain(chainData);

    expect(savedArg()).toMatchObject(chainData);
  });

  it('updates chain fields without overwriting off-chain metadata', async () => {
    const existing = makePool({
      creatorWallet: 'GOLD',
      goal: '5000',
      title: 'Existing Title',
      description: 'Existing description',
      imageUrl: 'https://example.com/img.png',
    });
    const { service, savedArg } = await buildService(existing);

    await service.upsertFromChain(chainData);

    const saved = savedArg();
    expect(saved.creatorWallet).toBe('GWALLET');
    expect(saved.goal).toBe('10000');
    expect(saved.title).toBe('Existing Title');
    expect(saved.description).toBe('Existing description');
    expect(saved.imageUrl).toBe('https://example.com/img.png');
  });

  // ── findAll ───────────────────────────────────────────────────────────────

  describe('findAll', () => {
    it('returns all pools with default pagination when no filters are provided', async () => {
      const existing = makePool();
      const { service } = await buildService(existing);

      const result = await service.findAll({});

      expect(result.data).toHaveLength(1);
      expect(result.total).toBe(1);
      expect(result.page).toBe(1);
      expect(result.limit).toBe(20);
    });

    it('applies custom page and limit parameters', async () => {
      const existing = makePool();
      const { service, qb } = await buildService(existing);

      const result = await service.findAll({ page: 3, limit: 10 });

      // skip = (3-1)*10 = 20, take = 10
      expect(qb.skip).toHaveBeenCalledWith(20);
      expect(qb.take).toHaveBeenCalledWith(10);
      expect(result.page).toBe(3);
      expect(result.limit).toBe(10);
    });

    it('applies category filter when provided', async () => {
      const existing = makePool({ category: 'charity' });
      const { service, qb } = await buildService(existing);

      await service.findAll({ category: 'charity' });

      expect(qb.andWhere).toHaveBeenCalledWith(
        'LOWER(pool.category) = LOWER(:category)',
        { category: 'charity' },
      );
    });

    it('applies status filter when provided', async () => {
      const existing = makePool({ status: PoolStatus.Active });
      const { service, qb } = await buildService(existing);

      await service.findAll({ status: PoolStatus.Active });

      expect(qb.andWhere).toHaveBeenCalledWith('pool.status = :status', {
        status: PoolStatus.Active,
      });
    });

    it('applies search filter when provided', async () => {
      const existing = makePool();
      const { service, qb } = await buildService(existing);

      await service.findAll({ search: 'water' });

      expect(qb.andWhere).toHaveBeenCalledWith(
        '(LOWER(pool.title) LIKE LOWER(:search) OR LOWER(pool.description) LIKE LOWER(:search))',
        { search: '%water%' },
      );
    });

    it('combined filter + pagination: applies both category filter and custom pagination', async () => {
      const pools = [makePool(), makePool({ id: 'uuid-2', contractPoolId: '2' })];
      const { service, qb } = await buildService(null, {
        getManyAndCount: jest.fn().mockResolvedValue([pools, 2]),
      });

      const result = await service.findAll({
        category: 'environment',
        page: 2,
        limit: 5,
      });

      // Category filter applied
      expect(qb.andWhere).toHaveBeenCalledWith(
        'LOWER(pool.category) = LOWER(:category)',
        { category: 'environment' },
      );
      // Pagination applied
      expect(qb.skip).toHaveBeenCalledWith(5); // (2-1)*5
      expect(qb.take).toHaveBeenCalledWith(5);
      expect(result.page).toBe(2);
      expect(result.limit).toBe(5);
      expect(result.total).toBe(2);
    });

    it('returns empty data array when no pools match', async () => {
      const { service } = await buildService(null, {
        getManyAndCount: jest.fn().mockResolvedValue([[], 0]),
      });

      const result = await service.findAll({ search: 'nonexistent' });

      expect(result.data).toEqual([]);
      expect(result.total).toBe(0);
    });

    it('orders results by createdAt DESC', async () => {
      const { service, qb } = await buildService(null, {
        getManyAndCount: jest.fn().mockResolvedValue([[], 0]),
      });

      await service.findAll({});

      expect(qb.orderBy).toHaveBeenCalledWith('pool.createdAt', 'DESC');
    });
  });

  // ── create ────────────────────────────────────────────────────────────────

  describe('create', () => {
    it('saves a new pool with all provided fields', async () => {
      const { service, repo, savedArg } = await buildService(null);
      const dto = {
        contractPoolId: 'pool-42',
        creatorWallet: 'GCREATOR',
        goal: '500000',
        title: 'New Pool',
        description: 'A brand new pool',
        category: 'tech',
        imageUrl: 'https://img.example.com/pool.png',
      };

      await service.create(dto);

      expect(repo.create).toHaveBeenCalledWith(
        expect.objectContaining({
          contractPoolId: 'pool-42',
          creatorWallet: 'GCREATOR',
          goal: '500000',
          title: 'New Pool',
          description: 'A brand new pool',
          category: 'tech',
          imageUrl: 'https://img.example.com/pool.png',
          status: PoolStatus.Active,
          raised: '0',
        }),
      );
      expect(repo.save).toHaveBeenCalled();
    });

    it('sets status to Active and raised to "0" for every new pool', async () => {
      const { service, savedArg } = await buildService(null);
      const dto = {
        contractPoolId: 'pool-99',
        creatorWallet: 'GWALLET',
        goal: '1000',
        title: 'Pool',
        description: 'desc',
      };

      await service.create(dto);

      expect(savedArg().status).toBe(PoolStatus.Active);
      expect(savedArg().raised).toBe('0');
    });

    it('uses empty strings for optional fields when omitted', async () => {
      const { service, repo } = await buildService(null);
      const dto = {
        contractPoolId: 'pool-minimal',
        creatorWallet: 'GWALLET',
        goal: '100',
        title: 'Minimal',
        description: 'desc',
      };

      await service.create(dto);

      expect(repo.create).toHaveBeenCalledWith(
        expect.objectContaining({
          category: '',
          imageUrl: null,
        }),
      );
    });

    it('returns the saved pool entity', async () => {
      const savedPool = makePool({ contractPoolId: 'pool-ret' });
      const { service, repo } = await buildService(null);
      repo.save.mockResolvedValue(savedPool);

      const result = await service.create({
        contractPoolId: 'pool-ret',
        creatorWallet: 'GWALLET',
        goal: '1000',
        title: 'Pool',
        description: 'desc',
      });

      expect(result).toBe(savedPool);
    });
  });

  // ── updateMeta ────────────────────────────────────────────────────────────

  describe('updateMeta', () => {
    it('updates description when provided', async () => {
      const existing = makePool({ description: 'Old description' });
      const { service, savedArg } = await buildService(existing);

      await service.updateMeta('1', { description: 'New description' });

      expect(savedArg().description).toBe('New description');
    });

    it('updates imageUrl when provided', async () => {
      const existing = makePool({ imageUrl: null });
      const { service, savedArg } = await buildService(existing);

      await service.updateMeta('1', { imageUrl: 'https://new-image.com/img.png' });

      expect(savedArg().imageUrl).toBe('https://new-image.com/img.png');
    });

    it('updates category when provided', async () => {
      const existing = makePool({ category: 'old-category' });
      const { service, savedArg } = await buildService(existing);

      await service.updateMeta('1', { category: 'new-category' });

      expect(savedArg().category).toBe('new-category');
    });

    it('updates multiple fields in one call', async () => {
      const existing = makePool({
        description: 'Old',
        imageUrl: null,
        category: 'old',
      });
      const { service, savedArg } = await buildService(existing);

      await service.updateMeta('1', {
        description: 'Updated desc',
        imageUrl: 'https://img.com/new.png',
        category: 'updated-cat',
      });

      expect(savedArg().description).toBe('Updated desc');
      expect(savedArg().imageUrl).toBe('https://img.com/new.png');
      expect(savedArg().category).toBe('updated-cat');
    });

    it('skips fields that are undefined in the DTO (partial update)', async () => {
      const existing = makePool({ description: 'Keep this', category: 'keep' });
      const { service, savedArg } = await buildService(existing);

      // Only update imageUrl — description and category should be unchanged
      await service.updateMeta('1', { imageUrl: 'https://new.png' });

      expect(savedArg().description).toBe('Keep this');
      expect(savedArg().category).toBe('keep');
    });

    it('returns null when pool is not found', async () => {
      const { service } = await buildService(null);

      const result = await service.updateMeta('nonexistent', {
        description: 'Should not save',
      });

      expect(result).toBeNull();
    });

    it('calls repo.save with the updated pool', async () => {
      const existing = makePool();
      const { service, repo } = await buildService(existing);

      await service.updateMeta('1', { description: 'Updated' });

      expect(repo.save).toHaveBeenCalledWith(existing);
    });
  });

  // ── findByContractId ──────────────────────────────────────────────────────

  describe('findByContractId', () => {
    it('returns the pool when a matching contractPoolId exists', async () => {
      const existing = makePool({ contractPoolId: 'pool-123' });
      const { service, repo } = await buildService(existing);

      const result = await service.findByContractId('pool-123');

      expect(repo.findOne).toHaveBeenCalledWith({
        where: { contractPoolId: 'pool-123' },
      });
      expect(result).toBe(existing);
    });

    it('returns null when no pool matches the given contractPoolId', async () => {
      const { service } = await buildService(null);

      const result = await service.findByContractId('nonexistent-id');

      expect(result).toBeNull();
    });

    it('queries by contractPoolId, not by numeric id', async () => {
      const existing = makePool({ contractPoolId: 'contract-abc' });
      const { service, repo } = await buildService(existing);

      await service.findByContractId('contract-abc');

      expect(repo.findOne).toHaveBeenCalledWith({
        where: { contractPoolId: 'contract-abc' },
      });
    });
  });

  // ── buildWithdrawTx ───────────────────────────────────────────────────────

  describe('buildWithdrawTx', () => {
    it('calls ContractService.buildWithdrawTransaction with creatorWallet, numeric poolId and TOKEN_ADDRESS', async () => {
      const pool = makePool({ contractPoolId: '7', creatorWallet: 'GCREATOR_KEY' });
      const { service, contractService } = await buildService(pool);

      service.buildWithdrawTx(pool);

      expect(contractService.buildWithdrawTransaction).toHaveBeenCalledWith(
        'GCREATOR_KEY',
        7,
        'CTOKENADDRESS',
      );
    });

    it('returns the XDR from ContractService and the pool contractPoolId', async () => {
      const pool = makePool({ contractPoolId: '77', creatorWallet: 'GWALLET' });
      const { service, contractService } = await buildService(pool);
      contractService.buildWithdrawTransaction.mockReturnValue('withdraw-xdr');

      const result = service.buildWithdrawTx(pool);

      expect(result).toEqual({ unsignedXdr: 'withdraw-xdr', poolId: '77' });
    });

    it('throws when TOKEN_ADDRESS is not configured', async () => {
      const pool = makePool({ contractPoolId: '1' });
      const { service, contractService, config } = await buildService(pool);
      config.getOrThrow.mockImplementationOnce(() => {
        throw new Error('Configuration key "TOKEN_ADDRESS" does not exist');
      });

      expect(() => service.buildWithdrawTx(pool)).toThrow('TOKEN_ADDRESS');
      expect(contractService.buildWithdrawTransaction).not.toHaveBeenCalled();
    });
  });

  // ── buildClosePoolTx ──────────────────────────────────────────────────────

  describe('buildClosePoolTx', () => {
    it('returns an object with unsignedXdr field', async () => {
      const pool = makePool({ contractPoolId: '5', creatorWallet: 'GCREATOR' });
      const { service } = await buildService(pool);

      const result = service.buildClosePoolTx(pool);

      expect(result).toHaveProperty('unsignedXdr');
    });

    it('calls ContractService.buildClosePoolTransaction with creatorWallet and numeric poolId', async () => {
      const pool = makePool({ contractPoolId: '5', creatorWallet: 'GCREATOR_KEY' });
      const { service, contractService } = await buildService(pool);

      service.buildClosePoolTx(pool);

      expect(contractService.buildClosePoolTransaction).toHaveBeenCalledWith(
        'GCREATOR_KEY',
        5,
      );
    });

    it('returns the XDR string from ContractService', async () => {
      const pool = makePool({ contractPoolId: '3', creatorWallet: 'GWALLET' });
      const { service, contractService } = await buildService(pool);
      contractService.buildClosePoolTransaction.mockReturnValue('close-pool-xdr');

      const result = service.buildClosePoolTx(pool);

      expect(result.unsignedXdr).toBe('close-pool-xdr');
    });

    it('parses contractPoolId as integer when calling ContractService', async () => {
      const pool = makePool({ contractPoolId: '42', creatorWallet: 'GWALLET' });
      const { service, contractService } = await buildService(pool);

      service.buildClosePoolTx(pool);

      const [, poolIdArg] = contractService.buildClosePoolTransaction.mock.calls[0];
      expect(typeof poolIdArg).toBe('number');
      expect(poolIdArg).toBe(42);
    });
  });

  // ── findOneMerged ─────────────────────────────────────────────────────────

  describe('findOneMerged', () => {
    it('returns null if the pool is not found in the DB', async () => {
      const { service } = await buildService(null);
      const result = await service.findOneMerged('1');
      expect(result).toBeNull();
    });

    it('returns merged data if the pool is found and contract service returns data', async () => {
      const existing = makePool({
        contractPoolId: '1',
        creatorWallet: 'GOLD',
        goal: '5000',
        title: 'Existing Title',
        description: 'Existing description',
        imageUrl: 'https://example.com/img.png',
      });
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
      const existing = makePool({
        contractPoolId: '1',
        creatorWallet: 'GOLD',
        goal: '5000',
        title: 'Existing Title',
        description: 'Existing description',
        imageUrl: 'https://example.com/img.png',
      });
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

  // ── markCompleted ─────────────────────────────────────────────────────────

  describe('markCompleted', () => {
    it('sets pool status to Completed and saves', async () => {
      const existing = makePool({ status: PoolStatus.Active });
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

  // ── incrementRaised ───────────────────────────────────────────────────────

  describe('incrementRaised', () => {
    it('adds amount to an existing non-zero raised value', async () => {
      const existing = makePool({ raised: '1000' });
      const { service, savedArg } = await buildService(existing);

      const result = await service.incrementRaised('1', '500');

      expect(savedArg().raised).toBe('1500');
      expect(result?.raised).toBe('1500');
    });

    it('adds amount when raised is "0"', async () => {
      const existing = makePool({ raised: '0' });
      const { service, savedArg } = await buildService(existing);

      const result = await service.incrementRaised('1', '250');

      expect(savedArg().raised).toBe('250');
      expect(result?.raised).toBe('250');
    });

    it('handles pool with empty or missing raised string by defaulting to "0"', async () => {
      const existing = makePool({ raised: '' });
      const { service, savedArg } = await buildService(existing);

      const result = await service.incrementRaised('1', '100');

      expect(savedArg().raised).toBe('100');
      expect(result?.raised).toBe('100');
    });

    it('returns null when the pool is not found', async () => {
      const { service, repo } = await buildService(null);

      const result = await service.incrementRaised('nonexistent', '100');

      expect(repo.findOne).toHaveBeenCalledWith({
        where: { contractPoolId: 'nonexistent' },
      });
      expect(result).toBeNull();
    });
  });
});

