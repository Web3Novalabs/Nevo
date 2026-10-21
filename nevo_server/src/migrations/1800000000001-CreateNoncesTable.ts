import { MigrationInterface, QueryRunner } from 'typeorm';

export class CreateNoncesTable1800000000001 implements MigrationInterface {
  name = 'CreateNoncesTable1800000000001';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `CREATE TABLE "nonces" ("id" uuid NOT NULL DEFAULT uuid_generate_v4(), "nonce" character varying(255) NOT NULL, "public_key" character varying(56) NOT NULL, "created_at" TIMESTAMP NOT NULL DEFAULT now(), "expires_at" TIMESTAMP NOT NULL, "used" boolean NOT NULL DEFAULT false, CONSTRAINT "PK_e7a29392674ca3ddd68c0418246" PRIMARY KEY ("id"))`,
    );
    await queryRunner.query(
      `CREATE UNIQUE INDEX "IDX_d4de55e1fe2897bbab03766a97" ON "nonces" ("nonce") `,
    );
    await queryRunner.query(
      `CREATE INDEX "IDX_b4f66db42abccde1b492075f4d" ON "nonces" ("public_key") `,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `DROP INDEX "public"."IDX_b4f66db42abccde1b492075f4d"`,
    );
    await queryRunner.query(
      `DROP INDEX "public"."IDX_d4de55e1fe2897bbab03766a97"`,
    );
    await queryRunner.query(`DROP TABLE "nonces"`);
  }
}
