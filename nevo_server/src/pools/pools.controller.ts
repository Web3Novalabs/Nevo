import {
  Body, ValidationPipe,
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
import {
  ApiBearerAuth,
  ApiCreatedResponse,
  ApiForbiddenResponse,
  ApiNotFoundResponse,
  ApiOkResponse,
  ApiOperation,
  ApiParam,
  ApiTags,
} from '@nestjs/swagger';
import type { Request } from 'express';
import { PoolsService } from './pools.service.js';
import { JwtAuthGuard } from '../auth/jwt-auth.guard.js';
import { ContractService } from '../contract/contract.service.js';
import { CreatePoolDto } from './dto/create-pool.dto.js';
import { DonatePoolDto } from './dto/donate-pool.dto.js';
import { FilterPoolsDto } from './dto/filter-pools.dto.js';
import { UpdatePoolDto } from './dto/update-pool.dto.js';

interface JwtPayload {
  sub: string;
  publicKey: string;
}

@ApiTags('pools')
@Controller('pools')
export class PoolsController {
  constructor(
    private readonly poolsService: PoolsService,
    private readonly contractService: ContractService,
  ) {}

  @ApiOperation({
    summary: 'Get a single pool',
    description:
      'Returns the stored pool merged with live on-chain state ' +
      '(raisedOnChain, closedOnChain, donorCount).',
  })
  @ApiParam({ name: 'id', description: 'On-chain pool id.' })
  @ApiOkResponse({ description: 'The pool, including on-chain state.' })
  @ApiNotFoundResponse({ description: 'No pool with that id.' })
  @Get(':id')
  async findOne(@Param('id') id: string) {
    const pool = await this.poolsService.findOneMerged(id);
    if (!pool) throw new NotFoundException('Pool not found');
    return pool;
  }

  @ApiOperation({
    summary: 'List pools',
    description:
      'Paginated list of pools, newest first. Optionally filtered by ' +
      'category, status or a free-text search over title and description.',
  })
  @ApiOkResponse({
    description: 'Paginated pools.',
    schema: {
      type: 'object',
      properties: {
        data: { type: 'array', items: { type: 'object' } },
        total: { type: 'integer' },
        page: { type: 'integer' },
        limit: { type: 'integer' },
      },
    },
  })
  @Get()
  async findAll(@Query() query: FilterPoolsDto) {
    return this.poolsService.findAll(query);
  }

  @ApiOperation({
    summary: 'Create a pool record',
    description:
      'Stores the off-chain record for a pool that already exists on-chain.',
  })
  @ApiCreatedResponse({ description: 'The created pool.' })
  @Post()
  create(@Body(new ValidationPipe({ transform: true, whitelist: true })) dto: CreatePoolDto) {
    return this.poolsService.create(dto);
  }

  @ApiOperation({
    summary: 'Update off-chain pool metadata',
    description:
      'Updates description, image and category. On-chain fields such as the ' +
      'goal and creator cannot be changed here.',
  })
  @ApiParam({ name: 'id', description: 'On-chain pool id.' })
  @ApiOkResponse({ description: 'The updated pool.' })
  @ApiNotFoundResponse({ description: 'No pool with that id.' })
  @Patch(':id')
  async updateMeta(@Param('id') id: string, @Body() dto: UpdatePoolDto) {
    const pool = await this.poolsService.updateMeta(id, dto);
    if (!pool) throw new NotFoundException('Pool not found');
    return pool;
  }

  @ApiOperation({
    summary: 'Build a withdrawal transaction',
    description:
      'Returns an unsigned XDR withdrawing the pool balance. The requesting ' +
      'wallet is taken from the JWT, and must be the pool creator.',
  })
  @ApiBearerAuth('bearer')
  @ApiParam({ name: 'id', description: 'On-chain pool id.' })
  @ApiOkResponse({
    description: 'Unsigned transaction for the creator to sign and submit.',
    schema: {
      type: 'object',
      properties: {
        unsignedXdr: { type: 'string' },
        poolId: { type: 'string' },
      },
    },
  })
  @ApiNotFoundResponse({ description: 'No pool with that id.' })
  @ApiForbiddenResponse({
    description: 'Authenticated wallet is not the pool creator.',
  })
  @UseGuards(JwtAuthGuard)
  @Post(':id/withdraw')
  async withdraw(
    @Param('id') id: string,
    @Req() req: { user: { publicKey: string } },
  ) {
    const pool = await this.poolsService.findByContractId(id);
    if (!pool) throw new NotFoundException('Pool not found');
    if (pool.creatorWallet !== req.user.publicKey) {
      throw new ForbiddenException('Only the pool creator may withdraw');
    }
    return this.poolsService.buildWithdrawTx(pool);
  }

  @ApiOperation({
    summary: 'Build a close-pool transaction',
    description:
      'Returns an unsigned XDR closing the pool. Only the authenticated pool ' +
      'creator may close it.',
  })
  @ApiBearerAuth('bearer')
  @ApiParam({ name: 'id', description: 'On-chain pool id.' })
  @ApiOkResponse({
    description: 'Unsigned transaction for the creator to sign and submit.',
    schema: {
      type: 'object',
      properties: { unsignedXdr: { type: 'string' } },
    },
  })
  @ApiNotFoundResponse({ description: 'No pool with that id.' })
  @ApiForbiddenResponse({
    description: 'Authenticated wallet is not the pool creator.',
  })
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

  @ApiOperation({
    summary: 'Build a donation transaction',
    description:
      'Returns an unsigned XDR donating to the pool from the authenticated ' +
      "wallet. The server never signs; the caller submits it with the wallet's key.",
  })
  @ApiBearerAuth('bearer')
  @ApiParam({ name: 'id', description: 'On-chain pool id (numeric).' })
  @ApiOkResponse({
    description: 'Unsigned transaction for the donor to sign and submit.',
    schema: {
      type: 'object',
      properties: { unsignedXdr: { type: 'string' } },
    },
  })
  @ApiNotFoundResponse({ description: 'No pool with that id.' })
  @UseGuards(JwtAuthGuard)
  @Post(':id/donate')
  async donate(
    @Param('id', ParseIntPipe) id: number,
    @Body() dto: DonatePoolDto,
    @Req() req: Request & { user: JwtPayload },
  ) {
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
