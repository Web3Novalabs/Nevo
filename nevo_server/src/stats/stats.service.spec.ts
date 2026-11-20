import { ConfigService } from '@nestjs/config';
import { DataSource } from 'typeorm';
import { StatsService } from './stats.service';

describe('StatsService', () => {
  let query: jest.Mock;
  let transaction: jest.Mock;
  let env: Record<string, string>;
  let service: StatsService;

  const seed = (
    totals: Array<{ asset: string; amount: string }>,
    pools: Array<{ status: string; count: string }>,
    donors: string,
  ) => {
    query.mockImplementation((sql: string) => {
      if (sql.includes('GROUP BY asset')) return Promise.resolve(totals);
      if (sql.includes('GROUP BY status')) return Promise.resolve(pools);
      return Promise.resolve([{ count: donors }]);
    });
  };

  beforeEach(() => {
    jest.useFakeTimers();
    env = {};
    query = jest.fn();
    transaction = jest.fn(
      (_level: string, cb: (m: { query: jest.Mock }) => Promise<unknown>) =>
        cb({ query }),
    );
    service = new StatsService(
      { transaction } as unknown as DataSource,
      { get: (k: string) => env[k] } as unknown as ConfigService,
    );
    seed([], [], '0');
  });

  afterEach(() => jest.useRealTimers());

  it('returns zeros and an empty asset list for an empty database', async () => {
    await expect(service.getStats()).resolves.toEqual({
      totalRaised: [],
      activePools: 0,
      completedPools: 0,
      uniqueDonors: 0,
    });
  });

  it('maps asset totals, pool statuses and donors', async () => {
    seed(
      [
        { asset: 'USDC', amount: '170141183460469231731687303715884105727' },
        { asset: 'XLM', amount: '9007199254740993' },
      ],
      [
        { status: 'Active', count: '3' },
        { status: 'Completed', count: '2' },
      ],
      '7',
    );

    const stats = await service.getStats();

    expect(stats).toEqual({
      totalRaised: [
        { asset: 'USDC', amount: '170141183460469231731687303715884105727' },
        { asset: 'XLM', amount: '9007199254740993' },
      ],
      activePools: 3,
      completedPools: 2,
      uniqueDonors: 7,
    });
    expect(typeof stats.totalRaised[0].amount).toBe('string');
  });

  it('reads everything in one REPEATABLE READ snapshot', async () => {
    await service.getStats();

    expect(transaction).toHaveBeenCalledTimes(1);
    expect((transaction.mock.calls as string[][])[0][0]).toBe(
      'REPEATABLE READ',
    );
  });

  it('serves the cached value within the TTL and refreshes after it', async () => {
    env.STATS_CACHE_TTL_MS = '1000';
    seed([], [{ status: 'Active', count: '1' }], '0');
    expect((await service.getStats()).activePools).toBe(1);

    seed([], [{ status: 'Active', count: '2' }], '0');
    jest.advanceTimersByTime(999);
    expect((await service.getStats()).activePools).toBe(1);
    expect(transaction).toHaveBeenCalledTimes(1);

    jest.advanceTimersByTime(2);
    expect((await service.getStats()).activePools).toBe(2);
    expect(transaction).toHaveBeenCalledTimes(2);
  });

  it('defaults to a 30s TTL', async () => {
    await service.getStats();
    jest.advanceTimersByTime(29_000);
    await service.getStats();
    expect(transaction).toHaveBeenCalledTimes(1);

    jest.advanceTimersByTime(1_500);
    await service.getStats();
    expect(transaction).toHaveBeenCalledTimes(2);
  });

  it('does not cache when the TTL is 0', async () => {
    env.STATS_CACHE_TTL_MS = '0';

    await service.getStats();
    await service.getStats();

    expect(transaction).toHaveBeenCalledTimes(2);
  });

  it('shares one query between concurrent callers', async () => {
    await Promise.all([
      service.getStats(),
      service.getStats(),
      service.getStats(),
    ]);

    expect(transaction).toHaveBeenCalledTimes(1);
  });

  it('does not cache failures', async () => {
    transaction.mockRejectedValueOnce(new Error('db down'));

    await expect(service.getStats()).rejects.toThrow('db down');
    await expect(service.getStats()).resolves.toMatchObject({ activePools: 0 });
  });
});
