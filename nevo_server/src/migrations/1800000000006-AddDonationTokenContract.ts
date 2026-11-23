import { MigrationInterface, QueryRunner } from 'typeorm';

export class AddDonationTokenContract1800000000006 implements MigrationInterface {
  name = 'AddDonationTokenContract1800000000006';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `ALTER TABLE "donations" ADD COLUMN "token_contract" character varying(56)`,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `ALTER TABLE "donations" DROP COLUMN "token_contract"`,
    );
  }
}
