import {
  BadRequestException,
  Body,
  Controller,
  NotFoundException,
  Patch,
  Req,
  UseGuards,
} from '@nestjs/common';
import type { Request } from 'express';
import { StellarAuthGuard } from '../auth/stellar-auth.guard.js';
import { UsersService } from './users.service.js';

export interface UpdateDisplayNameDto {
  displayName: string;
}

@Controller('users')
export class UsersController {
  constructor(private readonly usersService: UsersService) {}

  @UseGuards(StellarAuthGuard)
  @Patch('me')
  async updateMe(
    @Req() req: Request & { user: { publicKey: string } },
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
