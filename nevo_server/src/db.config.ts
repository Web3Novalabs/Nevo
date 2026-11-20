import type { DataSourceOptions } from 'typeorm';
import { Comment } from './comments/comment.entity.js';
import { Donation } from './donations/donation.entity.js';
import { Nonce } from './auth/nonce.entity.js';
import { Pool } from './pools/pool.entity.js';
import { SyncState } from './sync/sync-state.entity.js';
import { User } from './users/user.entity.js';

const ENTITIES = [User, Pool, Donation, SyncState, Nonce, Comment];

/**
 * Builds the Postgres connection options shared by the Nest application and the
 * TypeORM CLI data source.
 *
 * `DATABASE_URL` takes precedence when set; otherwise the individual
 * `DB_HOST`/`DB_PORT`/`DB_USER`/`DB_PASSWORD`/`DB_NAME` variables are used.
 *
 * @param migrations Glob patterns locating migration files. The running app
 *   points at the compiled `dist` output while the CLI points at `src`.
 */
export function buildDataSourceOptions(
  migrations: string[],
): DataSourceOptions {
  const common = {
    type: 'postgres',
    entities: ENTITIES,
    migrations,
    synchronize: false,
  } as const;

  const url = process.env.DATABASE_URL;
  if (url) {
    return { ...common, url };
  }

  return {
    ...common,
    host: process.env.DB_HOST ?? 'localhost',
    port: parseInt(process.env.DB_PORT ?? '5432', 10),
    username: process.env.DB_USER ?? 'postgres',
    password: process.env.DB_PASSWORD ?? 'postgres',
    database: process.env.DB_NAME ?? 'nevo',
  };
}
