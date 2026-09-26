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
   * Determines whether a transaction should be skipped to avoid duplicate processing.
   *
   * Duplicate detection works on two levels:
   * 1. **Within-run detection** — The method maintains a `Set` of tx hashes seen during
   *    the current poll cycle (`seenInRun`). If the same hash appears more than once in
   *    one run (e.g. Horizon delivers overlapping pages), it is flagged and skipped. A
   *    warning is logged for observability.
   * 2. **Persisted duplicate detection** — The method queries `DonationsService.isTxProcessed`
   *    which checks whether the tx hash has already been recorded in the database from a
   *    previous sync run. This protects against re-processing transactions after a
   *    restart or cursor reset.
   *
   * @param txHash - The transaction hash to check (from `HorizonContractEvent.txHash`).
   * @returns `true` if the transaction should be skipped, `false` if it is new and safe to process.
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

  /**
   * Processes a `pool_crtd` (pool created) event emitted by the Stellar smart contract.
   *
   * Expected event shape:
   * - `topic[0]` — Event symbol (e.g. `"pool_crtd"`).
   * - `topic[1]` — The on-chain pool identifier (`contractPoolId`).
   * - `value[0]` — Wallet address of the pool creator (`creatorWallet`).
   * - `value[1]` — Fundraising goal amount in base units (`goal`).
   *
   * The method first checks for duplicate transactions (both within-run and persisted).
   * If the transaction is not a duplicate, it upserts the pool record via `PoolsService`.
   *
   * @param event - The Horizon contract event to process.
   */
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

  /**
   * Processes a `pool_clos` (pool closed) event emitted by the Stellar smart contract.
   *
   * Expected event shape:
   * - `topic[0]` — Event symbol (e.g. `"pool_clos"`).
   * - `topic[1]` — The on-chain pool identifier (`contractPoolId`).
   * - `value` — Not used for this event type (empty array).
   *
   * The method checks for duplicate transactions, then marks the pool as completed
   * via `PoolsService.markCompleted()`.
   *
   * @param event - The Horizon contract event to process.
   */
  async processPoolClosedEvent(event: HorizonContractEvent): Promise<void> {
    if (event.txHash && (await this.isTxDuplicate(event.txHash))) {
      return;
    }

    const contractPoolId = event.topic[1];
    await this.poolsService.markCompleted(contractPoolId);
  }

  /**
   * Processes a `donation` event emitted by the Stellar smart contract.
   *
   * Expected event shape:
   * - `topic[0]` — Event symbol (e.g. `"donation"`).
   * - `topic[1]` — The on-chain pool identifier (`contractPoolId`).
   * - `value[0]` — Wallet address of the donor (`donorWallet`).
   * - `value[1]` — Donation amount in base units (`amount`).
   * - `value[2]` — Asset code; defaults to `"XLM"` if omitted (`asset`).
   * - `txHash` — **Required** for donation events. If missing, the event is skipped with a warning.
   *
   * The method checks for duplicate transactions, records the donation via
   * `DonationsService.recordDonation()`, and increments the pool's raised amount
   * via `PoolsService.incrementRaised()`.
   *
   * @param event - The Horizon contract event to process.
   */
  async processDonationEvent(event: HorizonContractEvent): Promise<void> {
    if (!event.txHash) {
      this.logger.warn('Donation event missing txHash — skipping');
      return;
    }

    if (await this.isTxDuplicate(event.txHash)) {
      return;
    }

    const contractPoolId = event.topic[1];
    const donorWallet = event.value[0];
    const amount = event.value[1];
    const asset = event.value[2] || 'XLM';

    await this.donationsService.recordDonation({
      poolId: contractPoolId,
      donorWallet,
      amount,
      asset,
      txHash: event.txHash,
    });

    await this.poolsService.incrementRaised(contractPoolId, amount);
  }
}
