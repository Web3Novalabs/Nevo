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
  ): Promise<Donation[]> {
    if (sortBy === 'largest') {
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
    sortBy: DonationSortBy = 'newest',
  ): Promise<Donation[]> {
    if (sortBy === 'largest') {
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
}
