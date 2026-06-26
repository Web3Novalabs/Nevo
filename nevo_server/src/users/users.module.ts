import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { JwtModule } from '@nestjs/jwt';
import { User } from './user.entity';
import { UsersService } from './users.service';
import { UsersController } from './users.controller';
import { DonationsModule } from '../donations/donations.module';
import { StellarAuthGuard } from '../auth/stellar-auth.guard';

@Module({
  imports: [
    TypeOrmModule.forFeature([User]),
    JwtModule.register({
      secret: process.env.JWT_SECRET ?? 'dev-secret',
      signOptions: { expiresIn: '7d' },
    }),
    DonationsModule,
  ],
  providers: [UsersService, StellarAuthGuard],
  controllers: [UsersController],
  exports: [UsersService],
})
export class UsersModule {}
