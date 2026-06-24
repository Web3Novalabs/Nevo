import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ScheduleModule } from '@nestjs/schedule';
import { SyncModule } from './sync/sync.module.js';
import { Pool } from './pools/pool.entity';
import { PoolsModule } from './pools/pools.module';

@Module({
  imports: [
    TypeOrmModule.forRoot({
      type: 'postgres',
      host: process.env.DB_HOST ?? 'localhost',
      port: parseInt(process.env.DB_PORT ?? '5432', 10),
      username: process.env.DB_USER ?? 'postgres',
      password: process.env.DB_PASSWORD ?? 'postgres',
      database: process.env.DB_NAME ?? 'nevo',
      entities: [User, Pool],
      migrations: ['dist/migrations/*.js'],
      synchronize: false,
    }),
    ScheduleModule.forRoot(),
    SyncModule,
    PoolsModule,
  ],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
