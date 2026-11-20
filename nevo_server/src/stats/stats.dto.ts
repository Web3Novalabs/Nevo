import { ApiProperty } from '@nestjs/swagger';

export class AssetTotalDto {
  @ApiProperty({ example: 'XLM', description: 'Asset code of the donations.' })
  asset: string;

  @ApiProperty({
    example: '170141183460469231731687303715884105727',
    description:
      'Sum of all donation amounts for the asset in base units, as an exact integer string.',
  })
  amount: string;
}

export class StatsResponseDto {
  @ApiProperty({
    type: [AssetTotalDto],
    description: 'Total donated per asset, ordered by asset code.',
  })
  totalRaised: AssetTotalDto[];

  @ApiProperty({ example: 12, description: 'Pools with status Active.' })
  activePools: number;

  @ApiProperty({ example: 4, description: 'Pools with status Completed.' })
  completedPools: number;

  @ApiProperty({
    example: 87,
    description: 'Distinct donor wallets across all donations.',
  })
  uniqueDonors: number;
}
