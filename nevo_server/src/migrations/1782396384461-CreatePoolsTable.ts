import { MigrationInterface, QueryRunner } from "typeorm";

export class CreatePoolsTable1782396384461 implements MigrationInterface {
    name = 'CreatePoolsTable1782396384461'

    public async up(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`CREATE TYPE "public"."pools_status_enum" AS ENUM('Active', 'Completed')`);
        await queryRunner.query(`CREATE TABLE "pools" ("id" uuid NOT NULL DEFAULT uuid_generate_v4(), "contract_pool_id" character varying(255) NOT NULL, "title" character varying(255) NOT NULL, "description" text NOT NULL, "category" character varying(100) NOT NULL DEFAULT '', "creator_wallet" character varying(56) NOT NULL, "status" "public"."pools_status_enum" NOT NULL DEFAULT 'Active', "goal" bigint NOT NULL, "raised" bigint NOT NULL DEFAULT '0', "image_url" character varying(2048), "created_at" TIMESTAMP NOT NULL DEFAULT now(), "updated_at" TIMESTAMP NOT NULL DEFAULT now(), CONSTRAINT "PK_6708c86fc389259de3ee43230ee" PRIMARY KEY ("id"))`);
        await queryRunner.query(`CREATE UNIQUE INDEX "IDX_0487876286eddb65ed9143fc88" ON "pools" ("contract_pool_id") `);
        await queryRunner.query(`CREATE INDEX "IDX_5721b0a7eb71af11e8cea56b56" ON "pools" ("category") `);
        await queryRunner.query(`CREATE INDEX "IDX_343a2f8668b32864c2c38cc0fd" ON "pools" ("creator_wallet") `);
        await queryRunner.query(`CREATE INDEX "IDX_eeefb6d09bef12cc37ed0d2286" ON "pools" ("status") `);
        await queryRunner.query(`CREATE INDEX "IDX_1dbe6587106d0e64c981d29324" ON "pools" ("created_at") `);
    }

    public async down(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`DROP INDEX "public"."IDX_1dbe6587106d0e64c981d29324"`);
        await queryRunner.query(`DROP INDEX "public"."IDX_eeefb6d09bef12cc37ed0d2286"`);
        await queryRunner.query(`DROP INDEX "public"."IDX_343a2f8668b32864c2c38cc0fd"`);
        await queryRunner.query(`DROP INDEX "public"."IDX_5721b0a7eb71af11e8cea56b56"`);
        await queryRunner.query(`DROP INDEX "public"."IDX_0487876286eddb65ed9143fc88"`);
        await queryRunner.query(`DROP TABLE "pools"`);
        await queryRunner.query(`DROP TYPE "public"."pools_status_enum"`);
    }

}
