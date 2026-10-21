import { DataSource, QueryRunner } from 'typeorm';
import { CreatePoolsTable1782396384461 } from './1782396384461-CreatePoolsTable';

describe('CreatePoolsTable migration', () => {
  let dataSource: DataSource;
  let queryRunner: QueryRunner;

  beforeAll(async () => {
    dataSource = new DataSource({
      type: 'postgres',
      host: process.env.DB_HOST || 'localhost',
      port: parseInt(process.env.DB_PORT || '5432', 10),
      username: process.env.DB_USERNAME || 'postgres',
      password: process.env.DB_PASSWORD || 'postgres',
      database: process.env.DB_DATABASE || 'nevo_test',
      entities: [],
      migrations: [],
      synchronize: false,
    });
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
    await queryRunner.query('DELETE FROM "pools"');
  });

  it('up() creates the pools table with the unique index on contract_pool_id', async () => {
    const migration = new CreatePoolsTable1782396384461();
    await migration.up(queryRunner);

    const tableResult = await queryRunner.query(
      `SELECT EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_schema = 'public' AND table_name = 'pools'
      ) AS "exists"`,
    );
    expect(tableResult[0].exists).toBe(true);

    const indexResult = await queryRunner.query(
      `SELECT EXISTS (
        SELECT 1 FROM pg_indexes
        WHERE schemaname = 'public'
          AND tablename = 'pools'
          AND indexname = 'IDX_0487876286eddb65ed9143fc88'
      ) AS "exists"`,
    );
    expect(indexResult[0].exists).toBe(true);
  });

  it('applies default values (category: "", raised: 0, status: Active) on insert', async () => {
    const migration = new CreatePoolsTable1782396384461();
    await migration.up(queryRunner);

    await queryRunner.query(
      `INSERT INTO "pools"
        ("contract_pool_id", "title", "description", "creator_wallet", "goal")
       VALUES ($1, $2, $3, $4, $5)`,
      ['contract-pool-1', 'Test Pool', 'A test pool', 'wallet-1', 1000],
    );

    const rows = await queryRunner.query(
      `SELECT "category", "raised", "status" FROM "pools" WHERE "contract_pool_id" = $1`,
      ['contract-pool-1'],
    );

    expect(rows).toHaveLength(1);
    expect(rows[0].category).toBe('');
    expect(Number(rows[0].raised)).toBe(0);
    expect(rows[0].status).toBe('Active');
  });

  it('down() reverses cleanly, removing the pools table and pools_status_enum type', async () => {
    const migration = new CreatePoolsTable1782396384461();
    await migration.up(queryRunner);
    await migration.down(queryRunner);

    const tableResult = await queryRunner.query(
      `SELECT EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_schema = 'public' AND table_name = 'pools'
      ) AS "exists"`,
    );
    expect(tableResult[0].exists).toBe(false);

    const typeResult = await queryRunner.query(
      `SELECT EXISTS (
        SELECT 1 FROM pg_type WHERE typname = 'pools_status_enum'
      ) AS "exists"`,
    );
    expect(typeResult[0].exists).toBe(false);
  });
});
</｜｜DSML｜｜ parameter>