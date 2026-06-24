import { Module } from '@nestjs/common';
import { ContractService } from './contract.service';

@Module({
  providers: [ContractService],
  exports: [ContractService],
})
export class ContractModule {}
