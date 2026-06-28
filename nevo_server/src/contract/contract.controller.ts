import { Controller, Post, Body, UseGuards } from '@nestjs/common';
import { ContractService } from './contract.service.js';
import { SubmitXdrDto } from './dto/submit-xdr.dto.js';
import { JwtAuthGuard } from '../auth/jwt-auth.guard.js';

@Controller('contract')
export class ContractController {
  constructor(private readonly contractService: ContractService) {}

  @Post('submit')
  @UseGuards(JwtAuthGuard)
  async submitXdr(@Body() dto: SubmitXdrDto) {
    const txHash = await this.contractService.submitSignedXdr(dto.signedXdr);
    return { txHash };
  }
}
