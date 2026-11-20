import { MigrationInterface, QueryRunner } from 'typeorm';

export class AddTokenVersionToUsers1750680081742 implements MigrationInterface {
  name = 'AddTokenVersionToUsers1750680081742';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `ALTER TABLE "users" ADD "token_version" integer NOT NULL DEFAULT 0`,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`ALTER TABLE "users" DROP COLUMN "token_version"`);
  }
}
