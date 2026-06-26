'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { EmptyState } from '@/components/EmptyState';
import ProtectedRoute from '@/components/ProtectedRoute';
import { apiClient } from '@/lib/api-client';
import type { Transaction, TxType, TxStatus } from '@/src/lib/mockTransactions';

const PAGE_SIZE = 8;

const TYPE_LABELS: Record<TxType, string> = {
  donation: 'Donation',
  pool_creation: 'Pool Created',
  withdrawal: 'Withdrawal',
};

const STATUS_STYLES: Record<TxStatus, string> = {
  completed: 'bg-success-light text-success-dark',
  pending: 'bg-warning-light text-warning-dark',
  failed: 'bg-error-light text-error',
};

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

function formatTime(iso: string) {
  return new Date(iso).toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
  });
}

/* ── Icons ──────────────────────────────────────────────────────────────── */

function DonationIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-4"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12Z"
      />
    </svg>
  );
}

function PoolIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-4"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M12 6v12m-3-2.818.879.659c1.171.879 3.07.879 4.242 0 1.172-.879 1.172-2.303 0-3.182C13.536 12.219 12.768 12 12 12c-.725 0-1.45-.22-2.003-.659-1.106-.879-1.106-2.303 0-3.182s2.9-.879 4.006 0l.415.33M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
      />
    </svg>
  );
}

function WithdrawIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-4"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3"
      />
    </svg>
  );
}

function SearchIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-4"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
      />
    </svg>
  );
}

const TYPE_ICON: Record<TxType, React.ReactNode> = {
  donation: <DonationIcon />,
  pool_creation: <PoolIcon />,
  withdrawal: <WithdrawIcon />,
};

const TYPE_ICON_BG: Record<TxType, string> = {
  donation: 'bg-brand-100 text-brand-600',
  pool_creation: 'bg-warning-light text-warning-dark',
  withdrawal: 'bg-success-light text-success-dark',
};

/* ── Main Page ──────────────────────────────────────────────────────────── */

