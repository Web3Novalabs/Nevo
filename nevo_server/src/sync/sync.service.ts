import { Injectable, Logger, OnModuleInit, Optional } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import {
  Address,
  Asset,
  FeeBumpTransaction,
  StrKey,
  TransactionBuilder,
  scValToNative,
  rpc as StellarRpc,
  type xdr,
} from '@stellar/stellar-sdk';
import {
  ADVISORY_LOCK_KEYS,
  AdvisoryLockService,
} from '../common/advisory-lock.service.js';
import { PoolsService } from '../pools/pools.service.js';
import { DonationsService } from '../donations/donations.service.js';
import { HistoryService } from '../transactions/history.service.js';
import { StellarConfig } from '../contract/stellar.config.js';
import { Pool, PoolStatus } from '../pools/pool.entity.js';
import { SyncState } from './sync-state.entity.js';

/** Minimal shape of a Stellar Horizon Soroban contract event. */
export interface HorizonContractEvent {
  /** Event topic; index 0 is the event symbol, index 1 is the pool_id. */
  topic: string[];
  /**
   * Event data value; entries are `scValToNative`-decoded and stringified
   * (e.g. `['GDONOR123', '500', '500']` for a contribution).
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

/**
 * Exact event symbols emitted by the contract, per `EVENTS_REFERENCE.md`:
 * - `pool_crtd` — pool created
 * - `contrib`   — contribution made via `donate` / `donate_with_token`
 * - `pool_cls`  — pool closed
 * - `don_refnd` — donation refunded
 * - `app_sub`   — application submitted
 * - `pool_stat` — pool state changed
 */
type EventHandler = (event: HorizonContractEvent) => Promise<void>;

@Injectable()
export class SyncService implements OnModuleInit {
  private readonly logger = new Logger(SyncService.name);
  private currentCursor: string | null = null;
  /** Tracks tx hashes seen in the current poll run to detect within-run duplicates. */
  private seenInRun = new Set<string>();

  private readonly rpcServer: StellarRpc.Server;

  /** Topic-symbol → handler map, keyed by the exact symbols from EVENTS_REFERENCE.md. */
  private readonly handlers: Record<string, EventHandler>;

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
    this.handlers = {
      pool_crtd: (e) => this.processPoolCreatedEvent(e),
      contrib: (e) => this.processContributionEvent(e),
      pool_cls: (e) => this.processPoolClosedEvent(e),
      don_refnd: (e) => this.processDonationRefundedEvent(e),
      app_sub: (e) => this.processApplicationSubmittedEvent(e),
      pool_stat: (e) => this.processPoolStateEvent(e),
    };
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
   * to the handlers registered in {@link handlers}. Runs under the advisory
   * lock so multiple instances can't poll the same cursor concurrently.
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
   * {@link HorizonContractEvent} shape used by the event handlers:
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

