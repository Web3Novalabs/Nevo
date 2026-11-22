import { Injectable } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { ThrottlerStorage } from '@nestjs/throttler';
import { InjectDataSource } from '@nestjs/typeorm';
import { DataSource } from 'typeorm';

type ThrottlerRecord = Awaited<ReturnType<ThrottlerStorage['increment']>>;

@Injectable()
export class PostgresThrottlerStorage implements ThrottlerStorage {
  constructor(@InjectDataSource() private readonly dataSource: DataSource) {}

  async increment(
    key: string,
    ttl: number,
    limit: number,
    blockDuration: number,
    throttlerName: string,
  ): Promise<ThrottlerRecord> {
    const [record] = await this.dataSource.query<ThrottlerRecord[]>(
      `
      INSERT INTO throttle_buckets AS bucket
        (bucket_key, hits, expires_at, blocked_until)
      VALUES (
        $1, 1, statement_timestamp() + $2 * INTERVAL '1 millisecond',
        CASE WHEN 1 > $3 THEN statement_timestamp() + $4 * INTERVAL '1 millisecond' END
      )
      ON CONFLICT (bucket_key) DO UPDATE
      SET (hits, expires_at, blocked_until) = (
        SELECT hits,
          CASE WHEN reset_window THEN EXCLUDED.expires_at ELSE bucket.expires_at END,
          CASE
            WHEN is_blocked THEN bucket.blocked_until
            WHEN hits > $3 THEN statement_timestamp() + $4 * INTERVAL '1 millisecond'
          END
        FROM (
          SELECT reset_window, is_blocked,
            CASE
              WHEN is_blocked THEN bucket.hits
              WHEN reset_window THEN 1
              ELSE bucket.hits + 1
            END AS hits
          FROM (
            SELECT
              COALESCE(bucket.blocked_until > statement_timestamp(), false) AS is_blocked,
              COALESCE(
                bucket.blocked_until <= statement_timestamp(),
                bucket.expires_at <= statement_timestamp()
              )
                AS reset_window
          ) AS current_window
        ) AS next_window
      )
      RETURNING
        hits AS "totalHits",
        GREATEST(0, CEIL(EXTRACT(EPOCH FROM expires_at - statement_timestamp())))::int
          AS "timeToExpire",
        COALESCE(blocked_until > statement_timestamp(), false) AS "isBlocked",
        GREATEST(0, CEIL(EXTRACT(EPOCH FROM blocked_until - statement_timestamp())))::int
          AS "timeToBlockExpire"
      `,
      [`${throttlerName}:${key}`, ttl, limit, blockDuration],
    );
    return record;
  }

  @Cron(CronExpression.EVERY_MINUTE)
  async deleteExpiredBuckets(): Promise<void> {
    await this.dataSource.query(
      `DELETE FROM throttle_buckets
       WHERE GREATEST(expires_at, blocked_until) <= statement_timestamp()`,
    );
  }
}
