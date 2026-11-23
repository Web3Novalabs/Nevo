import { ApiProperty } from '@nestjs/swagger';
import { PoolStatus } from '../pool.entity.js';

export class PoolResponseDto {
  @ApiProperty({ format: 'uuid' })
  id: string;

  @ApiProperty({ description: 'On-chain pool id.', example: '1' })
  contractPoolId: string;

  @ApiProperty()
  title: string;

  @ApiProperty()
  description: string;

  @ApiProperty()
  category: string;

  @ApiProperty({ description: 'Stellar public key of the pool creator.' })
  creatorWallet: string;

  @ApiProperty({ enum: PoolStatus, enumName: 'PoolStatus' })
  status: PoolStatus;

  @ApiProperty({ description: 'Goal in base units, as a decimal string.' })
  goal: string;

  @ApiProperty({
    description: 'Indexed amount raised in base units, as a decimal string.',
  })
  raised: string;

  @ApiProperty({ type: String, nullable: true })
  imageUrl: string | null;

  @ApiProperty({ type: String, format: 'date-time' })
  createdAt: Date;

  @ApiProperty({ type: String, format: 'date-time' })
  updatedAt: Date;
}

export class PoolDetailResponseDto extends PoolResponseDto {
  @ApiProperty({
    description: 'Live on-chain total in base units, as a decimal string.',
  })
  raisedOnChain: string;

  @ApiProperty({ description: 'Whether the pool is closed on-chain.' })
  closedOnChain: boolean;

  @ApiProperty({ type: 'integer', description: 'Unique on-chain donors.' })
  donorCount: number;
}

export class PaginatedPoolsDto {
  @ApiProperty({ type: [PoolResponseDto] })
  data: PoolResponseDto[];

  @ApiProperty({ type: 'integer' })
  total: number;

  @ApiProperty({ type: 'integer' })
  page: number;

  @ApiProperty({ type: 'integer' })
  limit: number;
}

export class UnsignedTransactionDto {
  @ApiProperty({ description: 'Base64 XDR for the wallet to sign and submit.' })
  unsignedXdr: string;
}

export class WithdrawTransactionDto extends UnsignedTransactionDto {
  @ApiProperty({ description: 'On-chain pool id.' })
  poolId: string;
}
