import { MigrationInterface, QueryRunner } from 'typeorm';

export class CreateTransactions1800000000003 implements MigrationInterface {
  async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`CREATE TABLE transactions (
      wallet varchar(56) NOT NULL,
      tx_hash varchar(64) NOT NULL,
      type varchar(20) NOT NULL,
      status varchar(10) NOT NULL,
      amount varchar(255) NOT NULL,
      recipient varchar(255) NOT NULL,
      checked_at timestamptz,
      created_at timestamptz NOT NULL DEFAULT now(),
      PRIMARY KEY (wallet, tx_hash, type)
    )`);
    await queryRunner.query(
      'CREATE INDEX transactions_wallet_date ON transactions (wallet, created_at DESC)',
    );
  }

  async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query('DROP TABLE transactions');
  }
}
