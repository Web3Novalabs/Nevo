import { DataSource } from 'typeorm';
import { User } from './users/user.entity';
import { Pool } from './pools/pool.entity';

export const AppDataSource = new DataSource({
  type: 'postgres',
  host: process.env.DB_HOST ?? 'localhost',
  port: parseInt(process.env.DB_PORT ?? '5432', 10),
  username: process.env.DB_USER ?? 'postgres',
  password: process.env.DB_PASSWORD ?? 'postgres',
  database: process.env.DB_NAME ?? 'nevo',
  entities: [User, Pool],
  migrations: ['src/migrations/*.ts'],
  synchronize: false,
});
