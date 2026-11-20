import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { InjectDataSource } from '@nestjs/typeorm';
import { DataSource } from 'typeorm';
import { PoolStatus } from '../pools/pool.entity.js';
import type { StatsResponseDto } from './stats.dto.js';

export const DEFAULT_STATS_CACHE_TTL_MS = 30_000;

@Injectable()
export class StatsService {
  private cached: { value: StatsResponseDto; expiresAt: number } | null = null;
  private inflight: Promise<StatsResponseDto> | null = null;

  constructor(
    @InjectDataSource() private readonly dataSource: DataSource,
    private readonly config: ConfigService,
  ) {}

  async getStats(): Promise<StatsResponseDto> {
    const ttl = this.ttlMs();
    if (this.cached && this.cached.expiresAt > Date.now()) {
      return this.cached.value;
    }
    // Concurrent misses share one query instead of stampeding the database.
    this.inflight ??= this.compute()
      .then((value) => {
        if (ttl > 0) this.cached = { value, expiresAt: Date.now() + ttl };
        return value;
      })
      .finally(() => {
        this.inflight = null;
      });
    return this.inflight;
  }

  private async compute(): Promise<StatsResponseDto> {
    // One snapshot so the totals and counts agree with each other.
    return this.dataSource.transaction('REPEATABLE READ', async (manager) => {
      const totals = await manager.query<
        Array<{ asset: string; amount: string }>
      >(
        `SELECT asset, SUM(amount::numeric)::text AS amount
         FROM donations GROUP BY asset ORDER BY asset`,
      );
      const poolCounts = await manager.query<
        Array<{ status: string; count: string }>
      >(
        `SELECT status::text AS status, COUNT(*)::text AS count FROM pools GROUP BY status`,
      );
      const [donors] = await manager.query<Array<{ count: string }>>(
        `SELECT COUNT(DISTINCT donor_wallet)::text AS count FROM donations`,
      );

      const countFor = (status: PoolStatus) =>
        Number(
          poolCounts.find((row) => row.status === String(status))?.count ?? 0,
        );

      return {
        totalRaised: totals.map(({ asset, amount }) => ({ asset, amount })),
        activePools: countFor(PoolStatus.Active),
        completedPools: countFor(PoolStatus.Completed),
        uniqueDonors: Number(donors?.count ?? 0),
      };
    });
  }

  private ttlMs(): number {
    const parsed = Number.parseInt(
      this.config.get<string>('STATS_CACHE_TTL_MS') ?? '',
      10,
    );
    return Number.isFinite(parsed) && parsed >= 0
      ? parsed
      : DEFAULT_STATS_CACHE_TTL_MS;
  }
}