function TransactionsPageContent() {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [loadingTx, setLoadingTx] = useState(true);
  const [search, setSearch] = useState('');
  const [typeFilter, setTypeFilter] = useState<TxType | 'all'>('all');
  const [statusFilter, setStatusFilter] = useState<TxStatus | 'all'>('all');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [page, setPage] = useState(1);

  useEffect(() => {
    apiClient
      .get<Transaction[]>('/transactions')
      .then((data) => setTransactions(data ?? []))
      .catch(() => setTransactions([]))
      .finally(() => setLoadingTx(false));
  }, []);

  const filtered = useMemo(() => {
    return transactions.filter((tx) => {
      if (typeFilter !== 'all' && tx.type !== typeFilter) return false;
      if (statusFilter !== 'all' && tx.status !== statusFilter) return false;
      if (
        search &&
        !tx.recipient.toLowerCase().includes(search.toLowerCase()) &&
        !tx.txHash.toLowerCase().includes(search.toLowerCase())
      )
        return false;
      if (dateFrom && tx.date < dateFrom) return false;
      if (dateTo && tx.date > dateTo + 'T23:59:59Z') return false;
      return true;
    });
  }, [transactions, search, typeFilter, statusFilter, dateFrom, dateTo]);

  const totalPages = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
  const currentPage = Math.min(page, totalPages);
  const paginated = filtered.slice(
    (currentPage - 1) * PAGE_SIZE,
    currentPage * PAGE_SIZE
  );

  function resetFilters() {
    setSearch('');
    setTypeFilter('all');
    setStatusFilter('all');
    setDateFrom('');
    setDateTo('');
    setPage(1);
  }

  const hasActiveFilters =
    search ||
    typeFilter !== 'all' ||
    statusFilter !== 'all' ||
    dateFrom ||
    dateTo;

  return (
    <main className="mx-auto max-w-5xl px-6 py-10">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight">
          Transaction History
        </h1>
        <p className="mt-1 text-sm text-[var(--color-text-muted)]">
          All your donations, pool creations, and withdrawals
        </p>
      </div>

      {/* Filters */}
      <section
        aria-label="Transaction filters"
        className="mb-6 flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-end"
      >
        {/* Search */}
        <div className="relative flex-1 min-w-48">
          <span className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]">
            <SearchIcon />
          </span>
          <input
            type="search"
            placeholder="Search by recipient or hash…"
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setPage(1);
            }}
            className="w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] py-2 pl-9 pr-4 text-sm placeholder:text-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-brand-500"
            aria-label="Search transactions"
          />
        </div>

        {/* Type filter */}
        <select
          value={typeFilter}
          onChange={(e) => {
            setTypeFilter(e.target.value as TxType | 'all');
            setPage(1);
          }}
          className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
          aria-label="Filter by transaction type"
        >
          <option value="all">All Types</option>
          <option value="donation">Donation</option>
          <option value="pool_creation">Pool Created</option>
          <option value="withdrawal">Withdrawal</option>
        </select>

        {/* Status filter */}
        <select
          value={statusFilter}
          onChange={(e) => {
            setStatusFilter(e.target.value as TxStatus | 'all');
            setPage(1);
          }}
          className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
          aria-label="Filter by status"
        >
          <option value="all">All Statuses</option>
          <option value="completed">Completed</option>
          <option value="pending">Pending</option>
          <option value="failed">Failed</option>
        </select>

        {/* Date range */}
        <input
          type="date"
          value={dateFrom}
          onChange={(e) => {
            setDateFrom(e.target.value);
            setPage(1);
          }}
          className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
          aria-label="From date"
        />
        <input
          type="date"
          value={dateTo}
          onChange={(e) => {
            setDateTo(e.target.value);
            setPage(1);
          }}
          className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
          aria-label="To date"
        />

        {hasActiveFilters && (
          <button
            onClick={resetFilters}
            className="rounded-xl border border-[var(--color-border)] px-3 py-2 text-sm text-[var(--color-text-muted)] hover:bg-[var(--color-surface-raised)] transition-colors"
            aria-label="Clear all filters"
          >
            Clear
          </button>
        )}
      </section>

      {/* Results count */}
      <p className="mb-3 text-xs text-[var(--color-text-muted)]">
        {filtered.length} transaction{filtered.length !== 1 ? 's' : ''}
      </p>

      {/* Transaction list */}
      {loadingTx ? (
        <TransactionsSkeleton />
      ) : paginated.length === 0 ? (
        <EmptyState
          icon="transaction"
          iconTone="muted"
          title={
            hasActiveFilters
              ? 'No matching transactions'
              : 'No transactions yet'
          }
          description={
            hasActiveFilters
              ? 'Try adjusting your filters or search term.'
              : 'Your donations, pool creations, and withdrawals will appear here.'
          }
          action={
            hasActiveFilters
              ? {
                  label: 'Clear filters',
                  onClick: resetFilters,
                  variant: 'secondary',
                }
              : {
                  label: 'Browse Pools',
                  href: '/pools',
                }
          }
          steps={
            hasActiveFilters
              ? undefined
              : [
                  { text: 'Donate to a pool on the Browse Pools page' },
                  { text: 'Create your own pool from the dashboard' },
                  { text: 'Track every on-chain transaction here' },
                ]
          }
        />
      ) : (
        <>
          <ul
            className="flex flex-col gap-3"
            role="list"
            aria-label="Transactions"
          >
            {paginated.map((tx) => (
              <TransactionRow key={tx.id} tx={tx} />
            ))}
          </ul>

          {/* Pagination */}
          {totalPages > 1 && (
            <nav
              aria-label="Pagination"
              className="mt-6 flex items-center justify-center gap-2"
            >
              <button
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                disabled={currentPage === 1}
                className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-sm disabled:opacity-40 hover:bg-[var(--color-surface-raised)] transition-colors"
                aria-label="Previous page"
              >
                ←
              </button>
              {Array.from({ length: totalPages }, (_, i) => i + 1).map((n) => (
                <button
                  key={n}
                  onClick={() => setPage(n)}
                  aria-current={n === currentPage ? 'page' : undefined}
                  className={`rounded-lg px-3 py-1.5 text-sm transition-colors ${
                    n === currentPage
                      ? 'bg-brand-600 text-white'
                      : 'border border-[var(--color-border)] hover:bg-[var(--color-surface-raised)]'
                  }`}
                >
                  {n}
                </button>
              ))}
              <button
                onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                disabled={currentPage === totalPages}
                className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-sm disabled:opacity-40 hover:bg-[var(--color-surface-raised)] transition-colors"
                aria-label="Next page"
              >
                →
              </button>
            </nav>
          )}
        </>
      )}
    </main>
  );
}

