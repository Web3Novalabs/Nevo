import { Controller, Get, Query, BadRequestException } from '@nestjs/common';
import { StrKey } from '@stellar/stellar-sdk';
import { NonceService } from './nonce.service';

@Controller('auth')
export class AuthController {
  constructor(private readonly nonceService: NonceService) {}

  @Get('challenge')
  async challenge(@Query('publicKey') publicKey: string) {
    if (!publicKey) {
      throw new BadRequestException('publicKey is required');
    }

    if (!StrKey.isValidEd25519PublicKey(publicKey)) {
      throw new BadRequestException('Invalid Stellar public key');
    }

    const nonce = await this.nonceService.generateNonce(publicKey);
    return { nonce };
  }
}
