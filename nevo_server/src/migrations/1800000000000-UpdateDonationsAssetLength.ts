import { MigrationInterface, QueryRunner } from 'typeorm';

export class UpdateDonationsAssetLength1800000000000 implements MigrationInterface {
  name = 'UpdateDonationsAssetLength1800000000000';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`ALTER TABLE "donations" ALTER COLUMN "asset" TYPE character varying(12)`);
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`ALTER TABLE "donations" ALTER COLUMN "asset" TYPE character varying(10)`);
  }
}
