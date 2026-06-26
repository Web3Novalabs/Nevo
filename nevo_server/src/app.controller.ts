import { Controller, Get, Req, UseGuards } from '@nestjs/common';
import { AppService } from './app.service';
import { StellarAuthGuard } from './auth/stellar-auth.guard';
import { Request } from 'express';

@Controller()
export class AppController {
  constructor(private readonly appService: AppService) {}

  @Get()
  getHello(): string {
    return this.appService.getHello();
  }

  @UseGuards(StellarAuthGuard)
  @Get('profile')
  getProfile(@Req() req: Request & { user?: { publicKey?: string } }) {
    return { publicKey: req.user?.publicKey };
  }
}
