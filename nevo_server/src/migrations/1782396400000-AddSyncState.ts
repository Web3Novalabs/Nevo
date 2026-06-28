import { MigrationInterface, QueryRunner } from "typeorm";

export class AddSyncState1782396400000 implements MigrationInterface {
    name = 'AddSyncState1782396400000'

    public async up(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`CREATE TABLE "sync_state" ("key" character varying(50) NOT NULL, "value" character varying(255) NOT NULL, CONSTRAINT "PK_sync_state" PRIMARY KEY ("key"))`);
    }

    public async down(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`DROP TABLE "sync_state"`);
    }
}
