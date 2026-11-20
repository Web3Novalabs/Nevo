import { Module } from '@nestjs/common';
import { ScheduleModule } from '@nestjs/schedule';
import { TypeOrmModule } from '@nestjs/typeorm';
import { AdvisoryLockService } from '../common/advisory-lock.service.js';
import { PoolsModule } from '../pools/pools.module.js';
import { DonationsModule } from '../donations/donations.module.js';
import { SyncService } from './sync.service.js';
import { SyncState } from './sync-state.entity.js';

import { TransactionsModule } from '../transactions/transactions.module.js';

@Module({
  imports: [
    ScheduleModule.forRoot(),
    PoolsModule,
    DonationsModule,
    TransactionsModule,
    TypeOrmModule.forFeature([SyncState]),
  ],
  providers: [AdvisoryLockService, SyncService],
  exports: [SyncService],
})
export class SyncModule {}
