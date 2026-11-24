import { DataSource, QueryRunner } from 'typeorm';
import { CreateDonationsTable1750680081742 } from './1750680081742-CreateDonationsTable';
import { CreatePoolsTable1782396384461 } from './1782396384461-CreatePoolsTable';
import { UpdateDonationsAssetLength1800000000000 } from './1800000000000-UpdateDonationsAssetLength';
import { MonetaryAmountsToNumeric1800000000004 } from './1800000000004-MonetaryAmountsToNumeric';
import { AddPoolApplicationDeadline1800000000007 } from './1800000000007-AddPoolApplicationDeadline';
import { Pool } from '../pools/pool.entity';
import { Donation } from '../donations/donation.entity';
import {
  DonationSortBy,
  DonationsService,
} from '../donations/donations.service';

const databaseUrl = process.env.TEST_DATABASE_URL;
const describeWithDatabase = databaseUrl ? describe : describe.skip;

const SCHEMA = 'mig_1710_numeric';
const I128_MAX = '170141183460469231731687303715884105727';
const BIGINT_MAX = '9223372036854775807';
const BEYOND_BIGINT = '9223372036854775808';

describeWithDatabase('MonetaryAmountsToNumeric migration (Postgres)', () => {
  let dataSource: DataSource;
  let queryRunner: QueryRunner;
  const migration = new MonetaryAmountsToNumeric1800000000004();

  const columnInfo = async (table: string, column: string) => {
    const [row] = await queryRunner.query(
      `SELECT data_type, numeric_precision, numeric_scale, character_maximum_length, column_default
       FROM information_schema.columns
       WHERE table_schema = $1 AND table_name = $2 AND column_name = $3`,
      [SCHEMA, table, column],
    );
    return row;
  };

  const insertPool = (contractPoolId: string, goal: string, raised: string) =>
    queryRunner.query(
      `INSERT INTO pools (contract_pool_id, title, description, creator_wallet, goal, raised)
       VALUES ($1, 't', 'd', 'GCREATOR', $2, $3)`,
      [contractPoolId, goal, raised],
    );

  const insertDonation = (txHash: string, amount: string) =>
    queryRunner.query(
      `INSERT INTO donations (pool_id, donor_wallet, amount, asset, tx_hash)
       VALUES ('1', 'GDONOR', $1, 'XLM', $2)`,
      [amount, txHash],
    );

  const resetSchema = async () => {
    await queryRunner.query(`DROP SCHEMA IF EXISTS ${SCHEMA} CASCADE`);
    await queryRunner.query(`CREATE SCHEMA ${SCHEMA}`);
    await queryRunner.query(`SET search_path TO ${SCHEMA}, public`);
    await queryRunner.query('CREATE EXTENSION IF NOT EXISTS "uuid-ossp"');
    // The pools migration schema-qualifies its enum with "public"; redirect it
    // so this spec stays isolated from other specs sharing the test database.
    const scoped = {
      query: (sql: string, params?: unknown[]) =>
        queryRunner.query(sql.replaceAll('"public".', `"${SCHEMA}".`), params),
    } as unknown as QueryRunner;
    await new CreatePoolsTable1782396384461().up(scoped);
    await new CreateDonationsTable1750680081742().up(scoped);
    await new UpdateDonationsAssetLength1800000000000().up(scoped);
  };

  beforeAll(async () => {
    dataSource = new DataSource({
      type: 'postgres',
      url: databaseUrl,
      entities: [Pool, Donation],
      synchronize: false,
    });
    await dataSource.initialize();
    queryRunner = dataSource.createQueryRunner();
    await queryRunner.connect();
  }, 30000);

  beforeEach(async () => {
    await resetSchema();
  });

  afterAll(async () => {
    if (queryRunner) {
      await queryRunner.query('SET search_path TO public');
      await queryRunner.query(`DROP SCHEMA IF EXISTS ${SCHEMA} CASCADE`);
      await queryRunner.release();
    }
    if (dataSource?.isInitialized) {
      await dataSource.destroy();
    }
  });

  describe('up', () => {
    it('converts goal, raised and amount to numeric(39,0)', async () => {
      await migration.up(queryRunner);

      for (const [table, column] of [
        ['pools', 'goal'],
        ['pools', 'raised'],
        ['donations', 'amount'],
      ]) {
        const info = await columnInfo(table, column);
        expect(info.data_type).toBe('numeric');
        expect(info.numeric_precision).toBe(39);
        expect(info.numeric_scale).toBe(0);
      }
    });

    it('keeps raised defaulting to 0', async () => {
      await migration.up(queryRunner);

      await queryRunner.query(
        `INSERT INTO pools (contract_pool_id, title, description, creator_wallet, goal)
         VALUES ('default-raised', 't', 'd', 'GCREATOR', 5)`,
      );
      const [row] = await queryRunner.query(
        `SELECT raised::text AS raised FROM pools WHERE contract_pool_id = 'default-raised'`,
      );
      expect(row.raised).toBe('0');
    });

    it('preserves existing values exactly, including bigint and varchar edge cases', async () => {
      await insertPool('p1', BIGINT_MAX, '0');
      await insertPool('p2', '1', '9007199254740993');
      await insertDonation('tx1', '9007199254740993');
      await insertDonation('tx2', BIGINT_MAX);
      await insertDonation('tx3', '0');

      await migration.up(queryRunner);

      const pools = await queryRunner.query(
        `SELECT contract_pool_id, goal::text AS goal, raised::text AS raised FROM pools ORDER BY contract_pool_id`,
      );
      expect(pools).toEqual([
        { contract_pool_id: 'p1', goal: BIGINT_MAX, raised: '0' },
        { contract_pool_id: 'p2', goal: '1', raised: '9007199254740993' },
      ]);
      const donations = await queryRunner.query(
        `SELECT tx_hash, amount::text AS amount FROM donations ORDER BY tx_hash`,
      );
      expect(donations).toEqual([
        { tx_hash: 'tx1', amount: '9007199254740993' },
        { tx_hash: 'tx2', amount: BIGINT_MAX },
        { tx_hash: 'tx3', amount: '0' },
      ]);
    });
  });

  describe('down', () => {
    it('restores bigint / bigint / varchar(255)', async () => {
      await migration.up(queryRunner);
      await migration.down(queryRunner);

      expect((await columnInfo('pools', 'goal')).data_type).toBe('bigint');
      const raised = await columnInfo('pools', 'raised');
      expect(raised.data_type).toBe('bigint');
      expect(raised.column_default).toContain('0');
      const amount = await columnInfo('donations', 'amount');
      expect(amount.data_type).toBe('character varying');
      expect(amount.character_maximum_length).toBe(255);
    });

    it('preserves representable values through up then down', async () => {
      await insertPool('p1', BIGINT_MAX, '12345');
      await insertDonation('tx1', '9007199254740993');

      await migration.up(queryRunner);
      await migration.down(queryRunner);

      const [pool] = await queryRunner.query(
        `SELECT goal::text AS goal, raised::text AS raised FROM pools`,
      );
      expect(pool).toEqual({ goal: BIGINT_MAX, raised: '12345' });
      const [donation] = await queryRunner.query(
        `SELECT amount FROM donations`,
      );
      expect(donation.amount).toBe('9007199254740993');
    });

    it('refuses to revert when a pool value exceeds bigint, leaving data intact', async () => {
      await migration.up(queryRunner);
      await insertPool('big', BEYOND_BIGINT, '0');

      await expect(migration.down(queryRunner)).rejects.toThrow(
        /above bigint max/,
      );

      expect((await columnInfo('pools', 'goal')).data_type).toBe('numeric');
      const [pool] = await queryRunner.query(
        `SELECT goal::text AS goal FROM pools WHERE contract_pool_id = 'big'`,
      );
      expect(pool.goal).toBe(BEYOND_BIGINT);
    });

    it('can run up again after down', async () => {
      await migration.up(queryRunner);
      await migration.down(queryRunner);
      await migration.up(queryRunner);

      expect((await columnInfo('pools', 'raised')).data_type).toBe('numeric');
    });
  });

  describe('i128 round-trip', () => {
    beforeEach(async () => {
      await migration.up(queryRunner);
      await new AddPoolApplicationDeadline1800000000007().up(queryRunner);
    });

    it('stores and returns i128::MAX exactly through the entities', async () => {
      const poolRepo = queryRunner.manager.getRepository(Pool);
      const donationRepo = queryRunner.manager.getRepository(Donation);

      const pool = await poolRepo.save(
        poolRepo.create({
          contractPoolId: 'i128',
          title: 't',
          description: 'd',
          category: '',
          creatorWallet: 'GCREATOR',
          goal: I128_MAX,
          raised: I128_MAX,
          imageUrl: null,
        }),
      );
      const donation = await donationRepo.save(
        donationRepo.create({
          poolId: 'i128',
          donorWallet: 'GDONOR',
          amount: I128_MAX,
          asset: 'XLM',
          txHash: 'tx-i128',
          memo: null,
        }),
      );

      const loadedPool = await poolRepo.findOneByOrFail({ id: pool.id });
      const loadedDonation = await donationRepo.findOneByOrFail({
        id: donation.id,
      });

      expect(typeof loadedPool.goal).toBe('string');
      expect(loadedPool.goal).toBe(I128_MAX);
      expect(loadedPool.raised).toBe(I128_MAX);
      expect(typeof loadedDonation.amount).toBe('string');
      expect(loadedDonation.amount).toBe(I128_MAX);
    });

    it('rejects values wider than 39 digits', async () => {
      await expect(
        insertDonation('tx-wide', '1' + '0'.repeat(39)),
      ).rejects.toMatchObject({ code: '22003' });
    });

    it('sorts largest donations numerically without a cast', async () => {
      await insertDonation('tx-a', '9');
      await insertDonation('tx-b', I128_MAX);
      await insertDonation('tx-c', '100');
      await insertDonation('tx-d', BEYOND_BIGINT);

      const service = new DonationsService(
        queryRunner.manager.getRepository(Donation),
      );
      const result = await service.findByPool('1', DonationSortBy.largest);

      expect(result.data.map((d) => d.amount)).toEqual([
        I128_MAX,
        BEYOND_BIGINT,
        '100',
        '9',
      ]);
    });
  });
});
