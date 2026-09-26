import { Transform } from 'class-transformer';
import { IsString, Length } from 'class-validator';

export class UpdateDisplayNameDto {
  @Transform(({ value }) => (typeof value === 'string' ? value.trim() : value))
  @IsString()
  @Length(1, 50)
  displayName: string;
}
