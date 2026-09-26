import {
  Body,
  Controller,
  Get,
  Param,
  Post,
  UsePipes,
  ValidationPipe,
} from '@nestjs/common';
import { PoolsService } from './pools.service';
import { DonatePoolDto } from './dto/donate-pool.dto';

@Controller('pools')
export class PoolsController {
  constructor(private readonly poolsService: PoolsService) {}

  @Get()
  findAll() {
    return this.poolsService.findAll();
  }

  @Get(':id')
  findOne(@Param('id') id: string) {
    return this.poolsService.findOne(id);
  }

  @Post(':id/donate')
  @UsePipes(new ValidationPipe({ whitelist: true, forbidNonWhitelisted: true }))
  donate(@Param('id') id: string, @Body() dto: DonatePoolDto) {
    return this.poolsService.donate(id, dto);
  }
}
