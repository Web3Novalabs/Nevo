import { Module } from '@nestjs/common';
import { JwtModule } from '@nestjs/jwt';
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
      signOptions: { expiresIn: '7d' },
    }),
    UsersModule,
  ],
  providers: [AuthService, NonceService],
  controllers: [AuthController],
})
export class AuthModule {}
