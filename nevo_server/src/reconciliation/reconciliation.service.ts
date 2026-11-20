import {
  Injectable,
  Logger,
  OnModuleDestroy,
  OnModuleInit,
} from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { SchedulerRegistry } from '@nestjs/schedule';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import {
  ADVISORY_LOCK_KEYS,
  AdvisoryLockService,
} from '../common/advisory-lock.service.js';
import { ContractService } from '../contract/contract.service.js';
import { Pool, PoolStatus } from '../pools/pool.entity.js';

export const DEFAULT_RECONCILIATION_INTERVAL_MS = 5 * 60 * 1000;
export const DEFAULT_RECONCILIATION_BATCH_SIZE = 20;
const INTERVAL_NAME = 'poolRaisedReconciliation';

export interface ReconciliationDrift {
  poolId: string;
  contractPoolId: string;
  dbRaised: string;
  onChainRaised: string;
  /** on-chain minus database; positive means the database is behind. */
  delta: string;
}

export interface ReconciliationRunResult {
  status: 'completed' | 'skipped';
  trigger: 'scheduled' | 'manual';
  startedAt: string;
  finishedAt: string;
  checked: number;
  matched: number;
  drifted: number;
  corrected: number;
  /** Updates skipped because `raised` changed concurrently; retried next run. */
  conflicts: number;
  rpcErrors: number;
  dbErrors: number;
  drifts: ReconciliationDrift[];
}

export interface ReconciliationStats {
  runs: number;
  checked: number;
  driftDetected: number;
  driftCorrected: number;
  rpcErrors: number;
  dbErrors: number;
  lastRun: ReconciliationRunResult | null;
}

@Injectable()
export class ReconciliationService implements OnModuleInit, OnModuleDestroy {
  private readonly logger = new Logger(ReconciliationService.name);
  private running = false;
  private readonly stats: ReconciliationStats = {
    runs: 0,
    checked: 0,
    driftDetected: 0,
    driftCorrected: 0,
    rpcErrors: 0,
    dbErrors: 0,
    lastRun: null,
  };

  constructor(
    @InjectRepository(Pool) private readonly poolRepo: Repository<Pool>,
    private readonly contractService: ContractService,
    private readonly advisoryLock: AdvisoryLockService,
    private readonly config: ConfigService,
    private readonly scheduler: SchedulerRegistry,
  ) {}

  onModuleInit(): void {
    const intervalMs = this.positiveInt(
      'RECONCILIATION_INTERVAL_MS',
      DEFAULT_RECONCILIATION_INTERVAL_MS,
    );
    if (this.config.get<string>('RECONCILIATION_ENABLED') === 'false') {
      this.logger.log('Scheduled pool reconciliation is disabled');
      return;
    }
    const handle = setInterval(() => {
      void this.reconcile('scheduled').catch((err: unknown) =>
        this.logger.error(
          'Scheduled reconciliation crashed',
          err instanceof Error ? err.stack : String(err),
        ),
      );
    }, intervalMs);
    this.scheduler.addInterval(INTERVAL_NAME, handle);
    this.logger.log(`Pool reconciliation scheduled every ${intervalMs}ms`);
  }

  onModuleDestroy(): void {
    if (this.scheduler.doesExist('interval', INTERVAL_NAME)) {
      this.scheduler.deleteInterval(INTERVAL_NAME);
    }
  }

  getStats(): ReconciliationStats {
    return { ...this.stats };
  }

  /**
   * Compares `pools.raised` with the on-chain `get_total_raised` for every
   * active pool and overwrites the database value when they differ. Safe to
   * call repeatedly: a second run over unchanged data changes nothing.
   */
  async reconcile(
    trigger: 'scheduled' | 'manual' = 'scheduled',
  ): Promise<ReconciliationRunResult> {
    const startedAt = new Date().toISOString();
    if (this.running) {
      this.logger.warn(
        'Reconciliation already running in this process; skipping',
      );
      return this.skipped(trigger, startedAt);
    }

    this.running = true;
    try {
      const result = await this.advisoryLock.runExclusive(
        ADVISORY_LOCK_KEYS.poolReconciliation,
        'ReconciliationService.reconcile',
        () => this.run(trigger, startedAt),
      );
      return result ?? this.skipped(trigger, startedAt);
    } finally {
      this.running = false;
    }
  }

