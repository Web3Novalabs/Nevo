import { IsNotEmpty, IsString } from 'class-validator';

export class VerifyAuthDto {
  @IsString()
  @IsNotEmpty()
  publicKey: string;

  @IsString()
  @IsNotEmpty()
  nonce: string;

  @IsString()
  @IsNotEmpty()
  signature: string;
}
