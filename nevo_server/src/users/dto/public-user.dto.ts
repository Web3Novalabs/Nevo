import { ApiProperty } from '@nestjs/swagger';

export class PublicUserDto {
  @ApiProperty({
    description: 'Stellar public key of the creator',
    example: 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF',
  })
  publicKey: string;

  @ApiProperty({
    description: 'Display name of the creator',
    example: 'Jane Doe',
    nullable: true,
  })
  displayName: string | null;

  @ApiProperty({
    description: 'Date the creator joined the platform',
    example: '2024-01-15T10:30:00.000Z',
    type: String,
    format: 'date-time',
  })
  joinedAt: Date;

  @ApiProperty({
    description: 'Number of pools created by this creator',
    example: 12,
  })
  poolsCreated: number;

  @ApiProperty({
    description:
      'Total amount raised across all pools created by this creator, in base units, as an exact integer string',
    example: '15000500000',
  })
  totalRaised: string;
}
