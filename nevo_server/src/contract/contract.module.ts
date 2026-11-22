import { Module } from '@nestjs/common';
import { ContractController } from './contract.controller.js';
import { ContractService } from './contract.service.js';
import { StellarConfig } from './stellar.config.js';

@Module({
  controllers: [ContractController],
  providers: [StellarConfig, ContractService],
  exports: [StellarConfig, ContractService],
})
export class ContractModule {}
