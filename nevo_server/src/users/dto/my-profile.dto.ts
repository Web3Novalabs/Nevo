import { ApiProperty } from '@nestjs/swagger';

export class MyProfileDto {
  @ApiProperty()
  publicKey: string;

  @ApiProperty({ type: String, nullable: true })
  displayName: string | null;

  @ApiProperty({ type: String, format: 'date-time' })
  createdAt: Date;
}
