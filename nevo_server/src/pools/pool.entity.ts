import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
  Index,
} from 'typeorm';

export enum PoolStatus {
  Active = 'Active',
  Completed = 'Completed',
}

@Entity('pools')
export class Pool {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Index({ unique: true })
  @Column({ name: 'contract_pool_id', type: 'varchar', length: 255 })
  contractPoolId: string;

  @Column({ type: 'varchar', length: 255 })
  title: string;

  @Column({ type: 'text' })
  description: string;

  @Index()
  @Column({ type: 'varchar', length: 100, default: '' })
  category: string;

  @Index()
  @Column({ name: 'creator_wallet', type: 'varchar', length: 56 })
  creatorWallet: string;

  @Index()
  @Column({
    type: 'enum',
    enum: PoolStatus,
    default: PoolStatus.Active,
  })
  status: PoolStatus;

  @Column({ type: 'bigint' })
  goal: string;

  @Column({ type: 'bigint', default: 0 })
  raised: string;

  @Column({ name: 'image_url', type: 'varchar', length: 2048, nullable: true })
  imageUrl: string | null;

  @Index()
  @CreateDateColumn({ name: 'created_at' })
  createdAt: Date;

  @UpdateDateColumn({ name: 'updated_at' })
  updatedAt: Date;
}
