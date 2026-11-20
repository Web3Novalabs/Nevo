import { Type } from 'class-transformer';
import {
  IsInt,
  IsOptional,
  IsString,
  IsUUID,
  Length,
  Max,
  Min,
} from 'class-validator';

export const COMMENT_TEXT_MIN_LENGTH = 1;
export const COMMENT_TEXT_MAX_LENGTH = 2000;

export class CreateCommentDto {
  @IsString()
  @Length(COMMENT_TEXT_MIN_LENGTH, COMMENT_TEXT_MAX_LENGTH)
  text: string;

  @IsOptional()
  @IsUUID()
  parentId?: string;
}

export class UpdateCommentDto {
  @IsString()
  @Length(COMMENT_TEXT_MIN_LENGTH, COMMENT_TEXT_MAX_LENGTH)
  text: string;
}

export class GetCommentsDto {
  @IsOptional()
  @Type(() => Number)
  @IsInt()
  @Min(1)
  page?: number = 1;

  @IsOptional()
  @Type(() => Number)
  @IsInt()
  @Min(1)
  @Max(100)
  limit?: number = 20;
}
