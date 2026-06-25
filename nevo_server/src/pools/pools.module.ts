import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { Pool } from './pool.entity';
import { PoolsService } from './pools.service';
import { PoolsController } from './pools.controller';
import { ContractModule } from '../contract/contract.module.js';

@Module({
  imports: [TypeOrmModule.forFeature([Pool]), ContractModule],
  providers: [PoolsService],
  controllers: [PoolsController],
  exports: [PoolsService],
})
export class PoolsModule {}
