import { Entity, PrimaryColumn, Column } from 'typeorm';

@Entity('sync_state')
export class SyncState {
  @PrimaryColumn({ type: 'varchar', length: 50 })
  key: string;

  @Column({ type: 'varchar', length: 255 })
  value: string;
}
