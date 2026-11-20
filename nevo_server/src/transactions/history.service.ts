import { Injectable, Logger } from '@nestjs/common';
import { ContractService } from '../contract/contract.service.js';
import { DataSource } from 'typeorm';
import type { HistoryItemDto, HistoryPageDto, HistoryQueryDto } from './history.dto.js';

export type HistoryEntry = Pick<HistoryItemDto, 'txHash' | 'type' | 'amount' | 'recipient' | 'status'>;

@Injectable()
export class HistoryService {
  private readonly logger = new Logger(HistoryService.name);

  constructor(
    private readonly dataSource: DataSource,
    private readonly contract: ContractService,
  ) {}

  async reconcilePending(): Promise<void> {
    const rows = await this.dataSource.query<Array<{ tx_hash: string }>>(
      `SELECT tx_hash FROM transactions WHERE status = 'pending'
       GROUP BY tx_hash ORDER BY min(checked_at) NULLS FIRST, tx_hash LIMIT 50`,
    );

    for (const { tx_hash: hash } of rows) {
      try {
        const status = await this.contract.getSubmissionStatus(hash);
        await this.dataSource.query(
          `UPDATE transactions SET status = COALESCE($2, status), checked_at = now()
           WHERE tx_hash = $1 AND status = 'pending'`,
          [hash, status],
        );
      } catch {
        this.logger.warn(`Unable to reconcile Stellar transaction ${hash}`);
      }
    }
  }

  async record(wallet: string, entry: HistoryEntry): Promise<void> {
    await this.dataSource.query(
      `INSERT INTO transactions (wallet, tx_hash, type, status, amount, recipient)
       VALUES ($1, $2, $3, $4, $5, $6)
       ON CONFLICT (wallet, tx_hash, type) DO UPDATE
       SET status = EXCLUDED.status, amount = EXCLUDED.amount, recipient = EXCLUDED.recipient
       WHERE EXCLUDED.status = 'completed'`,
      [wallet, entry.txHash, entry.type, entry.status, entry.amount, entry.recipient],
    );
  }

  async list(wallet: string, query: HistoryQueryDto): Promise<HistoryPageDto> {
    const { page, limit } = query;
    const [result] = await this.dataSource.query<Array<{ data: HistoryItemDto[]; total: number }>>(
      `WITH history AS (
         SELECT d.id::text, 'donation' AS type, 'completed' AS status, d.amount, d.asset,
                d.pool_id AS recipient, d.pool_id, d.tx_hash AS "txHash", d.created_at AS date
         FROM donations d WHERE d.donor_wallet = $1
         UNION ALL
         SELECT t.tx_hash || ':' || t.type, t.type, t.status, t.amount, '' AS asset,
                t.recipient, CASE WHEN t.type <> 'pool_creation' THEN t.recipient END,
                t.tx_hash, t.created_at
         FROM transactions t WHERE t.wallet = $1 AND (
           t.type <> 'donation' OR NOT EXISTS (
             SELECT 1 FROM donations d WHERE d.donor_wallet = $1 AND d.tx_hash = t.tx_hash
           )
         )
       ), filtered AS (
         SELECT * FROM history
         WHERE ($2::text IS NULL OR type = $2) AND ($3::text IS NULL OR status = $3)
           AND ($4::timestamptz IS NULL OR date >= $4)
           AND ($5::timestamptz IS NULL OR date < $5)
           AND ($6::text IS NULL OR pool_id = $6) AND ($7::text IS NULL OR asset = $7)
           AND ($10::text IS NULL OR strpos(lower(recipient), lower($10)) > 0
             OR strpos(lower("txHash"), lower($10)) > 0)
       ), paged AS (
         SELECT id, type, status, amount, asset, recipient, "txHash", date
         FROM filtered ORDER BY date DESC, id DESC LIMIT $8 OFFSET $9
       )
       SELECT COALESCE((SELECT json_agg(paged ORDER BY date DESC, id DESC) FROM paged), '[]') AS data,
              (SELECT count(*)::int FROM filtered) AS total`,
      [wallet, query.type, query.status, query.from,
        query.to?.length === 10 ? new Date(new Date(query.to).getTime() + 86400000) : query.to,
        query.poolId, query.asset, limit, (page - 1) * limit, query.search],
    );
    return { ...result, page, limit };
  }
}
