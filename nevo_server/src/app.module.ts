import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ScheduleModule } from '@nestjs/schedule';
import { AppController } from './app.controller.js';
import { AppService } from './app.service.js';
import { SyncModule } from './sync/sync.module.js';
import { Pool } from './pools/pool.entity.js';
import { PoolsModule } from './pools/pools.module.js';
import { User } from './users/user.entity.js';

@Module({
  imports: [
    ConfigModule.forRoot({ isGlobal: true }),
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
