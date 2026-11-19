import { Logger } from '@nestjs/common';
import { DataSource } from 'typeorm';
import { AdvisoryLockService } from './advisory-lock.service';

describe('AdvisoryLockService', () => {
  const heldLocks = new Set<number>();

  const createDataSource = () => {
    const queryRunner = {
      connect: jest.fn().mockResolvedValue(undefined),
      query: jest.fn((sql: string, params: unknown[]) => {
        const key = Number(params[0]);

        if (sql.includes('pg_try_advisory_lock')) {
          if (heldLocks.has(key)) {
            return Promise.resolve([{ acquired: false }]);
          }
          heldLocks.add(key);
          return Promise.resolve([{ acquired: true }]);
        }

        if (sql.includes('pg_advisory_unlock')) {
          heldLocks.delete(key);
          return Promise.resolve([{ released: true }]);
        }

        return Promise.reject(new Error(`Unexpected query: ${sql}`));
      }),
      release: jest.fn().mockResolvedValue(undefined),
    };

    return {
      queryRunner,
      dataSource: {
        createQueryRunner: jest.fn(() => queryRunner),
      } as unknown as DataSource,
    };
  };

  beforeEach(() => {
    heldLocks.clear();
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  it('lets only one of two instances run a job at a time', async () => {
    const first = new AdvisoryLockService(createDataSource().dataSource);
    const secondDb = createDataSource();
    const second = new AdvisoryLockService(secondDb.dataSource);
    const secondJob = jest.fn(() => Promise.resolve(undefined));
    const skipLogger = jest
      .spyOn((second as unknown as { logger: Logger }).logger, 'warn')
      .mockImplementation(() => {});

    await first.runExclusive(42, 'test-job', async () => {
      const skipped = await second.runExclusive(42, 'test-job', secondJob);
      expect(skipped).toBeUndefined();
    });

    expect(secondJob).not.toHaveBeenCalled();
    expect(skipLogger).toHaveBeenCalledWith(
      expect.stringContaining('held by another instance'),
    );
  });

  it('releases the lock when the job completes so another instance can run', async () => {
    const firstDb = createDataSource();
    const first = new AdvisoryLockService(firstDb.dataSource);
    const second = new AdvisoryLockService(createDataSource().dataSource);
    const secondJob = jest.fn().mockResolvedValue('second-result');

    await first.runExclusive(42, 'test-job', () =>
      Promise.resolve('first-result'),
    );
    const result = await second.runExclusive(42, 'test-job', secondJob);

    expect(result).toBe('second-result');
    expect(secondJob).toHaveBeenCalledTimes(1);
    expect(firstDb.queryRunner.query).toHaveBeenCalledWith(
      expect.stringContaining('pg_advisory_unlock'),
      [42],
    );
    expect(firstDb.queryRunner.release).toHaveBeenCalledTimes(1);
  });

  it('releases the lock when the job throws', async () => {
    const firstDb = createDataSource();
    const first = new AdvisoryLockService(firstDb.dataSource);
    const second = new AdvisoryLockService(createDataSource().dataSource);
    const secondJob = jest.fn().mockResolvedValue(undefined);

    await expect(
      first.runExclusive(42, 'test-job', () =>
        Promise.reject(new Error('job failed')),
      ),
    ).rejects.toThrow('job failed');

    await second.runExclusive(42, 'test-job', secondJob);

    expect(secondJob).toHaveBeenCalledTimes(1);
    expect(firstDb.queryRunner.release).toHaveBeenCalledTimes(1);
  });
});