  private async run(
    trigger: 'scheduled' | 'manual',
    startedAt: string,
  ): Promise<ReconciliationRunResult> {
    const batchSize = this.positiveInt(
      'RECONCILIATION_BATCH_SIZE',
      DEFAULT_RECONCILIATION_BATCH_SIZE,
    );
    const result: ReconciliationRunResult = {
      status: 'completed',
      trigger,
      startedAt,
      finishedAt: startedAt,
      checked: 0,
      matched: 0,
      drifted: 0,
      corrected: 0,
      conflicts: 0,
      rpcErrors: 0,
      dbErrors: 0,
      drifts: [],
    };

    let afterId: string | null = null;
    for (;;) {
      let batch: Pool[];
      try {
        batch = await this.nextBatch(afterId, batchSize);
      } catch (err) {
        result.dbErrors += 1;
        this.logger.error(
          'Failed to load pools for reconciliation',
          err instanceof Error ? err.stack : String(err),
        );
        break;
      }
      if (batch.length === 0) break;

      const outcomes = await Promise.allSettled(
        batch.map((pool) => this.reconcilePool(pool, result)),
      );
      outcomes.forEach((outcome, i) => {
        if (outcome.status === 'rejected') {
          result.dbErrors += 1;
          this.logger.error(
            `Unexpected reconciliation failure for pool ${batch[i].id}`,
            String(outcome.reason),
          );
        }
      });

      afterId = batch[batch.length - 1].id;
      if (batch.length < batchSize) break;
    }

    result.finishedAt = new Date().toISOString();
    this.record(result);
    return result;
  }

  private nextBatch(afterId: string | null, take: number): Promise<Pool[]> {
    const qb = this.poolRepo
      .createQueryBuilder('pool')
      .where('pool.status = :status', { status: PoolStatus.Active })
      .orderBy('pool.id', 'ASC')
      .take(take);
    if (afterId) qb.andWhere('pool.id > :afterId', { afterId });
    return qb.getMany();
  }

  private async reconcilePool(
    pool: Pool,
    result: ReconciliationRunResult,
  ): Promise<void> {
    result.checked += 1;

    const chainPoolId = Number(pool.contractPoolId);
    if (!Number.isSafeInteger(chainPoolId) || chainPoolId < 0) {
      result.rpcErrors += 1;
      this.logger.warn(
        `Skipping pool ${pool.id}: contractPoolId "${pool.contractPoolId}" is not a valid on-chain id`,
      );
      return;
    }

    let onChain: bigint;
    try {
      onChain = await this.contractService.fetchTotalRaisedOnChain(chainPoolId);
    } catch (err) {
      result.rpcErrors += 1;
      this.logger.error(
        `Failed to read get_total_raised for pool ${pool.id} (contract pool ${pool.contractPoolId}); leaving database untouched`,
        err instanceof Error ? err.stack : String(err),
      );
      return;
    }

    const dbRaised = BigInt(pool.raised || '0');
    if (dbRaised === onChain) {
      result.matched += 1;
      return;
    }

    const drift: ReconciliationDrift = {
      poolId: pool.id,
      contractPoolId: pool.contractPoolId,
      dbRaised: dbRaised.toString(),
      onChainRaised: onChain.toString(),
      delta: (onChain - dbRaised).toString(),
    };
    result.drifted += 1;
    result.drifts.push(drift);
    this.logger.warn(
      `Raised drift detected: ${JSON.stringify(drift)} (${onChain > dbRaised ? 'database behind chain' : 'database ahead of chain'})`,
    );

    try {
      // Conditional on the value we read so a concurrent indexer update is never clobbered.
      const update = await this.poolRepo.update(
        { id: pool.id, raised: dbRaised.toString() },
        { raised: onChain.toString() },
      );
      if (update.affected) {
        result.corrected += 1;
      } else {
        result.conflicts += 1;
        this.logger.warn(
          `Pool ${pool.id} raised changed during reconciliation; will retry next run`,
        );
      }
    } catch (err) {
      result.dbErrors += 1;
      this.logger.error(
        `Failed to correct raised for pool ${pool.id}`,
        err instanceof Error ? err.stack : String(err),
      );
    }
  }

  private record(result: ReconciliationRunResult): void {
    this.stats.runs += 1;
    this.stats.checked += result.checked;
    this.stats.driftDetected += result.drifted;
    this.stats.driftCorrected += result.corrected;
    this.stats.rpcErrors += result.rpcErrors;
    this.stats.dbErrors += result.dbErrors;
    this.stats.lastRun = result;
    const summary = `Reconciliation (${result.trigger}) checked=${result.checked} matched=${result.matched} drifted=${result.drifted} corrected=${result.corrected} conflicts=${result.conflicts} rpcErrors=${result.rpcErrors} dbErrors=${result.dbErrors}`;
    if (result.drifted || result.rpcErrors || result.dbErrors) {
      this.logger.warn(summary);
    } else {
      this.logger.log(summary);
    }
  }

  private skipped(
    trigger: 'scheduled' | 'manual',
    startedAt: string,
  ): ReconciliationRunResult {
    return {
      status: 'skipped',
      trigger,
      startedAt,
      finishedAt: new Date().toISOString(),
      checked: 0,
      matched: 0,
      drifted: 0,
      corrected: 0,
      conflicts: 0,
      rpcErrors: 0,
      dbErrors: 0,
      drifts: [],
    };
  }

  private positiveInt(key: string, fallback: number): number {
    const parsed = Number.parseInt(this.config.get<string>(key) ?? '', 10);
    return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
  }
}
