import { Injectable } from '@nestjs/common';
import { GetPoolsDto } from './dto/get-pools.dto';
import { MockPoolRepository } from './pools.repository';
import { Pool } from './entities/pool.entity';

@Injectable()
export class PoolsService {
  constructor(private readonly poolsRepository: MockPoolRepository) {}

  async findAll(query: GetPoolsDto): Promise<{
    data: Pool[];
    total: number;
    page: number;
    limit: number;
  }> {
    const page = query.page ? Math.max(1, parseInt(query.page, 10)) : 1;
    const limit = query.limit ? Math.max(1, parseInt(query.limit, 10)) : 10;

    // Standard skip conversion
    const skip = (page - 1) * limit;

    const [data, total] = await this.poolsRepository.findAndCount({
      where: {
        search: query.search,
        category: query.category,
        status: query.status,
      },
      order: { createdAt: 'DESC' },
      take: limit,
      skip,
    });

    return {
      data,
      total,
      page,
      limit,
    };
  }
}
