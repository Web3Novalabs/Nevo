import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';
import {
  IsNotEmpty,
  IsNumberString,
  IsOptional,
  IsString,
  IsUrl,
  Matches,
  MaxLength,
} from 'class-validator';
import { STELLAR_PUBLIC_KEY } from '../../common/stellar-public-key.js';

export class CreatePoolDto {
  @ApiProperty({ description: 'Pool id assigned by the contract on-chain.' })
  @IsString()
  @IsNotEmpty()
  contractPoolId: string;

  @ApiProperty({ description: 'Stellar public key (G...) of the creator.' })
  @IsString()
  @IsNotEmpty()
  @Matches(STELLAR_PUBLIC_KEY, {
    message: 'creatorWallet must be a valid Stellar public key (G...)',
  })
  creatorWallet: string;

  @ApiProperty({ maxLength: 100 })
  @IsString()
  @IsNotEmpty()
  @MaxLength(100)
  title: string;

  @ApiProperty({ maxLength: 1000 })
  @IsString()
  @IsNotEmpty()
  @MaxLength(1000)
  description: string;

  @ApiProperty({
    description: 'Fundraising target, as a numeric string in stroops.',
    example: '1000000000',
  })
  @IsNumberString()
  @Matches(/^[1-9][0-9]*$/, { message: 'goal must be a positive integer' })
  goal: string;

  @ApiPropertyOptional({
    description: 'Category of the pool (e.g., Education, Healthcare, etc.)',
    maxLength: 100,
  })
  @IsOptional()
  @IsString()
  @MaxLength(100)
  category?: string;

  @ApiPropertyOptional({
    description: 'URL to the pool\'s image',
    nullable: true,
  })
  @IsOptional()
  @IsString()
  @IsUrl()
  imageUrl?: string | null;
}