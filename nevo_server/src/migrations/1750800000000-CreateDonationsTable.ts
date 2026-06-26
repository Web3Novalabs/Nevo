import { MigrationInterface, QueryRunner } from 'typeorm';

export class CreateDonationsTable1750800000000 implements MigrationInterface {
  name = 'CreateDonationsTable1750800000000';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`
      CREATE TABLE "donations" (
        "id"           uuid                  NOT NULL DEFAULT uuid_generate_v4(),
        "tx_hash"      character varying(255) NOT NULL,
        "pool_id"      integer               NOT NULL,
        "donor_wallet" character varying(56)  NOT NULL,
        "amount"       bigint                NOT NULL,
        "asset"        character varying(56)  NOT NULL,
        "created_at"   TIMESTAMP             NOT NULL DEFAULT now(),
        CONSTRAINT "PK_donations_id" PRIMARY KEY ("id")
      )
    `);
    await queryRunner.query(`
      CREATE UNIQUE INDEX "UQ_donations_tx_hash" ON "donations" ("tx_hash")
    `);
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`DROP INDEX "UQ_donations_tx_hash"`);
    await queryRunner.query(`DROP TABLE "donations"`);
  }
}
