import { MigrationInterface, QueryRunner } from 'typeorm';

export class CreateNoncesTable1795065600000 implements MigrationInterface {
  name = 'CreateNoncesTable1795065600000';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`
      CREATE TABLE "nonces" (
        "id"          uuid              NOT NULL DEFAULT uuid_generate_v4(),
        "nonce"       character varying(255) NOT NULL,
        "public_key"  character varying(56) NOT NULL,
        "created_at"  TIMESTAMP         NOT NULL DEFAULT now(),
        "expires_at"  TIMESTAMP         NOT NULL,
        "used"        boolean           NOT NULL DEFAULT false,
        CONSTRAINT "PK_nonces_id" PRIMARY KEY ("id")
      )
    `);
    await queryRunner.query(`
      CREATE UNIQUE INDEX "UQ_nonces_nonce" ON "nonces" ("nonce")
    `);
    await queryRunner.query(`
      CREATE INDEX "IDX_nonces_public_key" ON "nonces" ("public_key")
    `);
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`DROP INDEX "IDX_nonces_public_key"`);
    await queryRunner.query(`DROP INDEX "UQ_nonces_nonce"`);
    await queryRunner.query(`DROP TABLE "nonces"`);
  }
}
