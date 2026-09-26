import { Body, Controller, Post, UseGuards } from '@nestjs/common';
import {
  ApiBearerAuth,
  ApiOkResponse,
  ApiOperation,
  ApiTags,
} from '@nestjs/swagger';
import { IsNotEmpty, IsString } from 'class-validator';
import { ContractService } from './contract.service.js';
import { JwtAuthGuard } from '../auth/jwt-auth.guard.js';

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
  async submitXdr(@Body() dto: SubmitXdrDto): Promise<{ txHash: string }> {
    const txHash = await this.contractService.submitSignedXdr(dto.xdr);
    return { txHash };
  }
}
