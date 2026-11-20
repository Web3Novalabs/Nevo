import { MigrationInterface, QueryRunner } from 'typeorm';

export class IndexPoolSearch1800000000002 implements MigrationInterface {
  async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query('CREATE EXTENSION IF NOT EXISTS pg_trgm');
    await queryRunner.query(
      'CREATE INDEX pools_title_trgm ON pools USING gin (title gin_trgm_ops)',
    );
    await queryRunner.query(
      'CREATE INDEX pools_description_trgm ON pools USING gin (description gin_trgm_ops)',
    );
  }

  async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query('DROP INDEX pools_description_trgm');
    await queryRunner.query('DROP INDEX pools_title_trgm');
  }
}
