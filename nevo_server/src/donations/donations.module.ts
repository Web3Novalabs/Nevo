import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { Donation } from './donation.entity.js';
import { DonationsService } from './donations.service.js';
import { DonationsController } from './donations.controller.js';
import { ContractModule } from '../contract/contract.module.js';

@Module({
  imports: [TypeOrmModule.forFeature([Donation]), ContractModule],
  providers: [DonationsService],
  controllers: [DonationsController],
  exports: [DonationsService],
})
export class DonationsModule {}
