import {
  BadRequestException,
  Body,
  Controller,
  NotFoundException,
  Patch,
  Request,
  UseGuards,
} from '@nestjs/common';
import { StellarAuthGuard } from '../auth/stellar-auth.guard.js';
import { UsersService } from './users.service.js';

export interface UpdateDisplayNameDto {
  displayName: string;
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
  constructor(private readonly usersService: UsersService) {}

  @UseGuards(StellarAuthGuard)
  @Patch('me')
  async updateMe(
    @Request() req: { user: { publicKey: string } },
    @Body() dto: UpdateDisplayNameDto,
  ) {
    if (!dto.displayName || dto.displayName.length > 50) {
      throw new BadRequestException('displayName must be 1–50 characters');
    }
    const user = await this.usersService.updateDisplayName(
      req.user.publicKey,
      dto.displayName,
    );
    if (!user) throw new NotFoundException('User not found');
    return user;
  }
}
