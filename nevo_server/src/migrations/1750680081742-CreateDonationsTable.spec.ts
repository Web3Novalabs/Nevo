import { DataSource, QueryRunner } from 'typeorm';
import { CreateDonationsTable1750680081742 } from './1750680081742-CreateDonationsTable';
import {
  createSchemaDataSource,
  dropSchemaAndDestroy,
} from '../testing/test-database';

const SCHEMA = 'spec_create_donations';

describe('CreateDonationsTable migration', () => {
  let dataSource: DataSource;
  let queryRunner: QueryRunner;
  const migration = new CreateDonationsTable1750680081742();

  const hasDonationsTable = async () => {
    const rows: unknown[] = await queryRunner.query(
      `SELECT 1 FROM information_schema.tables
       WHERE table_schema = $1 AND table_name = 'donations'`,
      [SCHEMA],
    );
    return rows.length > 0;
  };

  const insertDonation = (txHash: string) =>
    queryRunner.query(
      `INSERT INTO "donations" ("pool_id", "donor_wallet", "amount", "asset", "tx_hash")
       VALUES ('1', 'GDONOR', '10', 'XLM', $1)`,
      [txHash],
    );

  beforeAll(async () => {
    dataSource = await createSchemaDataSource(SCHEMA);
    queryRunner = dataSource.createQueryRunner();
    await queryRunner.connect();
    await queryRunner.query(`SET search_path TO "${SCHEMA}", public`);
  });

  afterAll(async () => {
    await queryRunner?.release();
    await dropSchemaAndDestroy(dataSource, SCHEMA);
  });

  afterEach(async () => {
    if (await hasDonationsTable()) {
      await migration.down(queryRunner);
    }
  });

  it('up() creates the donations table with a unique index on tx_hash', async () => {
    await migration.up(queryRunner);

    expect(await hasDonationsTable()).toBe(true);

    const indexes: Array<{ indexname: string; indexdef: string }> =
      await queryRunner.query(
        `SELECT indexname, indexdef FROM pg_indexes
         WHERE schemaname = $1 AND tablename = 'donations'`,
        [SCHEMA],
      );

    const uniqueIndex = indexes.find(
      (index) => index.indexname === 'UQ_donations_tx_hash',
    );
    expect(uniqueIndex).toBeDefined();
    expect(uniqueIndex!.indexdef).toMatch(/UNIQUE/i);
    expect(uniqueIndex!.indexdef).toMatch(/tx_hash/i);
  });

  it('rejects a duplicate tx_hash at the DB level', async () => {
    await migration.up(queryRunner);

    await insertDonation('duplicate-tx-hash-for-test');

    await expect(insertDonation('duplicate-tx-hash-for-test')).rejects.toThrow(
      /duplicate key value violates unique constraint/i,
    );
  });

  it('down() reverses the migration cleanly', async () => {
    await migration.up(queryRunner);

    await migration.down(queryRunner);

    expect(await hasDonationsTable()).toBe(false);
  });
});
