import { Controller, Get, HttpCode, Post, UseGuards } from '@nestjs/common';
import {
  ApiBearerAuth,
  ApiForbiddenResponse,
  ApiOkResponse,
  ApiOperation,
  ApiTags,
  ApiUnauthorizedResponse,
} from '@nestjs/swagger';
import { JwtAuthGuard } from '../auth/jwt-auth.guard.js';
import { AdminGuard } from './admin.guard.js';
import {
  ReconciliationService,
  type ReconciliationRunResult,
  type ReconciliationStats,
} from './reconciliation.service.js';

@ApiTags('admin')
@ApiBearerAuth()
@UseGuards(JwtAuthGuard, AdminGuard)
@ApiUnauthorizedResponse({ description: 'Missing or invalid JWT.' })
@ApiForbiddenResponse({ description: 'Caller is not an admin.' })
@Controller('admin/reconciliation')
export class ReconciliationController {
  constructor(private readonly reconciliation: ReconciliationService) {}

  @ApiOperation({
    summary: 'Run pool raised-total reconciliation now',
    description:
      'Compares pools.raised with on-chain get_total_raised for all active pools and corrects drift. Returns status "skipped" if a run is already in progress.',
  })
  @ApiOkResponse({ description: 'Summary of the run.' })
  @HttpCode(200)
  @Post()
  trigger(): Promise<ReconciliationRunResult> {
    return this.reconciliation.reconcile('manual');
  }

  @ApiOperation({
    summary: 'Reconciliation counters and last run',
    description:
      'Cumulative drift detected/corrected and RPC/DB error counters since process start.',
  })
  @ApiOkResponse({ description: 'Reconciliation statistics.' })
  @Get()
  status(): ReconciliationStats {
    return this.reconciliation.getStats();
  }
}
