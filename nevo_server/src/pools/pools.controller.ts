import {
  Body,
  Controller,
  BadRequestException,
  ForbiddenException,
  Get,
  NotFoundException,
  Param,
  ParseIntPipe,
  Patch,
  Post,
  Query,
  Req,
  UseGuards,
} from '@nestjs/common';
import type { Request } from 'express';
import { PoolsService } from './pools.service.js';
import { JwtAuthGuard } from '../auth/jwt-auth.guard.js';
import { GetPoolsDto } from './dto/get-pools.dto.js';
import { DonationsService } from '../donations/donations.service.js';
import { ContractService } from '../contract/contract.service.js';
import { StellarAuthGuard } from '../auth/stellar-auth.guard.js';

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

export interface DonateDto {
  amount: number;
  tokenAddress: string;
}

interface JwtPayload {
  sub: string;
  publicKey: string;
}

@Controller('pools')
export class PoolsController {
  constructor(
    private readonly poolsService: PoolsService,
    private readonly contractService: ContractService,
    private readonly donationsService: DonationsService,
  ) {}

  @Get(':id')
  async findOne(@Param('id') id: string) {
    const pool = await this.poolsService.findOneMerged(id);
    if (!pool) throw new NotFoundException('Pool not found');
    return pool;
  }

  @Get()
  async findAll(@Query() query: GetPoolsDto) {
    return this.poolsService.findAll(query);
  }

  @Get(':id/donations')
  getDonations(
    @Param('id', ParseIntPipe) id: number,
    @Query('page') page = '1',
    @Query('limit') limit = '20',
  ) {
    return this.donationsService.findByPool(String(id));
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
    if (pool.creatorWallet !== dto.requesterWallet) {
      throw new ForbiddenException('Only the pool creator may withdraw');
    }
    return this.poolsService.buildWithdrawTx(pool);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/close')
  async close(
    @Param('id') id: string,
    @Req() req: { user: { publicKey: string } },
  ) {
    const pool = await this.poolsService.findByContractId(id);
    if (!pool) throw new NotFoundException('Pool not found');
    if (pool.creatorWallet !== req.user.publicKey) {
      throw new ForbiddenException('Only the pool creator may close this pool');
    }
    return this.poolsService.buildClosePoolTx(pool);
  }

  @UseGuards(StellarAuthGuard)
  @Post(':id/donate')
  async donate(
    @Param('id', ParseIntPipe) id: number,
    @Body() dto: DonateDto,
    @Req() req: Request & { user: JwtPayload },
  ) {
    if (!Number.isInteger(dto.amount) || dto.amount <= 0) {
      throw new BadRequestException('amount must be a positive integer');
    }

    const pool = await this.poolsService.findByContractId(String(id));
    if (!pool) throw new NotFoundException('Pool not found');

    const unsignedXdr = this.contractService.buildDonateTransaction(
      req.user.publicKey,
      id,
      String(dto.amount),
    );
    return { unsignedXdr };
  }
}
