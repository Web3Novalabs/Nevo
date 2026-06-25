import {
  Body,
  Controller,
  ForbiddenException,
  Get,
  NotFoundException,
  Param,
  Patch,
  Post,
  Request,
  UseGuards,
} from '@nestjs/common';
import { PoolsService } from './pools.service';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';

export interface CreatePoolDto {
  contractPoolId: string;
  creatorWallet: string;
  goal: string;
  title?: string;
  description?: string;
  category?: string;
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

export interface ClosePoolDto {
  requesterWallet: string;
}

@Controller('pools')
export class PoolsController {
  constructor(private readonly poolsService: PoolsService) {}

  @Get(':id')
  async findOne(@Param('id') id: string) {
    const pool = await this.poolsService.findOneMerged(id);
    if (!pool) throw new NotFoundException('Pool not found');
    return pool;
  }

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

  @UseGuards(JwtAuthGuard)
  @Post(':id/close')
  async close(
    @Param('id') id: string,
    @Request() req: { user: { publicKey: string } },
  ) {
    const pool = await this.poolsService.findByContractId(id);
    if (!pool) throw new NotFoundException('Pool not found');
    if (pool.creatorWallet !== req.user.publicKey)
      throw new ForbiddenException('Only the pool creator may close this pool');
    return this.poolsService.buildClosePoolTx(pool);
  }
}
