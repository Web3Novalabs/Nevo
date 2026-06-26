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
    poolId: number,
    page: number,
    limit: number,
  ): Promise<PaginatedResult<Donation>> {
    const [data, total] = await this.donationRepo.findAndCount({
      where: { poolId },
      order: { createdAt: 'DESC' },
      skip: (page - 1) * limit,
      take: limit,
    });
    return { data, total, page, limit };
  }

  async findByDonor(
    donorWallet: string,
    page: number,
    limit: number,
  ): Promise<PaginatedResult<DonationWithPool>> {
    const [rows, total] = await this.donationRepo
      .createQueryBuilder('d')
      .leftJoin('pools', 'p', 'p.contract_pool_id = CAST(d.pool_id AS varchar)')
      .select([
        'd.id          AS id',
        'd.tx_hash      AS "txHash"',
        'd.pool_id      AS "poolId"',
        'd.donor_wallet AS "donorWallet"',
        'd.amount       AS amount',
        'd.asset        AS asset',
        'd.created_at   AS "createdAt"',
        'p.title        AS "poolTitle"',
      ])
      .where('d.donor_wallet = :donorWallet', { donorWallet })
      .orderBy('d.created_at', 'DESC')
      .offset((page - 1) * limit)
      .limit(limit)
      .getRawMany<DonationWithPool>()
      .then((data) => [data, 0] as [DonationWithPool[], number]);

    // get total count separately
    const count = await this.donationRepo.count({
      where: { donorWallet },
    });

    return { data: rows, total: count, page, limit };
  async findByPool(poolId: string, sortBy: DonationSortBy = 'newest'): Promise<Donation[]> {
    if (sortBy === 'largest') {
      return this.donationRepo
        .createQueryBuilder('d')
        .where('d.poolId = :poolId', { poolId })
        .orderBy('CAST(d.amount AS NUMERIC)', 'DESC')
        .getMany();
    }
    return this.donationRepo.find({ where: { poolId }, order: { createdAt: 'DESC' } });
  }

  async findByDonor(donorWallet: string, sortBy: DonationSortBy = 'newest'): Promise<Donation[]> {
    if (sortBy === 'largest') {
      return this.donationRepo
        .createQueryBuilder('d')
        .where('d.donorWallet = :donorWallet', { donorWallet })
        .orderBy('CAST(d.amount AS NUMERIC)', 'DESC')
        .getMany();
    }
    return this.donationRepo.find({ where: { donorWallet }, order: { createdAt: 'DESC' } });
  }
}
