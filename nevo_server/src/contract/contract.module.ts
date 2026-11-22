import { Module } from '@nestjs/common';
import { ContractController } from './contract.controller.js';
import { ContractService } from './contract.service.js';
import { HorizonService } from './horizon.service.js';
import { StellarConfig } from './stellar.config.js';

@Module({
  controllers: [ContractController],
  providers: [StellarConfig, ContractService, HorizonService],
  exports: [StellarConfig, ContractService, HorizonService],
})
export class ContractModule {}
