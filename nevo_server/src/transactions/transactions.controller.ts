import { Body, Controller, Post, UseGuards } from '@nestjs/common';
import {
  ApiBearerAuth,
  ApiOkResponse,
  ApiOperation,
  ApiTags,
} from '@nestjs/swagger';
import { IsNotEmpty, IsString } from 'class-validator';
import { ContractService } from '../contract/contract.service.js';
import { JwtAuthGuard } from '../auth/jwt-auth.guard.js';

export class SubmitXdrDto {
  @IsString()
  @IsNotEmpty()
  xdr!: string;
}

@ApiTags('transactions')
@Controller('transactions')
export class TransactionsController {
  constructor(private readonly contractService: ContractService) {}

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
  async submit(@Body() dto: SubmitXdrDto): Promise<{ txHash: string }> {
    const txHash = await this.contractService.submitSignedXdr(dto.xdr);
    return { txHash };
  }
}
