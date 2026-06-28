import {
  Injectable,
  BadRequestException,
  UnauthorizedException,
} from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import { Keypair, StrKey } from '@stellar/stellar-sdk';
import { UsersService } from '../users/users.service';
import { User } from '../users/user.entity';
import { randomBytes } from 'crypto';
import { NonceService } from './nonce.service';

export interface VerifyDto {
  publicKey: string;
  signature: string;
  message: string;
}

export interface AuthResult {
  accessToken: string;
  user: User;
}

export interface ChallengeResult {
  nonce: string;
  expiresAt: number;
}

interface ChallengeEntry {
  nonce: string;
  expiresAt: number;
}

const CHALLENGE_TTL_MS = 5 * 60 * 1000;

@Injectable()
export class AuthService {
  private readonly challenges = new Map<string, ChallengeEntry>();

  constructor(
    private readonly jwtService: JwtService,
    private readonly usersService: UsersService,
    private readonly nonceService: NonceService,
  ) {
    setInterval(() => this.cleanupExpiredChallenges(), CHALLENGE_TTL_MS);
  }

  generateChallenge(publicKey: string): ChallengeResult {
    if (!publicKey) {
      throw new BadRequestException('publicKey is required');
    }

    if (!StrKey.isValidEd25519PublicKey(publicKey)) {
      throw new BadRequestException('Invalid Stellar public key format');
    }

    const nonce = randomBytes(32).toString('hex');
    const expiresAt = Date.now() + CHALLENGE_TTL_MS;

    this.challenges.set(publicKey, { nonce, expiresAt });

    return { nonce, expiresAt };
  }

  async verify(dto: VerifyDto): Promise<AuthResult> {
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
    });

    return { accessToken, user };
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

  private cleanupExpiredChallenges() {
    const now = Date.now();
    for (const [key, entry] of this.challenges.entries()) {
      if (now >= entry.expiresAt) {
        this.challenges.delete(key);
      }
    }
  }
}
