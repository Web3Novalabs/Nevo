import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { Donation } from './donation.entity.js';

export enum DonationSortBy {
  newest = 'newest',
  largest = 'largest',
}

@Injectable()
export class DonationsService {
  constructor(
    @InjectRepository(Donation)
    private readonly donationRepo: Repository<Donation>,
  ) {}

  async findByPool(
    poolId: string,
    sortBy: DonationSortBy = DonationSortBy.newest,
  ): Promise<Donation[]> {
    if (sortBy === DonationSortBy.largest) {
      return this.donationRepo
        .createQueryBuilder('d')
        .where('d.poolId = :poolId', { poolId })
        .orderBy('CAST(d.amount AS NUMERIC)', 'DESC')
        .getMany();
    }
    return this.donationRepo.find({
      where: { poolId },
      order: { createdAt: 'DESC' },
    });
  }

  async findByDonor(
    donorWallet: string,
    sortBy: DonationSortBy = DonationSortBy.newest,
  ): Promise<Donation[]> {
    if (sortBy === DonationSortBy.largest) {
      return this.donationRepo
        .createQueryBuilder('d')
        .where('d.donorWallet = :donorWallet', { donorWallet })
        .orderBy('CAST(d.amount AS NUMERIC)', 'DESC')
        .getMany();
    }
    return this.donationRepo.find({
      where: { donorWallet },
      order: { createdAt: 'DESC' },
    });
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
