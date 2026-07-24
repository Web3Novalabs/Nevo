import {
  Controller,
  DefaultValuePipe,
  Get,
  Param,
  ParseEnumPipe,
  ParseUUIDPipe,
  Query,
  Req,
  UseGuards,
} from '@nestjs/common';
import type { Request } from 'express';
import { StellarAuthGuard } from '../auth/stellar-auth.guard.js';
import { DonationSortBy, DonationsService } from './donations.service.js';

@Controller()
export class DonationsController {
  constructor(private readonly donationsService: DonationsService) {}

  @Get('pools/:id/donations')
  findByPool(
    @Param('id', new ParseUUIDPipe()) id: string,
    @Query('sortBy', new DefaultValuePipe(DonationSortBy.newest), new ParseEnumPipe(DonationSortBy))
    sortBy: DonationSortBy,
  ) {
    return this.donationsService.findByPool(id, sortBy);
  }

  @UseGuards(StellarAuthGuard)
  @Get('users/me/donations')
  findMyDonations(
    @Req() req: Request & { user: { publicKey: string } },
    @Query('sortBy', new DefaultValuePipe(DonationSortBy.newest), new ParseEnumPipe(DonationSortBy))
    sortBy: DonationSortBy,
  ) {
    return this.donationsService.findByDonor(req.user.publicKey, sortBy);
  }
}
