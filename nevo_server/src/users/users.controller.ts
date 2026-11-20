import {
  BadRequestException,
  Body,
  Controller,
  Get,
  NotFoundException,
  Param,
  Patch,
  Req,
  UseGuards,
} from '@nestjs/common';
import {
  ApiBadRequestResponse,
  ApiBearerAuth,
  ApiNotFoundResponse,
  ApiOkResponse,
  ApiOperation,
  ApiParam,
  ApiTags,
} from '@nestjs/swagger';
import type { Request } from 'express';
import { JwtAuthGuard } from '../auth/jwt-auth.guard.js';
import { UsersService } from './users.service.js';
import { UpdateDisplayNameDto } from './dto/update-display-name.dto.js';
import { MyProfileDto } from './dto/my-profile.dto.js';
import { PublicUserDto } from './dto/public-user.dto.js';
import { STELLAR_PUBLIC_KEY } from '../common/stellar-public-key.js';

@ApiTags('users')
@Controller('users')
export class UsersController {
  constructor(private readonly usersService: UsersService) {}

  @UseGuards(JwtAuthGuard)
  @Patch('me')
  async updateMe(
    @Req() req: Request & { user: { publicKey: string } },
    @Body() dto: UpdateDisplayNameDto,
  ) {
    const user = await this.usersService.updateDisplayName(
      req.user.publicKey,
      dto.displayName.trim(),
    );
    if (!user) throw new NotFoundException('User not found');
    return user;
  }

  @Get('me')
  @UseGuards(JwtAuthGuard)
  @ApiBearerAuth('bearer')
  @ApiOperation({ summary: 'Get the signed-in user profile' })
  @ApiOkResponse({ type: MyProfileDto })
  @ApiNotFoundResponse({ description: 'User not found' })
  async getMe(
    @Req() req: Request & { user: { publicKey: string } },
  ): Promise<MyProfileDto> {
    const user = await this.usersService.findByPublicKey(req.user.publicKey);
    if (!user) throw new NotFoundException('User not found');
    const { publicKey, displayName, createdAt } = user;
    return { publicKey, displayName, createdAt };
  }

  @Get(':publicKey')
  @ApiOperation({ summary: 'Get a public creator profile' })
  @ApiParam({
    name: 'publicKey',
    description: 'Stellar public key of the creator',
    example: 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF',
  })
  @ApiOkResponse({ type: PublicUserDto })
  @ApiBadRequestResponse({ description: 'Invalid public key format' })
  @ApiNotFoundResponse({ description: 'User not found' })
  async getPublicProfile(
    @Param('publicKey') publicKey: string,
  ): Promise<PublicUserDto> {
    if (!STELLAR_PUBLIC_KEY.test(publicKey)) {
      throw new BadRequestException('Invalid public key format');
    }

    const profile = await this.usersService.findPublicProfile(publicKey);
    if (!profile) throw new NotFoundException('User not found');

    return profile;
  }
}
