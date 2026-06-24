import {
  Body,
  Controller,
  ForbiddenException,
  NotFoundException,
  Param,
  Patch,
  Post,
} from '@nestjs/common';
import { PoolsService } from './pools.service';

export interface CreatePoolDto {
  contractPoolId: string;
  creatorWallet: string;
  goal: string;
  title?: string;
  description?: string;
  imageUrl?: string;
}

export interface UpdatePoolDto {
  description?: string;
  imageUrl?: string;
  category?: string;
}

export interface WithdrawDto {
  requesterWallet: string;
}

@Controller('pools')
export class PoolsController {
  constructor(private readonly poolsService: PoolsService) {}

  @Post()
  create(@Body() dto: CreatePoolDto) {
    return this.poolsService.create(dto);
  }

  @Patch(':id')
  async updateMeta(@Param('id') id: string, @Body() dto: UpdatePoolDto) {
    const pool = await this.poolsService.updateMeta(id, dto);
    if (!pool) throw new NotFoundException('Pool not found');
    return pool;
  }

  @Post(':id/withdraw')
  async withdraw(@Param('id') id: string, @Body() dto: WithdrawDto) {
    const pool = await this.poolsService.findByContractId(id);
    if (!pool) throw new NotFoundException('Pool not found');
    if (pool.creatorWallet !== dto.requesterWallet)
      throw new ForbiddenException('Only the pool creator may withdraw');
    return this.poolsService.buildWithdrawTx(pool);
  }
}
