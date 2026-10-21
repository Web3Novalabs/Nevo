import { DataSource, QueryRunner } from 'typeorm';
import { CreateDonationsTable1750680081742 } from './1750680081742-CreateDonationsTable';
import { buildDataSourceOptions } from '../data-source';

describe('CreateDonationsTable migration', () => {
  let dataSource: DataSource;
  let queryRunner: QueryRunner;
  const migration = new CreateDonationsTable1750680081742();

  beforeAll(async () => {
    dataSource = new DataSource(buildDataSourceOptions());
    await dataSource.initialize();
    queryRunner = dataSource.createQueryRunner();
    await queryRunner.connect();
  });

  afterAll(async () => {
    if (queryRunner) {
      await queryRunner.release();
    }
    if (dataSource && dataSource.isInitialized) {
      await dataSource.destroy();
    }
  });

  afterEach(async () => {
    // Clean up any inserted rows between tests.
    if (await queryRunner.hasTable('donations')) {
      await queryRunner.query('DELETE FROM "donations"');
    }
  });

  it('up() creates the donations table with a unique index on tx_hash', async () => {
    await migration.up(queryRunner);

    expect(await queryRunner.hasTable('donations')).toBe(true);

    const indexes: Array<{ indexname: string; indexdef: string }> =
      await queryRunner.query(
        `SELECT indexname, indexdef FROM pg_indexes WHERE tablename = 'donations'`,
      );

    const uniqueIndex = indexes.find(
      (index) => index.indexname === 'UQ_donations_tx_hash',
    );
    expect(uniqueIndex).toBeDefined();
    expect(uniqueIndex!.indexdef).toMatch(/UNIQUE/i);
    expect(uniqueIndex!.indexdef).toMatch(/tx_hash/i);
  });

  it('rejects a duplicate tx_hash at the DB level', async () => {
    if (!(await queryRunner.hasTable('donations'))) {
      await migration.up(queryRunner);
    }

    const txHash = 'duplicate-tx-hash-for-test';

    await queryRunner.query(
      `INSERT INTO "donations" ("tx_hash") VALUES ($1)`,
      [txHash],
    );

    await expect(
      queryRunner.query(`INSERT INTO "donations" ("tx_hash") VALUES ($1)`, [
        txHash,
      ]),
    ).rejects.toThrow(/duplicate key value violates unique constraint/i);
  });

  it('down() reverses the migration cleanly', async () => {
    if (!(await queryRunner.hasTable('donations'))) {
      await migration.up(queryRunner);
    }

    await migration.down(queryRunner);

    expect(await queryRunner.hasTable('donations')).toBe(false);

    const indexes: Array<{ indexname: string }> = await queryRunner.query(
      `SELECT indexname FROM pg_indexes WHERE tablename = 'donations'`,
    );
    expect(indexes).toHaveLength(0);
  });
});