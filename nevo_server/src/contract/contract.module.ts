import { Module } from '@nestjs/common';
import { ContractService } from './contract.service.js';
import { HorizonService } from './horizon.service.js';

@Module({
  providers: [ContractService, HorizonService],
  exports: [ContractService, HorizonService],
})
export class ContractModule {}
