import { MigrationInterface, QueryRunner } from 'typeorm';

export class ExtendPoolStatus1800000000005 implements MigrationInterface {
  async up(queryRunner: QueryRunner): Promise<void> {
    // The DB enum was created with ('Active', 'Completed') (see
    // CreatePoolsTable); mirror the contract's PoolState values.
    await queryRunner.query(
      `ALTER TYPE "public"."pools_status_enum" ADD VALUE 'Paused'`,
    );
    await queryRunner.query(
      `ALTER TYPE "public"."pools_status_enum" ADD VALUE 'Cancelled'`,
    );
    await queryRunner.query(
      `ALTER TYPE "public"."pools_status_enum" ADD VALUE 'Disbursed'`,
    );
    await queryRunner.query(
      `ALTER TYPE "public"."pools_status_enum" ADD VALUE 'Closed'`,
    );
  }

  async down(queryRunner: QueryRunner): Promise<void> {
    // PostgreSQL cannot remove enum values; reverting requires rewriting the
    // stored statuses back to a subset of ('Active', 'Completed').
  }
}
