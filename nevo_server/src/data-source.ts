import { DataSource } from 'typeorm';
import { Donation } from './donations/donation.entity';
import { Pool } from './pools/pool.entity';
import { User } from './users/user.entity';
import { Nonce } from './auth/nonce.entity';
import { SyncState } from './sync/sync-state.entity';

export const AppDataSource = new DataSource({
  type: 'postgres',
  host: process.env.DB_HOST ?? 'localhost',
  port: parseInt(process.env.DB_PORT ?? '5432', 10),
  username: process.env.DB_USER ?? 'postgres',
  password: process.env.DB_PASSWORD ?? 'postgres',
  database: process.env.DB_NAME ?? 'nevo',
  entities: [User, Pool, Donation, SyncState],
  migrations: ['src/migrations/*.ts'],
  synchronize: false,
});
