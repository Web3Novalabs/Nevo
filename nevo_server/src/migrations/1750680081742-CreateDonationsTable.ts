import { MigrationInterface, QueryRunner } from 'typeorm';

export class CreateDonationsTable1750680081742 implements MigrationInterface {
  name = 'CreateDonationsTable1750680081742';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`
      CREATE TABLE "donations" (
        "id"          uuid              NOT NULL DEFAULT uuid_generate_v4(),
        "pool_id"     character varying(255) NOT NULL,
        "donor_wallet" character varying(56) NOT NULL,
        "amount"      character varying(255) NOT NULL,
        "asset"       character varying(10)  NOT NULL,
        "tx_hash"     character varying(64)  NOT NULL,
        "memo"        text,
        "created_at"  TIMESTAMP NOT NULL DEFAULT now(),
        CONSTRAINT "PK_donations_id" PRIMARY KEY ("id")
      )
    `);
    await queryRunner.query(`
      CREATE INDEX "IDX_donations_pool_id" ON "donations" ("pool_id")
    `);
    await queryRunner.query(`
      CREATE INDEX "IDX_donations_donor_wallet" ON "donations" ("donor_wallet")
    `);
    await queryRunner.query(`
      CREATE UNIQUE INDEX "UQ_donations_tx_hash" ON "donations" ("tx_hash")
    `);
    await queryRunner.query(`
      CREATE INDEX "IDX_donations_created_at" ON "donations" ("created_at")
    `);
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`DROP INDEX "IDX_donations_created_at"`);
    await queryRunner.query(`DROP INDEX "UQ_donations_tx_hash"`);
    await queryRunner.query(`DROP INDEX "IDX_donations_donor_wallet"`);
    await queryRunner.query(`DROP INDEX "IDX_donations_pool_id"`);
    await queryRunner.query(`DROP TABLE "donations"`);
  }
}
