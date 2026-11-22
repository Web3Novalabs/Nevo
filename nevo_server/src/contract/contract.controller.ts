import {
  Body,
  Controller,
  Get,
  Param,
  Post,
  UseGuards,
} from '@nestjs/common';
import {
  ApiBearerAuth,
  ApiOkResponse,
  ApiOperation,
  ApiTags,
} from '@nestjs/swagger';
import { IsNotEmpty, IsString } from 'class-validator';
import { ContractService, ContractSubmissionResult } from './contract.service.js';
import { JwtAuthGuard } from '../auth/jwt-auth.guard.js';

// Shared with TransactionsController so both XDR submit endpoints validate identically.
export class SubmitXdrDto {
  @IsString()
  @IsNotEmpty()
  xdr!: string;
}

@ApiTags('contract')
@Controller('contract')
export class ContractController {
  constructor(private readonly contractService: ContractService) {}

  @ApiOperation({
    summary: 'Submit a signed contract transaction',
    description:
      'Submits a signed XDR to the Stellar network and returns the transaction hash plus its terminal status (completed, failed or pending).',
  })
  @ApiBearerAuth('bearer')
  @ApiOkResponse({
    description: 'Transaction hash and status of the submitted transaction.',
    schema: {
      type: 'object',
      properties: {
        txHash: { type: 'string' },
        status: { type: 'string', enum: ['completed', 'failed', 'pending'] },
        returnValue: { type: 'object' },
      },
      required: ['txHash', 'status'],
    },
  })
  @UseGuards(JwtAuthGuard)
  @Post('submit')
  async submitXdr(@Body() dto: SubmitXdrDto): Promise<ContractSubmissionResult> {
    return this.contractService.submitSignedXdr(dto.xdr);
  }

  @ApiOperation({
    summary: 'Query the on-chain status of a submitted transaction',
  })
  @ApiBearerAuth('bearer')
  @ApiOkResponse({
    description:
      'Current status of the transaction (completed, failed or pending).',
    schema: {
      type: 'object',
      properties: {
        status: { type: 'string', enum: ['completed', 'failed', 'pending'] },
      },
      required: ['status'],
    },
  })
  @UseGuards(JwtAuthGuard)
  @Get('status/:txHash')
  async getTransactionStatus(
    @Param('txHash') txHash: string,
  ): Promise<{ status: 'completed' | 'failed' | 'pending' }> {
    return {
      status:
        (await this.contractService.getSubmissionStatus(txHash)) ?? 'pending',
    };
  }
}
