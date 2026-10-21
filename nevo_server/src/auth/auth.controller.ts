import {
  BadRequestException,
  Body,
  Controller,
  Get,
  Post,
  Query,
} from '@nestjs/common';
import {
  ApiBadRequestResponse,
  ApiBody,
  ApiOkResponse,
  ApiOperation,
  ApiQuery,
  ApiTags,
  ApiUnauthorizedResponse,
} from '@nestjs/swagger';
import { Throttle } from '@nestjs/throttler';
import { AuthService } from './auth.service';
import { NonceService } from './nonce.service';
import type { AuthResult } from './auth.service';
import { VerifyAuthDto } from './dto/verify-auth.dto';

@ApiTags('auth')
@Controller('auth')
export class AuthController {
  constructor(
    private readonly authService: AuthService,
    private readonly nonceService: NonceService,
  ) {}

  @ApiOperation({
    summary: 'Request a login challenge',
    description:
      'Issues a single-use nonce for the given Stellar public key. Sign it ' +
      'with the wallet and exchange it for a JWT via POST /auth/verify.',
  })
  @ApiQuery({
    name: 'publicKey',
    required: true,
    description: 'Stellar public key (G...) of the wallet signing in.',
  })
  @ApiOkResponse({
    description: 'Nonce to be signed by the wallet.',
    schema: {
      type: 'object',
      properties: { nonce: { type: 'string' } },
    },
  })
  @ApiBadRequestResponse({ description: 'publicKey was not supplied.' })
  @Throttle({ default: { limit: 10, ttl: 60000 } })
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

  @ApiOperation({
    summary: 'Verify a signed challenge and issue a JWT',
    description:
      'Checks the signature against the previously issued nonce and returns ' +
      'an access token plus the resolved user.',
  })
  @ApiBody({
    schema: {
      type: 'object',
      required: ['publicKey', 'signature', 'message'],
      properties: {
        publicKey: {
          type: 'string',
          description: 'Stellar public key (G...) that signed the challenge.',
        },
        signature: {
          type: 'string',
          description: 'Base64-encoded signature over `message`.',
        },
        message: {
          type: 'string',
          description: 'The nonce returned by GET /auth/challenge.',
        },
      },
    },
  })
  @ApiOkResponse({
    description: 'Access token and the authenticated user.',
    schema: {
      type: 'object',
      properties: {
        accessToken: { type: 'string' },
        user: { type: 'object' },
      },
    },
  })
  @ApiUnauthorizedResponse({ description: 'Signature or nonce is invalid.' })
  @Throttle({ default: { limit: 5, ttl: 60000 } })
  @Post('verify')
  verify(@Body() dto: VerifyAuthDto): Promise<AuthResult> {
    return this.authService.verify(dto);
  }
}
