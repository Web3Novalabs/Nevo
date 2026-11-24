import { Injectable, Logger } from '@nestjs/common';
import { InjectDataSource } from '@nestjs/typeorm';
import { DataSource } from 'typeorm';

export const ADVISORY_LOCK_KEYS = {
  syncHorizonEvents: 1754000001,
  nonceCleanup: 1754000002,
  poolReconciliation: 1754000003,
  poolDeadlineSync: 1754000004,
} as const;

@Injectable()
export class AdvisoryLockService {
  private readonly logger = new Logger(AdvisoryLockService.name);

  constructor(@InjectDataSource() private readonly dataSource: DataSource) {}

  async runExclusive<T>(
    key: number,
    job: string,
    callback: () => Promise<T>,
  ): Promise<T | undefined> {
    const queryRunner = this.dataSource.createQueryRunner();

    try {
      await queryRunner.connect();

      const rows = (await queryRunner.query(
        'SELECT pg_try_advisory_lock($1::bigint) AS acquired',
        [key],
      )) as Array<{ acquired: boolean }>;

      if (!rows[0]?.acquired) {
        this.logger.warn(
          `Skipping ${job}: advisory lock ${key} is held by another instance`,
        );
        return undefined;
      }

      try {
        return await callback();
      } finally {
        await queryRunner.query(
          'SELECT pg_advisory_unlock($1::bigint) AS released',
          [key],
        );
      }
    } finally {
      await queryRunner.release();
    }
  }
}
