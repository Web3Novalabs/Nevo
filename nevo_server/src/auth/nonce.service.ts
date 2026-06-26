import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { Nonce } from './nonce.entity';
import * as crypto from 'crypto';

@Injectable()
export class NonceService {
  private readonly NONCE_LIFETIME_SECONDS = 300; // 5 minutes

  constructor(
    @InjectRepository(Nonce)
    private readonly nonceRepository: Repository<Nonce>,
  ) {}

  async generateNonce(publicKey: string): Promise<string> {
    const nonce = crypto.randomBytes(16).toString('hex');
    const expiresAt = new Date(Date.now() + this.NONCE_LIFETIME_SECONDS * 1000);

    await this.nonceRepository.save({
      nonce,
      publicKey,
      expiresAt,
      used: false,
    });

    return nonce;
  }

  async findAndValidateNonce(nonce: string): Promise<Nonce | null> {
    const foundNonce = await this.nonceRepository.findOne({
      where: { nonce },
    });

    if (!foundNonce) {
      return null;
    }

    // Check if nonce is expired
    if (new Date() > foundNonce.expiresAt) {
      await this.nonceRepository.remove(foundNonce);
      return null;
    }

    // Check if nonce was already used
    if (foundNonce.used) {
      return null;
    }

    return foundNonce;
  }

  async markNonceAsUsed(nonceId: string): Promise<void> {
    await this.nonceRepository.update(nonceId, { used: true });
  }

  async deleteExpiredNonces(): Promise<void> {
    await this.nonceRepository.delete({
      expiresAt: new Date(),
    });
  }
}
