import { Injectable, Logger, OnModuleInit, Optional } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { scValToNative, rpc as StellarRpc } from '@stellar/stellar-sdk';
import {
  ADVISORY_LOCK_KEYS,
  AdvisoryLockService,
} from '../common/advisory-lock.service.js';
import { PoolsService } from '../pools/pools.service.js';
import { DonationsService } from '../donations/donations.service.js';
import { HistoryService } from '../transactions/history.service.js';
import { StellarConfig } from '../contract/stellar.config.js';
import { PoolStatus } from '../pools/pool.entity.js';
import { SyncState } from './sync-state.entity.js';

/** Minimal shape of a Stellar Horizon Soroban contract event. */
export interface HorizonContractEvent {
  /** Event topic; index 0 is the event symbol, index 1 is the pool_id. */
  topic: string[];
  /**
   * Event data value; entries are `scValToNative`-decoded and stringified
   * (e.g. `['GDONOR123', '500']` for a donation).
   */
  value: string[];
  /** Transaction hash for idempotency — may be undefined for non-donation events. */
  txHash?: string;
}

/** Maps on-chain PoolState names to the DB PoolStatus enum. */
const POOL_STATE_TO_STATUS: Record<string, PoolStatus> = {
  Active: PoolStatus.Active,
  Paused: PoolStatus.Paused,
  Completed: PoolStatus.Completed,
  Cancelled: PoolStatus.Cancelled,
  Disbursed: PoolStatus.Disbursed,
  Closed: PoolStatus.Closed,
};

@Injectable()
export class SyncService implements OnModuleInit {
  private readonly logger = new Logger(SyncService.name);
  private currentCursor: string | null = null;
  /** Tracks tx hashes seen in the current poll run to detect within-run duplicates. */
  private seenInRun = new Set<string>();

  private readonly rpcServer: StellarRpc.Server;

  constructor(
    private readonly poolsService: PoolsService,
    private readonly donationsService: DonationsService,
    @InjectRepository(SyncState)
    private readonly syncStateRepo: Repository<SyncState>,
    private readonly advisoryLock: AdvisoryLockService,
    private readonly stellar: StellarConfig,
    @Optional() private readonly history?: HistoryService,
  ) {
    this.rpcServer = new StellarRpc.Server(stellar.rpcUrl);
  }

  @Cron(CronExpression.EVERY_MINUTE)
  async syncPoolDeadlines(): Promise<void> {
    await this.advisoryLock.runExclusive(
      ADVISORY_LOCK_KEYS.poolDeadlineSync,
      'SyncService.syncPoolDeadlines',
      () => this.poolsService.syncApplicationDeadlines(),
    );
  }

