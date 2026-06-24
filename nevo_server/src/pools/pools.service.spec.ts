import { Test, TestingModule } from '@nestjs/testing';
import { PoolsService } from './pools.service';
import { MockPoolRepository } from './pools.repository';

describe('PoolsService', () => {
  let service: PoolsService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [PoolsService, MockPoolRepository],
    }).compile();

    service = module.get<PoolsService>(PoolsService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  it('should return a paginated list of pools with default limits', async () => {
    const result = await service.findAll({});
    expect(result.data).toBeDefined();
    expect(result.data.length).toBeLessThanOrEqual(10);
    expect(result.total).toBe(15);
    expect(result.page).toBe(1);
    expect(result.limit).toBe(10);
  });

  it('should handle pagination offset', async () => {
    const result = await service.findAll({ page: '2', limit: '5' });
    expect(result.data.length).toBe(5);
    expect(result.total).toBe(15);
    expect(result.page).toBe(2);
    expect(result.limit).toBe(5);
  });

  it('should filter by category (exact match case-insensitively)', async () => {
    const result = await service.findAll({ category: 'Environment' });
    expect(result.data.every((p) => p.category.toLowerCase() === 'environment')).toBe(true);
    expect(result.total).toBe(3); // Garden, Reforestation, Solar
  });

  it('should filter by status (exact match case-insensitively)', async () => {
    const result = await service.findAll({ status: 'Completed' });
    expect(result.data.every((p) => p.status.toLowerCase() === 'completed')).toBe(true);
  });

  it('should filter by search text (case-insensitive title or description)', async () => {
    const result = await service.findAll({ search: 'Stellar' });
    expect(result.data.length).toBe(2); // Open Source Dev Fund, Stellar Smart Contract Auditing
  });

  it('should sort by createdAt DESC by default', async () => {
    const result = await service.findAll({});
    const dates = result.data.map((p) => p.createdAt.getTime());
    for (let i = 0; i < dates.length - 1; i++) {
      expect(dates[i]).toBeGreaterThanOrEqual(dates[i + 1]);
    }
  });
});
