import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { PoolsService } from '../pools/pools.service.js';
import { DonationsService } from '../donations/donations.service.js';
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
  /** Transaction hash for idempotency — may be undefined for non-donation events. */
  txHash?: string;
}

@Injectable()
export class SyncService implements OnModuleInit {
  private readonly logger = new Logger(SyncService.name);
  private currentCursor: string | null = null;
  /** Tracks tx hashes seen in the current poll run to detect within-run duplicates. */
  private seenInRun = new Set<string>();

  constructor(
    private readonly poolsService: PoolsService,
    private readonly donationsService: DonationsService,
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
    this.seenInRun.clear();
    // stub — will call HorizonService.fetchContractEvents() when implemented
  }

  /**
   * Returns true if the tx should be skipped (already processed or duplicate in this run).
   * Logs a warning when the same hash appears more than once in a single run.
   */
  async isTxDuplicate(txHash: string): Promise<boolean> {
    if (this.seenInRun.has(txHash)) {
      this.logger.warn(`Duplicate tx hash in current run: ${txHash}`);
      return true;
    }
    this.seenInRun.add(txHash);

    const alreadyProcessed = await this.donationsService.isTxProcessed(txHash);
    if (alreadyProcessed) {
      return true;
    }
    return false;
  }

  async processPoolCreatedEvent(event: HorizonContractEvent): Promise<void> {
    if (event.txHash && (await this.isTxDuplicate(event.txHash))) {
      return;
    }

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
    if (event.txHash && (await this.isTxDuplicate(event.txHash))) {
      return;
    }

    const contractPoolId = event.topic[1];
    await this.poolsService.markCompleted(contractPoolId);
  }
}
