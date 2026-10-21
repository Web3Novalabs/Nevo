import { Controller, Get, Param, Query, Req, UseGuards } from '@nestjs/common';
import {
  ApiBearerAuth,
  ApiOkResponse,
  ApiOperation,
  ApiParam,
  ApiQuery,
  ApiTags,
} from '@nestjs/swagger';
import type { Request } from 'express';
import { JwtAuthGuard } from '../auth/jwt-auth.guard.js';
import { DonationSortBy, DonationsService } from './donations.service.js';
import { GetDonationsDto } from './dto/get-donations.dto.js';

@ApiTags('donations')
@Controller()
export class DonationsController {
  constructor(private readonly donationsService: DonationsService) {}

  @ApiOperation({ summary: 'List donations for a pool' })
  @ApiParam({ name: 'id', description: 'On-chain pool id.' })
  @ApiQuery({ name: 'page', required: false, type: Number })
  @ApiQuery({ name: 'limit', required: false, type: Number })
  @ApiQuery({ name: 'sortBy', required: false, enum: DonationSortBy })
  @ApiOkResponse({ description: 'Donations made to the pool.' })
  @Get('pools/:id/donations')
  findByPool(@Param('id') id: string, @Query() query: GetDonationsDto) {
    const sort =
      query.sortBy === DonationSortBy.largest
        ? DonationSortBy.largest
        : DonationSortBy.newest;
    return this.donationsService.findByPool(id, sort, query.page, query.limit);
  }

  @ApiOperation({ summary: 'List donations made by the authenticated user' })
  @ApiBearerAuth('bearer')
  @ApiQuery({ name: 'page', required: false, type: Number })
  @ApiQuery({ name: 'limit', required: false, type: Number })
  @ApiQuery({ name: 'sortBy', required: false, enum: DonationSortBy })
  @ApiOkResponse({ description: 'Donations made by the current user.' })
  @UseGuards(JwtAuthGuard)
  @Get('users/me/donations')
  findMyDonations(
    @Req() req: Request & { user: { publicKey: string } },
    @Query() query: GetDonationsDto,
  ) {
    const sort =
      query.sortBy === DonationSortBy.largest
        ? DonationSortBy.largest
        : DonationSortBy.newest;
    return this.donationsService.findByDonor(
      req.user.publicKey,
      sort,
      query.page,
      query.limit,
    );
  }
}
