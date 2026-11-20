import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';
import { Type } from 'class-transformer';
import {
  IsDateString,
  IsIn,
  IsInt,
  IsOptional,
  IsString,
  Max,
  MaxLength,
  Min,
} from 'class-validator';

const types = [
  'donation',
  'pool_creation',
  'withdrawal',
  'pool_close',
] as const;
const statuses = ['completed', 'pending', 'failed'] as const;

export class HistoryQueryDto {
  @ApiPropertyOptional({ default: 1, minimum: 1 })
  @Type(() => Number)
  @IsInt()
  @Min(1)
  @Max(10000000)
  page = 1;

  @ApiPropertyOptional({ default: 20, minimum: 1, maximum: 100 })
  @Type(() => Number)
  @IsInt()
  @Min(1)
  @Max(100)
  limit = 20;

  @ApiPropertyOptional({ enum: types })
  @IsOptional()
  @IsIn(types)
  type?: (typeof types)[number];

  @ApiPropertyOptional({ enum: statuses })
  @IsOptional()
  @IsIn(statuses)
  status?: (typeof statuses)[number];

  @ApiPropertyOptional()
  @IsOptional()
  @IsDateString({ strict: true })
  from?: string;

  @ApiPropertyOptional()
  @IsOptional()
  @IsDateString({ strict: true })
  to?: string;

  @ApiPropertyOptional()
  @IsOptional()
  @IsString()
  poolId?: string;

  @ApiPropertyOptional()
  @IsOptional()
  @IsString()
  @MaxLength(200)
  search?: string;

  @ApiPropertyOptional()
  @IsOptional()
  @IsString()
  asset?: string;
}

export class HistoryItemDto {
  @ApiProperty()
  id: string;

  @ApiProperty({ enum: types })
  type: (typeof types)[number];

  @ApiProperty({ enum: statuses })
  status: (typeof statuses)[number];

  @ApiProperty()
  amount: string;

  @ApiProperty()
  asset: string;

  @ApiProperty()
  recipient: string;

  @ApiProperty()
  txHash: string;

  @ApiProperty({ type: String, format: 'date-time' })
  date: Date;
}

export class HistoryPageDto {
  @ApiProperty({ type: [HistoryItemDto] })
  data: HistoryItemDto[];

  @ApiProperty()
  total: number;

  @ApiProperty()
  page: number;

  @ApiProperty()
  limit: number;
}
