import {
  BadRequestException,
  Body,
  Controller,
  Get,
  HttpCode,
  HttpStatus,
  Post,
  Query,
  Req,
  UseGuards,
} from '@nestjs/common';
import {
  ApiBadRequestResponse,
  ApiBearerAuth,
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
import { JwtAuthGuard } from './jwt-auth.guard';

interface AuthenticatedRequest {
  user: { publicKey: string };
}

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

  @ApiBearerAuth()
  @ApiOperation({
    summary: 'Log out and revoke issued tokens',
    description:
      'Increments the token version for the authenticated user, invalidating ' +
      'all previously issued JWTs. Subsequent requests with those tokens ' +
      'will be rejected with 401.',
  })
  @ApiOkResponse({ description: 'Tokens revoked.' })
  @ApiUnauthorizedResponse({ description: 'Missing or invalid token.' })
  @UseGuards(JwtAuthGuard)
  @HttpCode(HttpStatus.OK)
  @Post('logout')
  async logout(@Req() req: AuthenticatedRequest): Promise<{ success: true }> {
    await this.authService.logout(req.user.publicKey);
    return { success: true };
  }
}
