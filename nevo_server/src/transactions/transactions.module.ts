import { Module } from '@nestjs/common';
import { ContractModule } from '../contract/contract.module.js';
import { TransactionsController } from './transactions.controller.js';

import { HistoryService } from './history.service.js';

@Module({
  providers: [HistoryService],
  exports: [HistoryService],
  imports: [ContractModule],
  controllers: [TransactionsController],
})
export class TransactionsModule {}
