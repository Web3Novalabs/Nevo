import { Controller, Get, Query } from '@nestjs/common';
import { PoolsService } from './pools.service';
import { GetPoolsDto } from './dto/get-pools.dto';

@Controller('pools')
export class PoolsController {
  constructor(private readonly poolsService: PoolsService) {}

  @Get()
  async findAll(@Query() query: GetPoolsDto) {
    return this.poolsService.findAll(query);
  }
}
