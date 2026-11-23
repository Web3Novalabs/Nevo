import { ApiProperty } from '@nestjs/swagger';

export class AuthUserDto {
  @ApiProperty({ format: 'uuid' })
  id: string;

  @ApiProperty({ description: 'Stellar public key (G...).' })
  publicKey: string;

  @ApiProperty({ type: String, nullable: true })
  displayName: string | null;

  @ApiProperty({ type: 'integer' })
  tokenVersion: number;

  @ApiProperty({ type: String, format: 'date-time' })
  createdAt: Date;

  @ApiProperty({ type: String, format: 'date-time' })
  updatedAt: Date;
}

export class AuthResultDto {
  @ApiProperty({ description: 'JWT to send as a bearer token.' })
  accessToken: string;

  @ApiProperty({ type: AuthUserDto })
  user: AuthUserDto;
}

export class AuthChallengeDto {
  @ApiProperty({ description: 'Single-use nonce to sign with the wallet.' })
  nonce: string;
}

export class LogoutResultDto {
  @ApiProperty({ enum: [true] })
  success: true;
}
