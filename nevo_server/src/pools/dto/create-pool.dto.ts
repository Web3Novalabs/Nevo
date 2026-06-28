import { Type } from 'class-transformer';
import {
  IsNumber,
  IsOptional,
  IsString,
  IsUrl,
  MaxLength,
  Min,
  MinLength,
} from 'class-validator';

export class CreatePoolDto {
  @IsString()
  @MinLength(3)
  @MaxLength(100)
  title: string;

  @IsString()
  @MinLength(10)
  @MaxLength(2000)
  description: string;

  @IsString()
  category: string;

  @Type(() => Number)
  @IsNumber()
  @Min(1)
  goal: number;

  @IsOptional()
  @IsUrl()
  imageUrl?: string;
}
