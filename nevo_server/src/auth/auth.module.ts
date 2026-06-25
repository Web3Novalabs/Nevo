import { Module } from '@nestjs/common';
import { JwtModule } from '@nestjs/jwt';
import type { StringValue } from 'ms';
import { TypeOrmModule } from '@nestjs/typeorm';
import { UsersModule } from '../users/users.module';
import { AuthService } from './auth.service';
import { AuthController } from './auth.controller';
import { NonceService } from './nonce.service';
import { Nonce } from './nonce.entity';

@Module({
  imports: [
    TypeOrmModule.forFeature([Nonce]),
    JwtModule.register({
      secret: process.env.JWT_SECRET ?? 'dev-secret',
      signOptions: {
        expiresIn: (process.env.JWT_EXPIRY ?? '7d') as StringValue,
      },
    }),
    UsersModule,
  ],
  providers: [AuthService, NonceService],
  controllers: [AuthController],
})
export class AuthModule {}
