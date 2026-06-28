import { Injectable, OnModuleInit } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { PoolsService } from '../pools/pools.service.js';
import { SyncState } from './sync-state.entity.js';

/** Minimal shape of a Stellar Horizon Soroban contract event. */
export interface HorizonContractEvent {
  /** Event topic array; index 0 is the event symbol, index 1 is the pool_id. */
  topic: string[];
  /**
   * Event data value.
   * For pool_crtd: [creatorWallet, goal, title, description]
   */
  value: string[];
}

@Injectable()
export class SyncService implements OnModuleInit {
  private currentCursor: string | null = null;

  constructor(
    private readonly poolsService: PoolsService,
    @InjectRepository(SyncState)
    private readonly syncStateRepo: Repository<SyncState>,
  ) {}

  async onModuleInit() {
    const state = await this.syncStateRepo.findOne({ where: { key: 'horizon_cursor' } });
    if (state) {
      this.currentCursor = state.value;
    }
  }

  getCursor(): string | null {
    return this.currentCursor;
  }

  async saveCursor(cursor: string): Promise<void> {
    this.currentCursor = cursor;
    await this.syncStateRepo.save({ key: 'horizon_cursor', value: cursor });
  }

  // TODO: replace with real implementation once HorizonService (#46) is available
  @Cron(CronExpression.EVERY_MINUTE)
  async pollHorizonEvents(): Promise<void> {
    // stub — will call HorizonService.fetchContractEvents() when implemented
  }

  async processPoolCreatedEvent(event: HorizonContractEvent): Promise<void> {
    const contractPoolId = event.topic[1];
    const creatorWallet = event.value[0];
    const goal = event.value[1];

    await this.poolsService.upsertFromChain({
      contractPoolId,
      creatorWallet,
      goal,
    });
  }

  async processPoolClosedEvent(event: HorizonContractEvent): Promise<void> {
    const contractPoolId = event.topic[1];
    await this.poolsService.markCompleted(contractPoolId);
  }
}
