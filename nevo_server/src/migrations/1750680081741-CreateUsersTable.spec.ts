import { QueryRunner } from 'typeorm';
import { CreateUsersTable1750680081741 } from './1750680081741-CreateUsersTable';

describe('CreateUsersTable1750680081741', () => {
  let migration: CreateUsersTable1750680081741;
  let queryRunner: jest.Mocked<QueryRunner>;

  beforeEach(() => {
    migration = new CreateUsersTable1750680081741();
    queryRunner = {
      query: jest.fn().mockResolvedValue(undefined),
    } as unknown as jest.Mocked<QueryRunner>;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  const getExecutedSql = (): string[] =>
    queryRunner.query.mock.calls.map((call) => call[0] as string);

  describe('up', () => {
    it('creates the users table with the expected columns and types', async () => {
      await migration.up(queryRunner);

      const sql = getExecutedSql();
      expect(sql).toHaveLength(1);

      const createTableSql = sql[0];

      // Table name
      expect(createTableSql).toMatch(/CREATE TABLE\s+"?users"?/i);

      // id column: uuid, default uuid_generate_v4()
      expect(createTableSql).toMatch(/"id"\s+uuid/i);
      expect(createTableSql).toMatch(/uuid_generate_v4\(\)/i);

      // public_key column: character varying(56), NOT NULL
      expect(createTableSql).toMatch(/"public_key"\s+character varying\(56\)\s+NOT NULL/i);

      // username column: character varying(255)
      expect(createTableSql).toMatch(/"username"\s+character varying\(255\)/i);

      // created_at column: TIMESTAMP, default now()
      expect(createTableSql).toMatch(/"created_at"\s+TIMESTAMP/i);
      expect(createTableSql).toMatch(/now\(\)/i);

      // updated_at column: TIMESTAMP, default now()
      expect(createTableSql).toMatch(/"updated_at"\s+TIMESTAMP/i);

      // Primary key constraint
      expect(createTableSql).toMatch(/CONSTRAINT\s+"PK_users_id"\s+PRIMARY KEY\s+\("id"\)/i);
    });

    it('creates the unique index UQ_users_public_key on public_key', async () => {
      await migration.up(queryRunner);

      const sql = getExecutedSql();
      expect(sql).toHaveLength(1);

      const createTableSql = sql[0];

      expect(createTableSql).toMatch(
        /CONSTRAINT\s+"UQ_users_public_key"\s+UNIQUE\s+\("public_key"\)/i,
      );
    });

    it('executes the create table query exactly once', async () => {
      await migration.up(queryRunner);

      expect(queryRunner.query).toHaveBeenCalledTimes(1);
    });
  });

  describe('down', () => {
    it('drops the index and the table in reverse order', async () => {
      await migration.down(queryRunner);

      const sql = getExecutedSql();
      expect(sql).toHaveLength(2);

      // First drop the unique index
      expect(sql[0]).toMatch(/DROP INDEX\s+"?UQ_users_public_key"?/i);

      // Then drop the table
      expect(sql[1]).toMatch(/DROP TABLE\s+"?users"?/i);
    });

    it('executes the drop queries in the correct order', async () => {
      await migration.down(queryRunner);

      expect(queryRunner.query).toHaveBeenCalledTimes(2);

      const firstCall = queryRunner.query.mock.calls[0][0] as string;
      const secondCall = queryRunner.query.mock.calls[1][0] as string;

      expect(firstCall).toMatch(/DROP INDEX/i);
      expect(secondCall).toMatch(/DROP TABLE/i);
    });
  });

  describe('round-trip', () => {
    it('does not throw when running up, down, then up again', async () => {
      await expect(migration.up(queryRunner)).resolves.not.toThrow();
      await expect(migration.down(queryRunner)).resolves.not.toThrow();
      await expect(migration.up(queryRunner)).resolves.not.toThrow();

      // up (1) + down (2) + up (1) = 4 total queries
      expect(queryRunner.query).toHaveBeenCalledTimes(4);
    });

    it('produces the same SQL for the first and second up calls', async () => {
      await migration.up(queryRunner);
      const firstUpSql = getExecutedSql();

      await migration.down(queryRunner);
      queryRunner.query.mockClear();

      await migration.up(queryRunner);
      const secondUpSql = getExecutedSql();

      expect(secondUpSql).toEqual(firstUpSql);
    });

    it('produces the same SQL for repeated down calls', async () => {
      await migration.down(queryRunner);
      const firstDownSql = getExecutedSql();

      queryRunner.query.mockClear();

      await migration.down(queryRunner);
      const secondDownSql = getExecutedSql();

      expect(secondDownSql).toEqual(firstDownSql);
    });
  });
});