import { Body, Controller, Get, Post, Query } from '@nestjs/common';
import { AuthService } from './auth.service';
import type { VerifyDto, AuthResult, ChallengeResult } from './auth.service';

@Controller('auth')
export class AuthController {
  constructor(private readonly authService: AuthService) {}

  @Get('challenge')
  challenge(@Query('publicKey') publicKey: string): ChallengeResult {
    return this.authService.generateChallenge(publicKey);
  }

  @Post('verify')
  verify(@Body() dto: VerifyDto): Promise<AuthResult> {
    return this.authService.verify(dto);
  }
}
