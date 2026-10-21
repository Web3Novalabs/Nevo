import { ApiProperty } from '@nestjs/swagger';
import { IsNotEmpty, IsNumberString, Matches } from 'class-validator';

export class DonatePoolDto {
  @ApiProperty({
    description: 'Donation amount, as a numeric string in stroops.',
    example: '10000000',
  })
  @IsNumberString()
  @Matches(/^[1-9][0-9]*$/, { message: 'amount must be a positive integer' })
  @IsNotEmpty()
  amount: string;
}
