import { HttpException, HttpStatus, Injectable } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { InjectRepository } from '@nestjs/typeorm';
import { LessThan, MoreThan, Repository } from 'typeorm';
import {
  ADVISORY_LOCK_KEYS,
  AdvisoryLockService,
} from '../common/advisory-lock.service';
import { Nonce } from './nonce.entity';
import * as crypto from 'crypto';

@Injectable()
export class NonceService {
  private readonly NONCE_LIFETIME_SECONDS = 300; // 5 minutes
  /** Unused, unexpired nonces a single key may hold at once. */
  static readonly MAX_OUTSTANDING_NONCES_PER_KEY = 5;

  constructor(
    @InjectRepository(Nonce)
    private readonly nonceRepository: Repository<Nonce>,
    private readonly advisoryLock: AdvisoryLockService,
  ) {}

  async generateNonce(publicKey: string): Promise<string> {
    const outstanding = await this.nonceRepository.count({
      where: { publicKey, used: false, expiresAt: MoreThan(new Date()) },
    });
    if (outstanding >= NonceService.MAX_OUTSTANDING_NONCES_PER_KEY) {
      throw new HttpException(
        'Too many outstanding challenges for this public key',
        HttpStatus.TOO_MANY_REQUESTS,
      );
    }

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

  async consumeNonce(nonce: string, publicKey: string): Promise<boolean> {
    const update = this.nonceRepository
      .createQueryBuilder()
      .update(Nonce)
      .set({ used: true })
      .where({ nonce, publicKey, used: false })
      .andWhere('expires_at > CURRENT_TIMESTAMP');
    if (
      this.nonceRepository.manager.connection.driver.isReturningSqlSupported(
        'update',
      )
    ) {
      update.returning('id');
    }
    const result = await update.execute();
    return result.affected === 1;
  }

  async deleteExpiredNonces(): Promise<void> {
    await this.advisoryLock.runExclusive(
      ADVISORY_LOCK_KEYS.nonceCleanup,
      'NonceService.cleanupExpiredNonces',
      async () => {
        await this.nonceRepository.delete({
          expiresAt: LessThan(new Date()),
        });
      },
    );
  }

  @Cron(CronExpression.EVERY_HOUR, { name: 'cleanupExpiredNonces' })
  async cleanupExpiredNonces(): Promise<void> {
    await this.deleteExpiredNonces();
  }
}
