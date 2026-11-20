import {
  Body,
  Controller,
  Get,
  Post,
  Query,
  Req,
  UseGuards,
} from '@nestjs/common';
import {
  ApiBearerAuth,
  ApiOkResponse,
  ApiOperation,
  ApiTags,
} from '@nestjs/swagger';
import { ContractService } from '../contract/contract.service.js';
import { SubmitXdrDto } from '../contract/contract.controller.js';
import { JwtAuthGuard } from '../auth/jwt-auth.guard.js';

import { HistoryPageDto, HistoryQueryDto } from './history.dto.js';
import { HistoryService } from './history.service.js';

@ApiTags('transactions')
@Controller('transactions')
export class TransactionsController {
  constructor(
    private readonly contractService: ContractService,
    private readonly history: HistoryService,
  ) {}

  @Get()
  @UseGuards(JwtAuthGuard)
  @ApiBearerAuth('bearer')
  @ApiOperation({ summary: 'List the signed-in wallet’s transactions' })
  @ApiOkResponse({ type: HistoryPageDto })
  list(
    @Req() req: { user: { publicKey: string } },
    @Query() query: HistoryQueryDto,
  ) {
    return this.history.list(req.user.publicKey, query);
  }

  @ApiOperation({
    summary: 'Submit a signed transaction',
    description:
      'Submits a signed XDR to the Stellar network and returns the transaction hash.',
  })
  @ApiBearerAuth('bearer')
  @ApiOkResponse({
    description: 'Transaction hash of the submitted XDR.',
    schema: {
      type: 'object',
      properties: { txHash: { type: 'string' } },
    },
  })
  @UseGuards(JwtAuthGuard)
  @Post('submit')
  async submit(
    @Body() dto: SubmitXdrDto,
    @Req() req: { user: { publicKey: string } },
  ): Promise<{ txHash: string }> {
    const entry = this.contractService.describeSubmission(
      dto.xdr,
      req.user.publicKey,
    );
    const txHash = await this.contractService.submitSignedXdr(dto.xdr);
    if (entry)
      await this.history.record(req.user.publicKey, {
        ...entry,
        txHash,
        status: 'pending',
      });
    return { txHash };
  }
}
