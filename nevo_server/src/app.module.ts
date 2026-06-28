import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ScheduleModule } from '@nestjs/schedule';
import { AppController } from './app.controller.js';
import { AppService } from './app.service.js';
import { AuthModule } from './auth/auth.module.js';
import { ContractModule } from './contract/contract.module.js';
import { Donation } from './donations/donation.entity.js';
import { DonationsModule } from './donations/donations.module.js';
import { Pool } from './pools/pool.entity.js';
import { PoolsModule } from './pools/pools.module.js';
import { SyncModule } from './sync/sync.module.js';
import { SyncState } from './sync/sync-state.entity.js';

import { TransactionsModule } from './transactions/transactions.module.js';
import { User } from './users/user.entity.js';
import { UsersModule } from './users/users.module.js';

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
      entities: [User, Pool, Donation, SyncState],
      migrations: ['dist/migrations/*.js'],
      synchronize: false,
    }),
    ScheduleModule.forRoot(),
    AuthModule,
    ContractModule,
    DonationsModule,
    SyncModule,
    TransactionsModule,
    PoolsModule,
    UsersModule,
  ],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
