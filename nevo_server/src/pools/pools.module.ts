import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { Pool } from './pool.entity';
import { PoolsService } from './pools.service';
import { PoolsController } from './pools.controller';

@Module({
  imports: [TypeOrmModule.forFeature([Pool])],
  providers: [PoolsService],
  controllers: [PoolsController],
  exports: [PoolsService],
})
export class PoolsModule {}
