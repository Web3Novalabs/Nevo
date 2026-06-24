import { Module } from '@nestjs/common';
import { PoolsController } from './pools.controller';
import { PoolsService } from './pools.service';
import { MockPoolRepository } from './pools.repository';

@Module({
  controllers: [PoolsController],
  providers: [PoolsService, MockPoolRepository],
  exports: [PoolsService],
})
export class PoolsModule {}
