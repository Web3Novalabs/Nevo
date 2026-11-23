import { ApiProperty } from '@nestjs/swagger';

export class DonationResponseDto {
  @ApiProperty({ format: 'uuid' })
  id: string;

  @ApiProperty({ description: 'On-chain pool id.' })
  poolId: string;

  @ApiProperty({ description: 'Stellar public key of the donor.' })
  donorWallet: string;

  @ApiProperty({ description: 'Amount in base units, as a decimal string.' })
  amount: string;

  @ApiProperty({ description: 'Display asset code.', example: 'XLM' })
  asset: string;

  @ApiProperty({
    type: String,
    nullable: true,
    description: 'Soroban token contract id (C...). Null for legacy rows.',
  })
  tokenContract: string | null;

  @ApiProperty()
  txHash: string;

  @ApiProperty({ type: String, nullable: true })
  memo: string | null;

  @ApiProperty()
  refunded: boolean;

  @ApiProperty({ type: String, format: 'date-time' })
  createdAt: Date;
}

export class PaginatedDonationsDto {
  @ApiProperty({ type: [DonationResponseDto] })
  data: DonationResponseDto[];

  @ApiProperty({ type: 'integer' })
  total: number;

  @ApiProperty({ type: 'integer' })
  page: number;

  @ApiProperty({ type: 'integer' })
  limit: number;
}
