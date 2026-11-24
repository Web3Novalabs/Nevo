import { MigrationInterface, QueryRunner } from 'typeorm';

export class CreateThrottleBuckets1800000000006 implements MigrationInterface {
  async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`
      CREATE TABLE throttle_buckets (
        bucket_key text PRIMARY KEY,
        hits integer NOT NULL,
        expires_at timestamptz NOT NULL,
        blocked_until timestamptz
      )
    `);
    await queryRunner.query(`
      CREATE INDEX idx_throttle_buckets_expiration
      ON throttle_buckets (GREATEST(expires_at, blocked_until))
    `);
  }

  async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query('DROP TABLE throttle_buckets');
  }
}