  /** Dispatches a decoded contract event to the matching handler via the topic map. */
  private async dispatchEvent(
    event: StellarRpc.Api.EventResponse,
  ): Promise<void> {
    const decoded = this.decodeEvent(event);
    const symbol = decoded.topic[0];
    const handler = symbol ? this.handlers[symbol] : undefined;
    if (!handler) {
      this.logger.debug(`No handler for event topic "${symbol}" — skipping`);
      return;
    }
    await handler(decoded);
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
   * - `topic[0]` — Event symbol `"pool_crtd"`.
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
   * Processes a `pool_cls` (pool closed) event emitted by the Stellar smart contract.
   *
   * Expected event shape:
   * - `topic[0]` — Event symbol `"pool_cls"`.
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
   * Processes an `app_sub` (application submitted) event emitted by the contract.
   *
   * Expected event shape:
   * - `topic[0]` — Event symbol `"app_sub"`.
   * - `topic[1]` — The on-chain pool identifier (`contractPoolId`).
   * - `value[0]` — Wallet address of the applicant (`applicantWallet`).
   *
   * Application records are managed by the applications module; this handler
   * exists so the event is acknowledged rather than silently dropped.
   *
   * @param event - The Horizon contract event to process.
   */
  async processApplicationSubmittedEvent(
    event: HorizonContractEvent,
  ): Promise<void> {
    this.logger.debug(
      `Application submitted for pool ${event.topic[1]} by ${event.value[0]}`,
    );
  }

  /** Short display code; the exact token is stored in `tokenContract`. */
  private assetCode(
    tokenContract: string | null,
    poolToken?: string | null,
  ): string {
    const native = Asset.native().contractId(this.stellar.networkPassphrase);
    if (tokenContract === native) return 'XLM';
    if (poolToken && poolToken.length <= 12) return poolToken;
    return tokenContract ? 'TOKEN' : 'XLM';
  }

  /**
   * Reads the `token` argument of the matching `donate_with_token` call from
   * the transaction, including calls made through another contract.
   */
  private async resolveDonationToken(
    txHash: string,
    poolId: string,
    donor: string,
  ): Promise<string | null> {
    try {
      const response = await this.rpcServer.getTransaction(txHash);
      if (response.status !== StellarRpc.Api.GetTransactionStatus.SUCCESS) {
        return null;
      }
      const parsed = TransactionBuilder.fromXDR(
        response.envelopeXdr,
        this.stellar.networkPassphrase,
      );
      const tx =
        parsed instanceof FeeBumpTransaction
          ? parsed.innerTransaction
          : parsed;
      const op = tx.operations[0];
      if (op?.type !== 'invokeHostFunction') return null;

      const calls: xdr.InvokeContractArgs[] = [];
      if (op.func.switch().name === 'hostFunctionTypeInvokeContract') {
        calls.push(op.func.invokeContract());
      }
      const walk = (node: xdr.SorobanAuthorizedInvocation): void => {
        const fn = node.function();
        if (fn.switch().name === 'sorobanAuthorizedFunctionTypeContractFn') {
          calls.push(fn.contractFn());
        }
        node.subInvocations().forEach(walk);
      };
      (op.auth ?? []).forEach((entry) => walk(entry.rootInvocation()));

      for (const call of calls) {
        const args = call.args();
        if (
          call.functionName().toString() === 'donate_with_token' &&
          Address.fromScAddress(call.contractAddress()).toString() ===
            this.stellar.contractId &&
          args.length >= 3 &&
          String(scValToNative(args[0])) === poolId &&
          String(scValToNative(args[1])) === donor
        ) {
          return String(scValToNative(args[2]));
        }
      }
    } catch (error) {
      this.logger.warn(
        `Could not read token for tx ${txHash}: ${String(error)}`,
      );
    }
    return null;
  }

  /**
   * Processes a `contrib` (contribution) event emitted by the contract's
   * `donate` / `donate_with_token` functions.
   *
   * Expected event shape:
   * - `topic[0]` — Event symbol `"contrib"`.
   * - `topic[1]` — The on-chain pool identifier (`contractPoolId`).
   * - `value[0]` — Wallet address of the donor (`donorWallet`).
   * - `value[1]` — Contribution amount in base units (`amount`).
   * - `value[2]` — The pool's running total after this contribution
   *   (`newTotal`), **not** an asset code.
   * - `txHash` — **Required** for contribution events. If missing, the event is skipped with a warning.
   *
   * The exact token contract is read from the transaction, the display asset
   * code is derived from it, the donation is recorded via `DonationsService.recordDonation()`, and the
   * pool's raised amount is set to the running total from the event.
   *
   * @param event - The Horizon contract event to process.
   */
  async processContributionEvent(event: HorizonContractEvent): Promise<void> {
    if (!event.txHash) {
      this.logger.warn('Contribution event missing txHash — skipping');
      return;
    }

    if (await this.isTxDuplicate(event.txHash)) {
      return;
    }

    const contractPoolId = event.topic[1];
    const donorWallet = event.value[0];
    const amount = event.value[1];
    const newTotal = event.value[2];

    // The event carries no token, so read it from the donation transaction.
    const pool = await this.poolsService.findByContractId(contractPoolId);
    const poolToken = (pool as (Pool & { token?: string | null }) | null)?.token;
    const tokenContract =
      (await this.resolveDonationToken(
        event.txHash,
        contractPoolId,
        donorWallet,
      )) ?? (poolToken && StrKey.isValidContract(poolToken) ? poolToken : null);
    if (!tokenContract) {
      this.logger.warn(`No token contract resolved for tx ${event.txHash}`);
    }
    const asset = this.assetCode(tokenContract, poolToken);

    try {
      await this.donationsService.recordDonation({
        poolId: contractPoolId,
        donorWallet,
        amount,
        asset,
        tokenContract,
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

    // The contract publishes the running total as `value[2]`; prefer it over
    // incrementing so the stored total stays in sync with on-chain state.
    if (newTotal !== undefined && newTotal !== '') {
      await this.poolsService.setRaised(contractPoolId, newTotal);
    } else {
      await this.poolsService.incrementRaised(contractPoolId, amount);
    }

    // Mirror the contract: a contribution that reaches the goal moves the
    // pool to Completed (`donate` sets `PoolState::Completed` when
    // `collected >= goal`).
    const updated = await this.poolsService.findByContractId(contractPoolId);
    if (
      updated &&
      BigInt(updated.raised || '0') >= BigInt(updated.goal || '0')
    ) {
      await this.poolsService.markStatus(contractPoolId, PoolStatus.Completed);
    }
  }

  /**
   * Processes a `don_refnd` (donation refunded) event emitted by the contract.
   *
   * Expected event shape:
   * - `topic[0]` — Event symbol `"don_refnd"`.
   * - `topic[1]` — The on-chain pool identifier (`contractPoolId`).
   * - `value[0]` — Wallet address of the donor (`donorWallet`).
   * - `value[1]` — Refunded amount in base units (`amount`).
   * - `txHash` — **Required** for refund events. If missing, the event is skipped with a warning.
   *
   * Decrements the pool's raised amount by the refunded amount and marks the
   * corresponding donation as refunded.
   *
   * @param event - The Horizon contract event to process.
   */
  async processDonationRefundedEvent(
    event: HorizonContractEvent,
  ): Promise<void> {
    if (!event.txHash) {
      this.logger.warn('Refund event missing txHash — skipping');
      return;
    }

    if (await this.isTxDuplicate(event.txHash)) {
      return;
    }

    const contractPoolId = event.topic[1];
    const donorWallet = event.value[0];
    const amount = event.value[1];

    await this.poolsService.decrementRaised(contractPoolId, amount);

    await this.donationsService.markRefunded({
      poolId: contractPoolId,
      donorWallet,
      amount,
      txHash: event.txHash,
    });
  }
}