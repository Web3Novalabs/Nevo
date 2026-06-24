import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { JwtModule } from '@nestjs/jwt';
import { Pool } from './pool.entity';
import { PoolsService } from './pools.service';
import { PoolsController } from './pools.controller';
import { DonationsModule } from '../donations/donations.module';
import { ContractModule } from '../contract/contract.module';
import { StellarAuthGuard } from '../auth/stellar-auth.guard';

@Module({
  imports: [
    TypeOrmModule.forFeature([Pool]),
    JwtModule.register({
      secret: process.env.JWT_SECRET ?? 'dev-secret',
      signOptions: { expiresIn: '7d' },
    }),
    DonationsModule,
    ContractModule,
  ],
  providers: [PoolsService, StellarAuthGuard],
  controllers: [PoolsController],
  exports: [PoolsService],
})
export class PoolsModule {}
