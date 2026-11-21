import { Controller, Get } from '@nestjs/common';
import { ApiOkResponse, ApiOperation, ApiTags } from '@nestjs/swagger';
import { StatsResponseDto } from './stats.dto.js';
import { StatsService } from './stats.service.js';

@ApiTags('stats')
@Controller('stats')
export class StatsController {
  constructor(private readonly statsService: StatsService) {}

  @ApiOperation({
    summary: 'Platform impact statistics',
    description:
      'Aggregate totals derived from the database: total donated per asset, ' +
      'active and completed pool counts and unique donor wallets. Cached for ' +
      'STATS_CACHE_TTL_MS (default 30s) so values are at most that stale.',
  })
  @ApiOkResponse({ type: StatsResponseDto })
  @Get()
  getStats(): Promise<StatsResponseDto> {
    return this.statsService.getStats();
  }
}
