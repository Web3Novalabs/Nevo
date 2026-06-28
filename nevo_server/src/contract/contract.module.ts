import { Module } from '@nestjs/common';
import { ContractService } from './contract.service.js';
import { HorizonService } from './horizon.service.js';
import { ContractController } from './contract.controller.js';

@Module({
  controllers: [ContractController],
  providers: [ContractService, HorizonService],
  exports: [ContractService, HorizonService],
})
export class ContractModule {}
