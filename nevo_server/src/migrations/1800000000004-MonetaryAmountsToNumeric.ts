import { MigrationInterface, QueryRunner } from 'typeorm';

const BIGINT_MAX = '9223372036854775807';

export class MonetaryAmountsToNumeric1800000000004 implements MigrationInterface {
  name = 'MonetaryAmountsToNumeric1800000000004';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `ALTER TABLE "pools" ALTER COLUMN "goal" TYPE numeric(39,0) USING "goal"::numeric(39,0)`,
    );
    await queryRunner.query(`ALTER TABLE "pools" ALTER COLUMN "raised" DROP DEFAULT`);
    await queryRunner.query(
      `ALTER TABLE "pools" ALTER COLUMN "raised" TYPE numeric(39,0) USING "raised"::numeric(39,0)`,
    );
    await queryRunner.query(`ALTER TABLE "pools" ALTER COLUMN "raised" SET DEFAULT 0`);
    await queryRunner.query(
      `ALTER TABLE "donations" ALTER COLUMN "amount" TYPE numeric(39,0) USING "amount"::numeric(39,0)`,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    const [overflow] = await queryRunner.query(
      `SELECT count(*)::int AS n FROM "pools" WHERE "goal" > ${BIGINT_MAX} OR "raised" > ${BIGINT_MAX}`,
    );
    if (overflow.n > 0) {
      throw new Error(
        `Cannot revert: ${overflow.n} pool(s) hold goal/raised values above bigint max (${BIGINT_MAX})`,
      );
    }
    await queryRunner.query(
      `ALTER TABLE "donations" ALTER COLUMN "amount" TYPE character varying(255) USING "amount"::text`,
    );
    await queryRunner.query(`ALTER TABLE "pools" ALTER COLUMN "raised" DROP DEFAULT`);
    await queryRunner.query(
      `ALTER TABLE "pools" ALTER COLUMN "raised" TYPE bigint USING "raised"::bigint`,
    );
    await queryRunner.query(`ALTER TABLE "pools" ALTER COLUMN "raised" SET DEFAULT '0'`);
    await queryRunner.query(
      `ALTER TABLE "pools" ALTER COLUMN "goal" TYPE bigint USING "goal"::bigint`,
    );
  }
}
