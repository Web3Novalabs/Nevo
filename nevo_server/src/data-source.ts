import { DataSource } from 'typeorm';
import { buildDataSourceOptions } from './db.config.js';

export const AppDataSource = new DataSource(
  buildDataSourceOptions(['src/migrations/!(*.spec).ts']),
);
