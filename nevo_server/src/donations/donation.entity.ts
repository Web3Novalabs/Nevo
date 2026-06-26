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

  @Index()
  @Column({ name: 'pool_id', type: 'varchar', length: 255 })
  poolId: string;

  @Index()
  @Column({ name: 'donor_wallet', type: 'varchar', length: 56 })
  donorWallet: string;

  @Column({ type: 'varchar', length: 255 })
  amount: string;

  @Column({ type: 'varchar', length: 10 })
  asset: string;

  @Index({ unique: true })
  @Column({ name: 'tx_hash', type: 'varchar', length: 64 })
  txHash: string;

  @Column({ type: 'text', nullable: true })
  memo: string | null;

  @Index()
  @CreateDateColumn({ name: 'created_at' })
  createdAt: Date;
}
