import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { Donation } from './donation.entity.js';

export type DonationSortBy = 'newest' | 'largest';

@Injectable()
export class DonationsService {
  constructor(
    @InjectRepository(Donation)
    private readonly donationRepo: Repository<Donation>,
  ) {}

  async findByPool(
    poolId: string,
    sortBy: DonationSortBy = 'newest',
    page?: number | string,
    limit?: number | string,
  ): Promise<Donation[]> {
    const pageNum = page !== undefined ? Math.max(1, parseInt(String(page), 10) || 1) : undefined;
    const limitNum =
      limit !== undefined
        ? Math.max(1, Math.min(100, parseInt(String(limit), 10) || 10))
        : undefined;

    if (sortBy === 'largest') {
      const qb = this.donationRepo
        .createQueryBuilder('d')
        .where('d.poolId = :poolId', { poolId })
        .orderBy('CAST(d.amount AS NUMERIC)', 'DESC');

      if (pageNum !== undefined || limitNum !== undefined) {
        const p = pageNum ?? 1;
        const l = limitNum ?? 10;
        qb.skip((p - 1) * l).take(l);
      }
      return qb.getMany();
    }

    const findOptions: any = {
      where: { poolId },
      order: { createdAt: 'DESC' },
    };

    if (pageNum !== undefined || limitNum !== undefined) {
      const p = pageNum ?? 1;
      const l = limitNum ?? 10;
      findOptions.skip = (p - 1) * l;
      findOptions.take = l;
    }

    return this.donationRepo.find(findOptions);
  }

  async findByDonor(
    donorWallet: string,
    sortBy: DonationSortBy = 'newest',
    page?: number | string,
    limit?: number | string,
  ): Promise<Donation[]> {
    const pageNum = page !== undefined ? Math.max(1, parseInt(String(page), 10) || 1) : undefined;
    const limitNum =
      limit !== undefined
        ? Math.max(1, Math.min(100, parseInt(String(limit), 10) || 10))
        : undefined;

    if (sortBy === 'largest') {
      const qb = this.donationRepo
        .createQueryBuilder('d')
        .where('d.donorWallet = :donorWallet', { donorWallet })
        .orderBy('CAST(d.amount AS NUMERIC)', 'DESC');

      if (pageNum !== undefined || limitNum !== undefined) {
        const p = pageNum ?? 1;
        const l = limitNum ?? 10;
        qb.skip((p - 1) * l).take(l);
      }
      return qb.getMany();
    }

    const findOptions: any = {
      where: { donorWallet },
      order: { createdAt: 'DESC' },
    };

    if (pageNum !== undefined || limitNum !== undefined) {
      const p = pageNum ?? 1;
      const l = limitNum ?? 10;
      findOptions.skip = (p - 1) * l;
      findOptions.take = l;
    }

    return this.donationRepo.find(findOptions);
  }

  async isTxProcessed(txHash: string): Promise<boolean> {
    const count = await this.donationRepo.countBy({ txHash });
    return count > 0;
  }

  async recordDonation(data: {
    poolId: string;
    donorWallet: string;
    amount: string;
    asset: string;
    txHash: string;
    memo?: string;
  }): Promise<Donation> {
    return this.donationRepo.save(
      this.donationRepo.create({
        poolId: data.poolId,
        donorWallet: data.donorWallet,
        amount: data.amount,
        asset: data.asset,
        txHash: data.txHash,
        memo: data.memo || null,
      }),
    );
  }
}
