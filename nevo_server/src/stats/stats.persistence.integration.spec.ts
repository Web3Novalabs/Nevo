import { ConfigService } from '@nestjs/config';
import { DataSource, QueryRunner } from 'typeorm';
import { CreateDonationsTable1750680081742 } from '../migrations/1750680081742-CreateDonationsTable';
import { CreatePoolsTable1782396384461 } from '../migrations/1782396384461-CreatePoolsTable';
import { UpdateDonationsAssetLength1800000000000 } from '../migrations/1800000000000-UpdateDonationsAssetLength';
import { StatsService } from './stats.service';

const databaseUrl = process.env.TEST_DATABASE_URL;
const describeWithDatabase = databaseUrl ? describe : describe.skip;
const SCHEMA = 'it_1716_stats';

describeWithDatabase('StatsService (Postgres)', () => {
  let dataSource: DataSource;
  let service: StatsService;

  const insertPool = (id: string, status: string) =>
    dataSource.query(
      `INSERT INTO pools (contract_pool_id, title, description, creator_wallet, status, goal)
       VALUES ($1, 't', 'd', 'GCREATOR', $2, 1000)`,
      [id, status],
    );
  const donate = (tx: string, donor: string, amount: string, asset: string) =>
    dataSource.query(
      `INSERT INTO donations (pool_id, donor_wallet, amount, asset, tx_hash)
       VALUES ('1', $1, $2, $3, $4)`,
      [donor, amount, asset, tx],
    );

  beforeAll(async () => {
    dataSource = new DataSource({
      type: 'postgres',
      url: databaseUrl,
      synchronize: false,
      extra: { options: `-c search_path=${SCHEMA},public` },
    });
    await dataSource.initialize();
    await dataSource.query(`DROP SCHEMA IF EXISTS ${SCHEMA} CASCADE`);
    await dataSource.query(`CREATE SCHEMA ${SCHEMA}`);
    await dataSource.query('CREATE EXTENSION IF NOT EXISTS "uuid-ossp"');

    const runner = dataSource.createQueryRunner();
    await runner.connect();
    // The pools migration schema-qualifies its enum with "public"; keep this spec isolated.
    const scoped = {
      query: (sql: string, params?: unknown[]) =>
        runner.query(sql.replaceAll('"public".', `"${SCHEMA}".`), params),
    } as unknown as QueryRunner;
    await new CreatePoolsTable1782396384461().up(scoped);
    await new CreateDonationsTable1750680081742().up(scoped);
    await new UpdateDonationsAssetLength1800000000000().up(scoped);
    await runner.release();

    service = new StatsService(dataSource, {
      get: () => '0',
    } as unknown as ConfigService);
  }, 30000);

  afterEach(async () => {
    await dataSource.query('TRUNCATE pools, donations');
  });

  afterAll(async () => {
    if (dataSource?.isInitialized) {
      await dataSource.query(`DROP SCHEMA IF EXISTS ${SCHEMA} CASCADE`);
      await dataSource.destroy();
    }
  });

  it('returns zeros for an empty database', async () => {
    await expect(service.getStats()).resolves.toEqual({
      totalRaised: [],
      activePools: 0,
      completedPools: 0,
      uniqueDonors: 0,
    });
  });

  it('aggregates known fixtures exactly', async () => {
    await insertPool('1', 'Active');
    await insertPool('2', 'Active');
    await insertPool('3', 'Completed');
    await donate('t1', 'GA', '9007199254740993', 'XLM');
    await donate('t2', 'GA', '7', 'XLM');
    await donate('t3', 'GB', '170141183460469231731687303715884105727', 'USDC');
    await donate('t4', 'GC', '170141183460469231731687303715884105727', 'USDC');

    await expect(service.getStats()).resolves.toEqual({
      totalRaised: [
        { asset: 'USDC', amount: '340282366920938463463374607431768211454' },
        { asset: 'XLM', amount: '9007199254741000' },
      ],
      activePools: 2,
      completedPools: 1,
      uniqueDonors: 3,
    });
  });
});
