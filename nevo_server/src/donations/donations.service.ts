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

  /**
   * Lists the donations recorded against a single pool.
   * @param poolId The pool to list donations for.
   * @param sortBy `newest` orders by creation date descending (the default);
   *   `largest` orders by donation amount descending.
   * @param page 1-based page number. Pagination is applied only when `page` or
   *   `limit` is supplied, and defaults to page 1.
   * @param limit Page size, clamped to 1-100 and defaulting to 10.
   * @returns The matching donations.
   */
  async findByPool(
    poolId: string,
    sortBy: DonationSortBy = DonationSortBy.newest,
    page?: number | string,
    limit?: number | string,
  ): Promise<Donation[]> {
    const pageNum =
      page !== undefined ? Math.max(1, parseInt(String(page), 10) || 1) : undefined;
    const limitNum =
      limit !== undefined
        ? Math.max(1, Math.min(100, parseInt(String(limit), 10) || 10))
        : undefined;

    if (sortBy === DonationSortBy.largest) {
      return this.buildLargestDonationsQuery(
        'poolId',
        poolId,
        pageNum,
        limitNum,
      ).getMany();
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
    sortBy: DonationSortBy = DonationSortBy.newest,
    page?: number | string,
    limit?: number | string,
  ): Promise<Donation[]> {
    const pageNum =
      page !== undefined ? Math.max(1, parseInt(String(page), 10) || 1) : undefined;
    const limitNum =
      limit !== undefined
        ? Math.max(1, Math.min(100, parseInt(String(limit), 10) || 10))
        : undefined;

    if (sortBy === DonationSortBy.largest) {
      return this.buildLargestDonationsQuery(
        'donorWallet',
        donorWallet,
        pageNum,
        limitNum,
      ).getMany();
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

  private buildLargestDonationsQuery(
    filterColumn: 'poolId' | 'donorWallet',
    filterValue: string,
    pageNum?: number,
    limitNum?: number,
  ) {
    const qb = this.donationRepo
      .createQueryBuilder('d')
      .where(`d.${filterColumn} = :${filterColumn}`, {
        [filterColumn]: filterValue,
      })
      .orderBy('CAST(d.amount AS NUMERIC)', 'DESC');

    if (pageNum !== undefined || limitNum !== undefined) {
      const p = pageNum ?? 1;
      const l = limitNum ?? 10;
      qb.skip((p - 1) * l).take(l);
    }

    return qb;
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
