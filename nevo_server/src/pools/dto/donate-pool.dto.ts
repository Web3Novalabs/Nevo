import { IsNotEmpty, IsNumberString } from 'class-validator';

export class DonatePoolDto {
  @IsNumberString()
  @IsNotEmpty()
  amount: string;
}