/* ── TransactionsSkeleton ───────────────────────────────────────────────── */

function TransactionsSkeleton() {
  return (
    <ul
      className="flex flex-col gap-3"
      aria-label="Loading transactions"
      aria-busy="true"
    >
      {[1, 2, 3, 4, 5].map((i) => (
        <li
          key={i}
          className="h-20 animate-pulse rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)]"
        />
      ))}
    </ul>
  );
}

/* ── TransactionRow ─────────────────────────────────────────────────────── */

import { CopyButton } from '@/components/CopyButton';

const STELLAR_EXPERT_BASE = 'https://stellar.expert/explorer/testnet/tx';

function ExternalLinkIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-3"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M13.5 6H5.25A2.25 2.25 0 0 0 3 8.25v10.5A2.25 2.25 0 0 0 5.25 21h10.5A2.25 2.25 0 0 0 18 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25"
      />
    </svg>
  );
}

function TransactionRow({ tx }: { tx: Transaction }) {
  const explorerUrl = `${STELLAR_EXPERT_BASE}/${tx.txHash}`;

  return (
    <li className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 transition-shadow hover:shadow-sm">
      <div className="flex items-start gap-4">
        {/* Type icon */}
        <div
          className={`flex size-9 flex-shrink-0 items-center justify-center rounded-full ${TYPE_ICON_BG[tx.type]}`}
          aria-hidden="true"
        >
          {TYPE_ICON[tx.type]}
        </div>

        {/* Main content */}
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex flex-wrap items-center gap-2">
              <span className="font-medium text-sm">
                {TYPE_LABELS[tx.type]}
              </span>
              <StatusBadge status={tx.status} />
            </div>
            {tx.amount !== '0' && (
              <span className="font-semibold text-sm tabular-nums">
                {tx.amount} {tx.asset}
              </span>
            )}
          </div>

          <p className="mt-0.5 text-sm text-[var(--color-text-muted)] truncate">
            {tx.recipient}
          </p>

          <div className="mt-1.5 flex flex-wrap gap-3 text-xs text-[var(--color-text-muted)]">
            <time dateTime={tx.date}>
              {formatDate(tx.date)} · {formatTime(tx.date)}
            </time>
            <span className="inline-flex items-center gap-1.5">
              <span className="font-mono truncate max-w-32" title={tx.txHash}>
                {tx.txHash.slice(0, 8)}…
              </span>
              <CopyButton
                text={tx.txHash}
                iconOnly
                aria-label={`Copy transaction hash ${tx.txHash}`}
              />
              <a
                href={explorerUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-0.5 text-brand-600 hover:text-brand-700 hover:underline focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-brand-600 rounded"
                aria-label={`View transaction ${tx.txHash.slice(0, 8)} on Stellar Expert blockchain explorer (opens in new tab)`}
              >
                Explorer
                <ExternalLinkIcon />
              </a>
            </span>
          </div>
        </div>
      </div>
    </li>
  );
}

/* ── StatusBadge ────────────────────────────────────────────────────────── */

function StatusBadge({ status }: { status: TxStatus }) {
  return (
    <span
      className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium capitalize ${STATUS_STYLES[status]}`}
      aria-label={`Status: ${status}`}
    >
      {status}
    </span>
  );
}

export default function TransactionsPage() {
  return (
    <ProtectedRoute>
      <TransactionsPageContent />
    </ProtectedRoute>
  );
}
