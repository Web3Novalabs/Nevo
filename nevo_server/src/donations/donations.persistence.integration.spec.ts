import { DataSource } from 'typeorm';
import { Donation } from './donation.entity';
import { DonationSortBy, DonationsService } from './donations.service';
import { UpdateDonationsAssetLength1800000000000 } from '../migrations/1800000000000-UpdateDonationsAssetLength';

const databaseUrl = process.env.TEST_DATABASE_URL;
const describeWithDatabase = databaseUrl ? describe : describe.skip;

describeWithDatabase('Donations persistence (Postgres)', () => {
  let dataSource: DataSource;
  let service: DonationsService;

  beforeAll(async () => {
    dataSource = new DataSource({
      type: 'postgres',
      url: databaseUrl,
      entities: [Donation],
      synchronize: false,
    });
    await dataSource.initialize();
    await dataSource.query(`
      CREATE TABLE "donations" (
        "id" uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
        "pool_id" character varying(255) NOT NULL,
        "donor_wallet" character varying(56) NOT NULL,
        "amount" character varying(255) NOT NULL,
        "asset" character varying(10) NOT NULL,
        "tx_hash" character varying(64) NOT NULL UNIQUE,
        "memo" text,
        "created_at" TIMESTAMP NOT NULL DEFAULT now()
      )
    `);

    const queryRunner = dataSource.createQueryRunner();
    await new UpdateDonationsAssetLength1800000000000().up(queryRunner);
    await queryRunner.release();

    service = new DonationsService(dataSource.getRepository(Donation));
  }, 30000);

  afterEach(async () => {
    await dataSource.query('TRUNCATE TABLE "donations"');
  });

  afterAll(async () => {
    if (dataSource?.isInitialized) {
      await dataSource.query('DROP TABLE "donations"');
      await dataSource.destroy();
    }
  });

  const record = (asset: string, txHash: string) =>
    service.recordDonation({
      poolId: 'pool-1',
      donorWallet: 'GDONOR',
      amount: '100',
      asset,
      txHash,
    });

  it('persists a 12-character asset code after the migration', async () => {
    const donation = await record('ABCDEFGHIJKL', 'tx-asset-12');

    expect(donation.asset).toBe('ABCDEFGHIJKL');
  });

  it('rejects a 13-character asset code at the database level', async () => {
    await expect(record('ABCDEFGHIJKLM', 'tx-asset-13')).rejects.toMatchObject({
      driverError: { code: '22001' },
    });
  });

  it('preserves lowercase and mixed-case asset codes', async () => {
    const donation = await record('aBc123', 'tx-asset-case');

    expect(donation.asset).toBe('aBc123');
  });

  describe('paginated queries', () => {
    const seed = async () => {
      for (let i = 0; i < 5; i++) {
        await service.recordDonation({
          poolId: 'pool-1',
          donorWallet: 'GDONOR',
          amount: '100',
          asset: 'XLM',
          txHash: `tx-pool-${i}`,
        });
      }
      for (let i = 0; i < 3; i++) {
        await service.recordDonation({
          poolId: 'pool-2',
          donorWallet: 'GOTHER',
          amount: '50',
          asset: 'XLM',
          txHash: `tx-other-${i}`,
        });
      }
    };

    it('findByPool returns { data, total, page, limit } with the full total', async () => {
      await seed();

      const result = await service.findByPool(
        'pool-1',
        DonationSortBy.newest,
        1,
        2,
      );

      expect(result.total).toBe(5);
      expect(result.page).toBe(1);
      expect(result.limit).toBe(2);
      expect(result.data).toHaveLength(2);
      expect(result.data.every((d) => d.poolId === 'pool-1')).toBe(true);
    });

    it('findByPool paginates through the correct window', async () => {
      await seed();

      const first = await service.findByPool(
        'pool-1',
        DonationSortBy.newest,
        1,
        2,
      );
      const second = await service.findByPool(
        'pool-1',
        DonationSortBy.newest,
        2,
        2,
      );
      const third = await service.findByPool(
        'pool-1',
        DonationSortBy.newest,
        3,
        2,
      );

      expect(first.data).toHaveLength(2);
      expect(second.data).toHaveLength(2);
      expect(third.data).toHaveLength(1);
      expect(third.total).toBe(5);
      expect(third.page).toBe(3);
      expect(third.limit).toBe(2);

      const ids = new Set(
        [...first.data, ...second.data, ...third.data].map((d) => d.id),
      );
      expect(ids.size).toBe(5);
    });

    it('findByDonor returns { data, total, page, limit } with the full total', async () => {
      await seed();

      const result = await service.findByDonor(
        'GDONOR',
        DonationSortBy.newest,
        1,
        2,
      );

      expect(result.total).toBe(5);
      expect(result.page).toBe(1);
      expect(result.limit).toBe(2);
      expect(result.data).toHaveLength(2);
      expect(result.data.every((d) => d.donorWallet === 'GDONOR')).toBe(true);
    });

    it('findByDonor paginates through the correct window', async () => {
      await seed();

      const first = await service.findByDonor(
        'GDONOR',
        DonationSortBy.newest,
        1,
        2,
      );
      const second = await service.findByDonor(
        'GDONOR',
        DonationSortBy.newest,
        2,
        2,
      );
      const third = await service.findByDonor(
        'GDONOR',
        DonationSortBy.newest,
        3,
        2,
      );

      expect(first.data).toHaveLength(2);
      expect(second.data).toHaveLength(2);
      expect(third.data).toHaveLength(1);
      expect(third.total).toBe(5);

      const ids = new Set(
        [...first.data, ...second.data, ...third.data].map((d) => d.id),
      );
      expect(ids.size).toBe(5);
    });
  });
});
