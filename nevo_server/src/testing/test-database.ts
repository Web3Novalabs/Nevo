import { DataSource, DataSourceOptions } from 'typeorm';

export const TEST_DATABASE_URL =
  process.env.TEST_DATABASE_URL ??
  'postgres://postgres:postgres@localhost:5432/nevo_test';

/**
 * Connects to the test Postgres inside a throwaway schema so specs never see
 * each other's tables. `uuid-ossp` lives in `public` and stays on the search
 * path for `uuid_generate_v4()` defaults.
 */
export async function createSchemaDataSource(
  schema: string,
  extra: Partial<DataSourceOptions> = {},
): Promise<DataSource> {
  const dataSource = new DataSource({
    type: 'postgres',
    url: TEST_DATABASE_URL,
    synchronize: false,
    ...extra,
    extra: { options: `-c search_path=${schema},public` },
  } as DataSourceOptions);
  await dataSource.initialize();
  await dataSource.query('CREATE EXTENSION IF NOT EXISTS "uuid-ossp"');
  await dataSource.query(`DROP SCHEMA IF EXISTS "${schema}" CASCADE`);
  await dataSource.query(`CREATE SCHEMA "${schema}"`);
  return dataSource;
}

export async function dropSchemaAndDestroy(
  dataSource: DataSource | undefined,
  schema: string,
): Promise<void> {
  if (dataSource?.isInitialized) {
    await dataSource.query(`DROP SCHEMA IF EXISTS "${schema}" CASCADE`);
    await dataSource.destroy();
  }
}
