import { BadRequestException, Body, Controller, Get, Post, Query } from '@nestjs/common';
import { AuthService } from './auth.service';
import { NonceService } from './nonce.service';
import type { VerifyDto, AuthResult } from './auth.service';

@Controller('auth')
export class AuthController {
  constructor(
    private readonly authService: AuthService,
    private readonly nonceService: NonceService,
  ) {}

  @Get('challenge')
  async challenge(
    @Query('publicKey') publicKey?: string,
  ): Promise<{ nonce: string }> {
    if (!publicKey) {
      throw new BadRequestException('publicKey is required');
    }

    const nonce = await this.nonceService.generateNonce(publicKey);
    return { nonce };
  }

  @Post('verify')
  verify(@Body() dto: VerifyDto): Promise<AuthResult> {
    return this.authService.verify(dto);
  }
}
