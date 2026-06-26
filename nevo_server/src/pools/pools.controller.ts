import {
  BadRequestException,
  Body,
  Controller,
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
import { Request } from 'express';
import { PoolsService } from './pools.service';
import { GetPoolsDto } from './dto/get-pools.dto';
import { DonationsService } from '../donations/donations.service';
import { ContractService } from '../contract/contract.service';
import { StellarAuthGuard } from '../auth/stellar-auth.guard';

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
    private readonly donationsService: DonationsService,
    private readonly contractService: ContractService,
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

  @Get(':id/donations')
  getDonations(
    @Param('id', ParseIntPipe) id: number,
    @Query('page') page = '1',
    @Query('limit') limit = '20',
  ) {
    return this.donationsService.findByPool(
      id,
      Math.max(1, parseInt(page, 10) || 1),
      Math.min(100, Math.max(1, parseInt(limit, 10) || 20)),
    );
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