import { DataSource } from 'typeorm';
import { Donation } from './donation.entity';
import { DonationsService } from './donations.service';
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
});
