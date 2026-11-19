import { DataSource, QueryRunner } from 'typeorm';
import { AddSyncState1782396400000 } from './1782396400000-AddSyncState';

describe('AddSyncState migration', () => {
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
    // Ensure the table is dropped between tests so each test starts clean.
    await queryRunner.query('DROP TABLE IF EXISTS "sync_state"');
  });

  it('creates the sync_state table with the columns the SyncState entity expects', async () => {
    const migration = new AddSyncState1782396400000();

    await migration.up(queryRunner);

    const table = await queryRunner.getTable('sync_state');
    expect(table).toBeDefined();
    expect(table!.name).toBe('sync_state');

    const keyColumn = table!.findColumnByName('key');
    expect(keyColumn).toBeDefined();
    expect(keyColumn!.type).toBe('varchar');
    expect(keyColumn!.length).toBe('50');
    expect(keyColumn!.isPrimary).toBe(true);
    expect(keyColumn!.isNullable).toBe(false);

    const valueColumn = table!.findColumnByName('value');
    expect(valueColumn).toBeDefined();
    expect(valueColumn!.type).toBe('varchar');
    expect(valueColumn!.length).toBe('255');
    expect(valueColumn!.isNullable).toBe(false);
  });

  it('drops the sync_state table on down', async () => {
    const migration = new AddSyncState1782396400000();

    await migration.up(queryRunner);
    await migration.down(queryRunner);

    const table = await queryRunner.getTable('sync_state');
    expect(table).toBeUndefined();
  });
});