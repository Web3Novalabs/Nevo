import { Injectable, BadRequestException } from '@nestjs/common';
import { StrKey } from '@stellar/stellar-sdk';

@Injectable()
export class AuthService {
  validatePublicKey(publicKey: string): void {
    if (!publicKey || !StrKey.isValidEd25519PublicKey(publicKey)) {
      throw new BadRequestException('Invalid Stellar public key');
    }
  }
}
