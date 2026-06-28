import { Module } from '@nestjs/common';
import { ScheduleModule } from '@nestjs/schedule';
import { TypeOrmModule } from '@nestjs/typeorm';
import { PoolsModule } from '../pools/pools.module.js';
import { SyncService } from './sync.service.js';
import { SyncState } from './sync-state.entity.js';

@Module({
  imports: [ScheduleModule.forRoot(), PoolsModule, TypeOrmModule.forFeature([SyncState])],
  providers: [SyncService],
  exports: [SyncService],
})
export class SyncModule {}
