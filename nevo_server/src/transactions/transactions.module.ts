import { Module } from '@nestjs/common';
import { ContractModule } from '../contract/contract.module.js';
import { TransactionsController } from './transactions.controller.js';

@Module({
  imports: [ContractModule],
  controllers: [TransactionsController],
})
export class TransactionsModule {}
