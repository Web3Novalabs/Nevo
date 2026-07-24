import { DataSource } from 'typeorm';
import { Donation } from './donations/donation.entity.js';
import { Pool } from './pools/pool.entity.js';
import { User } from './users/user.entity.js';
import { Nonce } from './auth/nonce.entity.js';
import { SyncState } from './sync/sync-state.entity.js';

export const AppDataSource = new DataSource({
  type: 'postgres',
  host: process.env.DB_HOST ?? 'localhost',
  port: parseInt(process.env.DB_PORT ?? '5432', 10),
  username: process.env.DB_USER ?? 'postgres',
  password: process.env.DB_PASSWORD ?? 'postgres',
  database: process.env.DB_NAME ?? 'nevo',
  entities: [User, Pool, Donation, SyncState, Nonce],
  migrations: ['src/migrations/*.ts'],
  synchronize: false,
});
