import { IsNotEmpty, IsString } from 'class-validator';

export class SubmitXdrDto {
  @IsString()
  @IsNotEmpty()
  signedXdr: string;
}
