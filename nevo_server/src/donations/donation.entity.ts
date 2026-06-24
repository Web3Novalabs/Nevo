import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  Index,
} from 'typeorm';

@Entity('donations')
export class Donation {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Index({ unique: true })
  @Column({ name: 'tx_hash', type: 'varchar', length: 255 })
  txHash: string;

  @Column({ name: 'pool_id', type: 'int' })
  poolId: number;

  @Column({ name: 'donor_wallet', type: 'varchar', length: 56 })
  donorWallet: string;

  @Column({ type: 'bigint' })
  amount: string;

  @Column({ type: 'varchar', length: 56 })
  asset: string;

  @CreateDateColumn({ name: 'created_at' })
  createdAt: Date;
}
