import { IsNotEmpty, IsNumberString, Matches } from 'class-validator';

export class DonatePoolDto {
  @IsNotEmpty()
  @IsNumberString()
  @Matches(/^[1-9][0-9]*$/, {
    message: 'amount must be a positive integer string',
  })
  amount: string;
}
