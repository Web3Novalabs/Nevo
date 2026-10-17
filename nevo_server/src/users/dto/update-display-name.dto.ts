import { Transform } from 'class-transformer';
import { IsString, Length } from 'class-validator';

export class UpdateDisplayNameDto {
  // Trimmed before validation, so empty and whitespace-only values are rejected
  // by @Length(1, 50). The 50-char cap sits well within the entity's
  // varchar(255) column.
  @Transform(({ value }) => (typeof value === 'string' ? value.trim() : value))
  @IsString()
  @Length(1, 50, {
    message: 'displayName must be between 1 and 50 characters after trimming',
  })
  displayName: string;
}
