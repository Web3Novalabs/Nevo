import { Controller, Get, Param, Query, Req, UseGuards } from '@nestjs/common';
import type { Request } from 'express';
import { StellarAuthGuard } from '../auth/stellar-auth.guard.js';
import { DonationSortBy, DonationsService } from './donations.service.js';

@Controller()
export class DonationsController {
  constructor(private readonly donationsService: DonationsService) {}

  @Get('pools/:id/donations')
  findByPool(@Param('id') id: string, @Query('sortBy') sortBy?: string) {
    const sort: DonationSortBy = sortBy === 'largest' ? 'largest' : 'newest';
    return this.donationsService.findByPool(id, sort);
  }

  @UseGuards(StellarAuthGuard)
  @Get('users/me/donations')
  findMyDonations(
    @Req() req: Request & { user: { publicKey: string } },
    @Query('sortBy') sortBy?: string,
  ) {
    const sort: DonationSortBy = sortBy === 'largest' ? 'largest' : 'newest';
    return this.donationsService.findByDonor(req.user.publicKey, sort);
  }
}
