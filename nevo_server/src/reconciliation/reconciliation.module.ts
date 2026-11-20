import { Module } from '@nestjs/common';
import { ScheduleModule } from '@nestjs/schedule';
import { TypeOrmModule } from '@nestjs/typeorm';
import { AdvisoryLockService } from '../common/advisory-lock.service.js';
import { ContractModule } from '../contract/contract.module.js';
import { Pool } from '../pools/pool.entity.js';
import { AdminGuard } from './admin.guard.js';
import { ReconciliationController } from './reconciliation.controller.js';
import { ReconciliationService } from './reconciliation.service.js';

@Module({
  imports: [
    ScheduleModule.forRoot(),
    TypeOrmModule.forFeature([Pool]),
    ContractModule,
  ],
  controllers: [ReconciliationController],
  providers: [AdvisoryLockService, AdminGuard, ReconciliationService],
})
export class ReconciliationModule {}