  async onModuleInit() {
    const state = await this.syncStateRepo.findOne({
      where: { key: 'horizon_cursor' },
    });
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

  /**
   * Fetches contract events via Soroban RPC `getEvents` and dispatches them
   * to the `process*Event` handlers. Runs under the advisory lock so
   * multiple instances can't poll the same cursor concurrently.
   *
   * The cursor is persisted after each processed page (`saveCursor`), so a
   * restart resumes where the last run stopped. On first start or when the
   * saved cursor has fallen out of RPC retention, polling starts from the
   * configured ledger (`syncStartLedger` / the RPC's oldest ledger).
   */
  @Cron(CronExpression.EVERY_MINUTE)
  async pollHorizonEvents(): Promise<void> {
    await this.advisoryLock.runExclusive(
      ADVISORY_LOCK_KEYS.syncHorizonEvents,
      'SyncService.pollHorizonEvents',
      async () => {
        this.seenInRun.clear();
        const filters: StellarRpc.Api.EventFilter[] = [
          { type: 'contract', contractIds: [this.stellar.contractId] },
        ];

        let cursor = this.currentCursor;
        if (cursor && (await this.isCursorStale(cursor))) {
          // The cursor fell out of RPC retention; restart from the
          // configured start ledger (or the RPC's oldest ledger).
          cursor = null;
          this.currentCursor = null;
        }

        let request: StellarRpc.Api.GetEventsRequest;
        if (cursor) {
          request = { filters, cursor };
        } else {
          request = { filters, startLedger: await this.startLedger() };
        }

        for (;;) {
          const response = await this.rpcServer.getEvents(request);
          const events = response.events ?? [];
          for (const event of events) {
            await this.dispatchEvent(event);
          }
          if (!response.cursor || events.length === 0) break;
          // Persist the cursor after each processed page.
          await this.saveCursor(response.cursor);
          request = { filters, cursor: response.cursor };
        }
      },
    );
  }

  /**
   * Ledger to start polling from: the configured `SYNC_START_LEDGER`, else
   * the RPC's oldest retained ledger.
   */
  private async startLedger(): Promise<number> {
    const configured = this.stellar.syncStartLedger;
    if (configured > 0) return configured;
    try {
      const health = await this.rpcServer.getHealth();
      return health.oldestLedger;
    } catch {
      return configured;
    }
  }

  /** True when the saved cursor has fallen out of RPC retention. */
  private async isCursorStale(cursor: string): Promise<boolean> {
    try {
      const health = await this.rpcServer.getHealth();
      return Number(cursor) < health.oldestLedger;
    } catch {
      return false;
    }
  }

  /**
   * Decodes a Soroban RPC contract event into the flat
   * {@link HorizonContractEvent} shape used by the `process*Event` handlers:
   * topic/value entries are `scValToNative`-decoded and stringified.
   */
  private decodeEvent(
    event: StellarRpc.Api.EventResponse,
  ): HorizonContractEvent {
    const topic = (event.topic ?? []).map((v) => String(scValToNative(v)));
    const nativeValue: unknown = scValToNative(event.value);
    // `pool_stat` values arrive as [[oldState], [newState]] (the contract
    // publishes contracttype enums as ScVal::Vec([Symbol])); unwrap them.
    const value = (
      Array.isArray(nativeValue) ? nativeValue : [nativeValue]
    ).map((v) =>
      Array.isArray(v) && v.length === 1 && typeof v[0] === 'string'
        ? v[0]
        : String(v),
    );
    return { topic, value, txHash: event.txHash || undefined };
  }

  /** Dispatches a decoded contract event to the matching `process*Event` handler. */
  private async dispatchEvent(
    event: StellarRpc.Api.EventResponse,
  ): Promise<void> {
    const decoded = this.decodeEvent(event);
    switch (decoded.topic[0]) {
      case 'pool_crtd':
        await this.processPoolCreatedEvent(decoded);
        break;
      case 'pool_cls':
        await this.processPoolClosedEvent(decoded);
        break;
      case 'pool_stat':
        await this.processPoolStateEvent(decoded);
        break;
      case 'donation':
        await this.processDonationEvent(decoded);
        break;
    }
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
    if (event.txHash) {
      await this.history?.record(creatorWallet, {
        txHash: event.txHash,
        type: 'pool_creation',
        status: 'completed',
        amount: '0',
        recipient: creatorWallet,
      });
    }
  }

  /**
   * Processes a `pool_clos` (pool closed) event emitted by the Stellar smart contract.
   *
   * Expected event shape:
   * - `topic[0]` — Event symbol (e.g. `"pool_clos"`).
   * - `topic[1]` — The on-chain pool identifier (`contractPoolId`).
   * - `value` — Not used for this event type (empty array).
   *
   * The method checks for duplicate transactions, then marks the pool as
   * Closed via `PoolsService.markStatus()`.
   *
   * @param event - The Horizon contract event to process.
   */
  async processPoolClosedEvent(event: HorizonContractEvent): Promise<void> {
    if (event.txHash && (await this.isTxDuplicate(event.txHash))) {
      return;
    }

    const contractPoolId = event.topic[1];
    const pool = await this.poolsService.markStatus(
      contractPoolId,
      PoolStatus.Closed,
    );
    if (event.txHash && pool) {
      await this.history?.record(pool.creatorWallet, {
        txHash: event.txHash,
        type: 'pool_close',
        status: 'completed',
        amount: '0',
        recipient: contractPoolId,
      });
    }
  }

  /**
   * Processes a `pool_stat` event emitted by the contract's `set_pool_state`.
   *
   * Expected event shape (after decoding):
   * - `topic[0]` — Event symbol `"pool_stat"`.
   * - `topic[1]` — The on-chain pool identifier (`contractPoolId`).
   * - `value[1]` — New on-chain state name (e.g. `"Completed"`).
   *
   * Maps the on-chain state to the DB `PoolStatus` and stores it.
   *
   * @param event - The Horizon contract event to process.
   */
  async processPoolStateEvent(event: HorizonContractEvent): Promise<void> {
    const contractPoolId = event.topic[1];
    const stateName = event.value[1];
    const status = stateName ? POOL_STATE_TO_STATUS[stateName] : undefined;
    if (!status) return;
    await this.poolsService.markStatus(contractPoolId, status);
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

    try {
      await this.donationsService.recordDonation({
        poolId: contractPoolId,
        donorWallet,
        amount,
        asset,
        txHash: event.txHash,
      });
    } catch (error) {
      if (
        error instanceof Error &&
        (error as NodeJS.ErrnoException).code === '23505'
      ) {
        this.logger.warn(
          `Donation already recorded for tx ${event.txHash} — skipping`,
        );
        return;
      }
      throw error;
    }

    await this.poolsService.incrementRaised(contractPoolId, amount);

    // Mirror the contract: a donation that reaches the goal moves the pool
    // to Completed (`donate` sets `PoolState::Completed` when `collected >= goal`).
    const pool = await this.poolsService.findByContractId(contractPoolId);
    if (pool && BigInt(pool.raised || '0') >= BigInt(pool.goal || '0')) {
      await this.poolsService.markStatus(contractPoolId, PoolStatus.Completed);
    }
  }
}
