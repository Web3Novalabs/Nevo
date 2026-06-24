import { Module } from '@nestjs/common';
import { ScheduleModule } from '@nestjs/schedule';
import { PoolsModule } from '../pools/pools.module';
import { SyncService } from './sync.service';

@Module({
  imports: [ScheduleModule.forRoot(), PoolsModule],
  providers: [SyncService],
  exports: [SyncService],
})
export class SyncModule {}
