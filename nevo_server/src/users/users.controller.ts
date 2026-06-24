import { Controller, Get, Query, Req, UseGuards } from '@nestjs/common';
import { Request } from 'express';
import { StellarAuthGuard } from '../auth/stellar-auth.guard';
import { DonationsService } from '../donations/donations.service';

interface JwtPayload {
  sub: string;
  publicKey: string;
}

@Controller('users')
export class UsersController {
  constructor(private readonly donationsService: DonationsService) {}

  @UseGuards(StellarAuthGuard)
  @Get('me/donations')
  getMyDonations(
    @Req() req: Request & { user: JwtPayload },
    @Query('page') page = '1',
    @Query('limit') limit = '20',
  ) {
    return this.donationsService.findByDonor(
      req.user.publicKey,
      Math.max(1, parseInt(page, 10) || 1),
      Math.min(100, Math.max(1, parseInt(limit, 10) || 20)),
    );
  }
}
