import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { QueryFailedError, Repository } from 'typeorm';
import { Donation } from './donation.entity.js';

export enum DonationSortBy {
  newest = 'newest',
  largest = 'largest',
}

export interface PaginatedDonations {
  data: Donation[];
  total: number;
  page: number;
  limit: number;
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
   * @param page 1-based page number, defaulting to 1.
   * @param limit Page size, clamped to 1-100 and defaulting to 10.
   * @returns A paginated result of the matching donations.
   */
  async findByPool(
    poolId: string,
    sortBy: DonationSortBy = DonationSortBy.newest,
    page?: number,
    limit?: number,
  ): Promise<PaginatedDonations> {
    return this.buildDonationQuery({ poolId }, sortBy, page, limit);
  }

  /**
   * Lists the donations recorded by a single donor.
   * @param donorWallet The donor wallet to list donations for.
   * @param sortBy `newest` orders by creation date descending (the default);
   *   `largest` orders by donation amount descending.
   * @param page 1-based page number, defaulting to 1.
   * @param limit Page size, clamped to 1-100 and defaulting to 10.
   * @returns A paginated result of the matching donations.
   */
  async findByDonor(
    donorWallet: string,
    sortBy: DonationSortBy = DonationSortBy.newest,
    page?: number,
    limit?: number,
  ): Promise<PaginatedDonations> {
    return this.buildDonationQuery({ donorWallet }, sortBy, page, limit);
  }

  private async buildDonationQuery(
    filter: { poolId?: string; donorWallet?: string },
    sortBy: DonationSortBy = DonationSortBy.newest,
    page?: number,
    limit?: number,
  ): Promise<PaginatedDonations> {
    const pageNum = Math.max(1, page ?? 1);
    const limitNum = Math.max(1, Math.min(100, limit ?? 10));

    const qb = this.donationRepo.createQueryBuilder('d');

    if (filter.poolId !== undefined) {
      qb.where('d.poolId = :poolId', { poolId: filter.poolId });
    }
    if (filter.donorWallet !== undefined) {
      qb.where('d.donorWallet = :donorWallet', {
        donorWallet: filter.donorWallet,
      });
    }

    if (sortBy === DonationSortBy.largest) {
      qb.orderBy('d.amount', 'DESC');
    } else {
      qb.orderBy('d.createdAt', 'DESC');
    }

    qb.skip((pageNum - 1) * limitNum).take(limitNum);

    const [data, total] = await qb.getManyAndCount();

    return { data, total, page: pageNum, limit: limitNum };
  }

  async isTxProcessed(txHash: string): Promise<boolean> {
    const count = await this.donationRepo.countBy({ txHash });
    return count > 0;
  }

  /**
   * Finds the donation a refund event refers to.
   * @param txHash The hash of the original donation transaction.
   * @returns The matching donation, or `null` when none is recorded.
   */
  async findByTxHash(txHash: string): Promise<Donation | null> {
    return this.donationRepo.findOneBy({ txHash });
  }

  /**
   * Marks a donation as refunded. Idempotent: refunding an already refunded
   * donation is a no-op and returns the existing record.
   * @param txHash The hash of the original donation transaction.
   * @returns The updated donation, or `null` when no donation is recorded.
   */
  async markRefunded(txHash: string): Promise<Donation | null> {
    const donation = await this.findByTxHash(txHash);
    if (!donation) {
      return null;
    }
    if (donation.refunded) {
      return donation;
    }
    donation.refunded = true;
    return this.donationRepo.save(donation);
  }

  async recordDonation(data: {
    poolId: string;
    donorWallet: string;
    amount: string;
    asset: string;
    tokenContract?: string | null;
    txHash: string;
    memo?: string | null;
  }): Promise<Donation> {
    try {
      return await this.donationRepo.save(
        this.donationRepo.create({
          poolId: data.poolId,
          donorWallet: data.donorWallet,
          amount: data.amount,
          asset: data.asset,
          tokenContract: data.tokenContract ?? null,
          txHash: data.txHash,
          memo: data.memo ?? null,
        }),
      );
    } catch (error) {
      if (
        error instanceof QueryFailedError &&
        'code' in error.driverError &&
        (error.driverError as { code?: string }).code === '23505'
      ) {
        const existing = await this.donationRepo.findOneBy({
          txHash: data.txHash,
        });
        if (existing) {
          return existing;
        }
      }
      throw error;
    }
  }
}
