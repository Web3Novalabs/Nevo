'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { EmptyState } from '@/components/EmptyState';
import { useDonationsStore, Donation } from '@/src/store';
import { CopyButton } from '@/components/CopyButton';
import { fetchMyDonations } from '@/lib/api-client';

const PAGE_SIZE = 8;

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

function csvEscape(val: string) {
  if (val.includes(',') || val.includes('\n') || val.includes('"')) {
    return '"' + val.replace(/"/g, '""') + '"';
  }
  return val;
}

export default function DonationsPage() {
  const history = useDonationsStore((s) => s.history);
  const addDonation = useDonationsStore((s) => s.addDonation);
  const [search, setSearch] = useState('');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [page, setPage] = useState(1);
  const [sortBy, setSortBy] = useState<'date' | 'amount'>('date');
  const [sortDir, setSortDir] = useState<'desc' | 'asc'>('desc');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchMyDonations()
      .then((donations) => {
        donations.forEach((d) => {
          if (!history.some((h) => h.id === d.id)) {
            addDonation(d);
          }
        });
      })
      .catch(() => {
        // API unavailable — fall back to local store data
      })
      .finally(() => setLoading(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const filtered = useMemo(() => {
    return history
      .filter((d) => {
        if (search && !d.poolName.toLowerCase().includes(search.toLowerCase()))
          return false;
        if (dateFrom && d.timestamp < dateFrom) return false;
        if (dateTo && d.timestamp > dateTo + 'T23:59:59Z') return false;
        return true;
      })
      .slice()
      .sort((a, b) => {
        if (sortBy === 'date') {
          const av = new Date(a.timestamp).getTime();
          const bv = new Date(b.timestamp).getTime();
          return sortDir === 'desc' ? bv - av : av - bv;
        }
        const an = parseFloat(a.amount || '0');
        const bn = parseFloat(b.amount || '0');
        return sortDir === 'desc' ? bn - an : an - bn;
      });
  }, [history, search, dateFrom, dateTo, sortBy, sortDir]);

  const totalPages = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
  const currentPage = Math.min(page, totalPages);
  const paginated = filtered.slice(
    (currentPage - 1) * PAGE_SIZE,
    currentPage * PAGE_SIZE
  );

  function resetFilters() {
    setSearch('');
    setDateFrom('');
    setDateTo('');
    setPage(1);
  }

  function exportCSV() {
    const rows = [
      [
        'Pool ID',
        'Pool Name',
        'Amount',
        'Asset',
        'Date',
        'Time',
        'TxHash',
        'Status',
      ],
    ];
    for (const d of filtered) {
      rows.push([
        d.poolId,
        d.poolName,
        d.amount,
        d.asset,
        formatDate(d.timestamp),
        formatTime(d.timestamp),
        d.txHash,
        d.status,
      ]);
    }
    const csv = rows
      .map((r) => r.map((c) => csvEscape(String(c))).join(','))
      .join('\n');
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'donations-history.csv';
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
  }

  const hasActiveFilters = search || dateFrom || dateTo;

  return (
    <main className="mx-auto max-w-5xl px-6 py-10">
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight">Donation History</h1>
        <p className="mt-1 text-sm text-[var(--color-text-muted)]">
          All your donations on Nevo
        </p>
      </div>

      {loading ? (
        <ul
          className="flex flex-col gap-3"
          aria-busy="true"
          aria-label="Loading donations"
        >
          {Array.from({ length: 5 }).map((_, i) => (
            <DonationSkeletonRow key={i} />
          ))}
        </ul>
      ) : (
        <>
          <section
            aria-label="Donation filters"
            className="mb-6 flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-end"
          >
            <input
              type="search"
              placeholder="Search by pool name…"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setPage(1);
              }}
              className="w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] py-2 px-4 text-sm placeholder:text-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-brand-500"
              aria-label="Search pools"
            />

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

            <div className="flex gap-2">
              <select
                value={sortBy}
                onChange={(e) => {
                  setSortBy(e.target.value as 'date' | 'amount');
                  setPage(1);
                }}
                className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm"
                aria-label="Sort by"
              >
                <option value="date">Date</option>
                <option value="amount">Amount</option>
              </select>
              <button
                onClick={() =>
                  setSortDir((s) => (s === 'desc' ? 'asc' : 'desc'))
                }
                className="rounded-xl border border-[var(--color-border)] px-3 py-2 text-sm"
                aria-label="Toggle sort direction"
              >
                {sortDir === 'desc' ? 'Desc' : 'Asc'}
              </button>
            </div>

            {hasActiveFilters && (
              <button
                onClick={resetFilters}
                className="rounded-xl border border-[var(--color-border)] px-3 py-2 text-sm text-[var(--color-text-muted)]"
              >
                Clear
              </button>
            )}
          </section>

          <div className="mb-3 flex items-center justify-between gap-4">
            <p className="text-xs text-[var(--color-text-muted)]">
              {filtered.length} donation{filtered.length !== 1 ? 's' : ''}
            </p>
            <div className="flex gap-2">
              <button
                onClick={exportCSV}
                className="rounded-xl border border-[var(--color-border)] px-3 py-2 text-sm"
              >
                Export CSV
              </button>
            </div>
          </div>

          {paginated.length === 0 ? (
            <EmptyState
              icon="history"
              iconTone="muted"
              title={
                hasActiveFilters
                  ? 'No matching donations'
                  : "You haven't donated to any pools yet"
              }
              description={
                hasActiveFilters
                  ? 'Try adjusting your filters or search term.'
                  : 'Your donations will appear here.'
              }
              action={
                hasActiveFilters
                  ? {
                      label: 'Clear filters',
                      onClick: resetFilters,
                      variant: 'secondary',
                    }
                  : { label: 'Browse Pools', href: '/pools' }
              }
            />
          ) : (
            <>
              <ul
                className="flex flex-col gap-3"
                role="list"
                aria-label="Donations"
              >
                {paginated.map((d) => (
                  <DonationRow key={d.id} d={d} />
                ))}
              </ul>

              {totalPages > 1 && (
                <nav
                  aria-label="Pagination"
                  className="mt-6 flex items-center justify-center gap-2"
                >
                  <button
                    onClick={() => setPage((p) => Math.max(1, p - 1))}
                    disabled={currentPage === 1}
                    className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-sm disabled:opacity-40"
                  >
                    ←
                  </button>
                  {Array.from({ length: totalPages }, (_, i) => i + 1).map(
                    (n) => (
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
                    )
                  )}
                  <button
                    onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                    disabled={currentPage === totalPages}
                    className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-sm disabled:opacity-40"
                  >
                    →
                  </button>
                </nav>
              )}
            </>
          )}
        </>
      )}
    </main>
  );
}

function DonationRow({ d }: { d: Donation }) {
  const explorer = `https://stellar.expert/explorer/public/tx/${d.txHash}`;

  return (
    <li className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 transition-shadow hover:shadow-sm">
      <div className="flex items-start gap-4">
        <div
          className="flex size-9 flex-shrink-0 items-center justify-center rounded-full bg-brand-100 text-brand-600"
          aria-hidden
        >
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
        </div>

        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex flex-wrap items-center gap-2">
              <a
                href={`/pools/${d.poolId}`}
                className="font-medium text-sm hover:underline truncate"
              >
                {d.poolName}
              </a>
              <span className="text-xs text-[var(--color-text-muted)]">
                · {formatDate(d.timestamp)}
              </span>
            </div>

            <div className="text-right">
              <div className="font-semibold text-sm tabular-nums">
                {d.amount} {d.asset}
              </div>
              <div className="mt-1 text-xs text-[var(--color-text-muted)] flex items-center gap-1">
                <a
                  href={explorer}
                  target="_blank"
                  rel="noreferrer"
                  className="font-mono truncate max-w-40 inline-block hover:underline"
                >
                  {d.txHash.slice(0, 10)}…
                </a>
                <CopyButton
                  text={d.txHash}
                  iconOnly
                  aria-label={`Copy transaction hash ${d.txHash}`}
                />
              </div>
            </div>
          </div>

          <p className="mt-1 text-sm text-[var(--color-text-muted)]">
            Status: <span className="font-medium capitalize">{d.status}</span>
          </p>
        </div>
      </div>
    </li>
  );
}

function DonationSkeletonRow() {
  return (
    <li className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 animate-pulse">
      <div className="flex items-start gap-4">
        <div className="flex size-9 flex-shrink-0 rounded-full bg-[var(--color-border)]" />
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex flex-wrap items-center gap-2">
              <div className="h-4 w-32 rounded bg-[var(--color-border)]" />
              <div className="h-3 w-16 rounded bg-[var(--color-border)]" />
            </div>
            <div className="text-right">
              <div className="h-4 w-20 rounded bg-[var(--color-border)] mb-1" />
              <div className="h-3 w-24 rounded bg-[var(--color-border)]" />
            </div>
          </div>
          <div className="mt-2 h-3 w-32 rounded bg-[var(--color-border)]" />
        </div>
      </div>
    </li>
  );
}
