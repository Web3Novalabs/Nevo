import { Injectable, UnauthorizedException } from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import { Keypair } from '@stellar/stellar-sdk';
import { UsersService } from '../users/users.service';
import { User } from '../users/user.entity';
import { NonceService } from './nonce.service';
import { VerifyAuthDto } from './dto/verify-auth.dto';

export interface AuthResult {
  accessToken: string;
  user: User;
}

@Injectable()
export class AuthService {
  constructor(
    private readonly jwtService: JwtService,
    private readonly usersService: UsersService,
    private readonly nonceService: NonceService,
  ) {}

  async verify(dto: VerifyAuthDto): Promise<AuthResult> {
    if (!this.verifySignature(dto.publicKey, dto.signature, dto.message)) {
      throw new UnauthorizedException('Invalid signature');
    }

    // Validate nonce from message
    const nonce = await this.nonceService.findAndValidateNonce(dto.message);
    if (!nonce) {
      throw new UnauthorizedException('Invalid or expired nonce');
    }

    // Mark nonce as used
    await this.nonceService.markNonceAsUsed(nonce.id);

    const user = await this.usersService.findOrCreate(dto.publicKey);
    const accessToken = this.jwtService.sign({
      sub: user.id,
      publicKey: user.publicKey,
      tokenVersion: user.tokenVersion,
    });

    return { accessToken, user };
  }

  async logout(publicKey: string): Promise<void> {
    const user = await this.usersService.findByPublicKey(publicKey);
    if (!user) {
      throw new UnauthorizedException('User not found');
    }

    await this.usersService.incrementTokenVersion(user.publicKey);
  }

  private verifySignature(
    publicKey: string,
    signature: string,
    message: string,
  ): boolean {
    try {
      // Verify the Stellar Ed25519 signature
      const keypair = Keypair.fromPublicKey(publicKey);
      return keypair.verify(
        Buffer.from(message),
        Buffer.from(signature, 'hex'),
      );
    } catch {
      return false;
    }
  }
}
