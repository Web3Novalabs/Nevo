import { MigrationInterface, QueryRunner } from 'typeorm';

export class AddPoolApplicationDeadline1800000000007 implements MigrationInterface {
  async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`
      ALTER TABLE pools ADD COLUMN application_deadline numeric(20, 0)
    `);
    await queryRunner.query(`
      CREATE INDEX idx_pools_pending_deadline ON pools (id)
      WHERE application_deadline IS NULL
    `);
    await queryRunner.query(`
      CREATE INDEX idx_pools_raised_sort ON pools (raised DESC, created_at DESC, id DESC)
    `);
    await queryRunner.query(`
      CREATE INDEX idx_pools_goal_sort ON pools (goal ASC, created_at DESC, id DESC)
    `);
  }

  async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query('DROP INDEX idx_pools_goal_sort');
    await queryRunner.query('DROP INDEX idx_pools_raised_sort');
    await queryRunner.query('DROP INDEX idx_pools_pending_deadline');
    await queryRunner.query(
      'ALTER TABLE pools DROP COLUMN application_deadline',
    );
  }
}
