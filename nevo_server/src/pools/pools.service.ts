import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { Pool, PoolStatus } from './pool.entity.js';
import type { CreatePoolDto, UpdatePoolDto } from './pools.controller.js';
import type { GetPoolsDto } from './dto/get-pools.dto.js';
import { ContractService } from '../contract/contract.service.js';

export interface ChainPoolData {
  contractPoolId: string;
  creatorWallet: string;
  goal: string;
}

@Injectable()
export class PoolsService {
  constructor(
    @InjectRepository(Pool)
    private readonly poolRepo: Repository<Pool>,
    private readonly contractService: ContractService,
  ) {}

  async upsertFromChain(data: ChainPoolData): Promise<Pool> {
    const existing = await this.poolRepo.findOne({
      where: { contractPoolId: data.contractPoolId },
    });

    if (existing) {
      existing.creatorWallet = data.creatorWallet;
      existing.goal = data.goal;
      return this.poolRepo.save(existing);
    }

    return this.poolRepo.save(
      this.poolRepo.create({
        contractPoolId: data.contractPoolId,
        creatorWallet: data.creatorWallet,
        goal: data.goal,
        title: '',
        description: '',
        category: '',
        status: PoolStatus.Active,
        raised: '0',
        imageUrl: null,
      }),
    );
  }

  async findAll(query: GetPoolsDto): Promise<{
    data: Pool[];
    total: number;
    page: number;
    limit: number;
  }> {
    const page = query.page ? Math.max(1, parseInt(query.page, 10)) : 1;
    const limit = query.limit ? Math.max(1, parseInt(query.limit, 10)) : 10;
    const skip = (page - 1) * limit;

    const queryBuilder = this.poolRepo.createQueryBuilder('pool');

    if (query.category) {
      queryBuilder.andWhere('LOWER(pool.category) = LOWER(:category)', {
        category: query.category,
      });
    }

    if (query.status) {
      const normalizedStatus =
        query.status.charAt(0).toUpperCase() +
        query.status.slice(1).toLowerCase();
      queryBuilder.andWhere('pool.status = :status', {
        status: normalizedStatus as PoolStatus,
      });
    }

    if (query.search) {
      queryBuilder.andWhere(
        '(LOWER(pool.title) LIKE LOWER(:search) OR LOWER(pool.description) LIKE LOWER(:search))',
        { search: `%${query.search}%` },
      );
    }

    queryBuilder.orderBy('pool.createdAt', 'DESC').skip(skip).take(limit);

    const [data, total] = await queryBuilder.getManyAndCount();

    return {
      data,
      total,
      page,
      limit,
    };
  }

  async create(dto: CreatePoolDto): Promise<Pool> {
    return this.poolRepo.save(
      this.poolRepo.create({
        contractPoolId: dto.contractPoolId,
        creatorWallet: dto.creatorWallet,
        goal: dto.goal,
        title: dto.title ?? '',
        description: dto.description ?? '',
        category: dto.category ?? '',
        status: PoolStatus.Active,
        raised: '0',
        imageUrl: dto.imageUrl ?? null,
      }),
    );
  }

  async updateMeta(
    contractPoolId: string,
    dto: UpdatePoolDto,
  ): Promise<Pool | null> {
    const pool = await this.poolRepo.findOne({ where: { contractPoolId } });
    if (!pool) return null;
    if (dto.description !== undefined) pool.description = dto.description;
    if (dto.imageUrl !== undefined) pool.imageUrl = dto.imageUrl;
    if (dto.category !== undefined) pool.category = dto.category;
    return this.poolRepo.save(pool);
  }

  async findByContractId(contractPoolId: string): Promise<Pool | null> {
    return this.poolRepo.findOne({ where: { contractPoolId } });
  }

  async findOneMerged(contractPoolId: string) {
    const pool = await this.poolRepo.findOne({ where: { contractPoolId } });
    if (!pool) return null;

    const poolIdNum = parseInt(contractPoolId, 10);
    let raisedOnChain = '0';
    let closedOnChain = false;
    let donorCount = 0;

    if (!Number.isNaN(poolIdNum)) {
      const [poolOnChain, totalRaisedOnChain, donorCountOnChain] =
        await Promise.all([
          this.contractService.getPoolOnChain(poolIdNum),
          this.contractService.getTotalRaisedOnChain(poolIdNum),
          this.contractService.getDonorCountOnChain(poolIdNum),
        ]);

      if (poolOnChain) {
        raisedOnChain = poolOnChain.collected.toString();
        closedOnChain = poolOnChain.closed;
      } else if (totalRaisedOnChain) {
        raisedOnChain = totalRaisedOnChain.toString();
      }

      if (donorCountOnChain) {
        donorCount = donorCountOnChain;
      }
    }

    return {
      ...pool,
      raisedOnChain,
      closedOnChain,
      donorCount,
    };
  }

  async markCompleted(contractPoolId: string): Promise<Pool | null> {
    const pool = await this.poolRepo.findOne({ where: { contractPoolId } });
    if (!pool) return null;
    pool.status = PoolStatus.Completed;
    return this.poolRepo.save(pool);
  }

  buildWithdrawTx(pool: Pool): { unsignedXdr: string; poolId: string } {
    // TODO: replace with real Stellar transaction build calling contract.withdraw (#657)
    return { unsignedXdr: 'placeholder_xdr', poolId: pool.contractPoolId };
  }

  buildClosePoolTx(pool: Pool): { unsignedXdr: string } {
    const poolIdNum = parseInt(pool.contractPoolId, 10);
    const unsignedXdr = this.contractService.buildClosePoolTransaction(
      pool.creatorWallet,
      poolIdNum,
    );
    return { unsignedXdr };
  }
}
