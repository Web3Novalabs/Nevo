'use client';

import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { useWalletStore } from '@/src/store/walletStore';
import { WalletAddress } from '@/components/WalletAddress';
import type { Pool } from '@/src/store/poolsStore';

// TODO: Replace with real API call once backend pool endpoints are implemented
const MOCK_CREATOR_POOLS: Pool[] = [
  {
    id: '1',
    title: 'Clean Water Initiative',
    description: 'Providing clean drinking water to rural communities in need.',
    category: 'Humanitarian',
    status: 'Active',
    target: 10000,
    raised: 6800,
    imageColor: '#27926e',
    creator: 'GABCDE1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890',
    createdAt: '2025-03-01',
  },
  {
    id: '2',
    title: 'Open Source Dev Fund',
    description: 'Supporting open source contributors building on Stellar.',
    category: 'Technology',
    status: 'Active',
    target: 5000,
    raised: 5000,
    imageColor: '#1c7459',
    creator: 'GABCDE1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890',
    createdAt: '2025-01-15',
  },
  {
    id: '3',
    title: 'Community Garden Project',
    description: 'Building urban gardens to improve food security locally.',
    category: 'Environment',
    status: 'Completed',
    target: 3000,
    raised: 3200,
    imageColor: '#47ae88',
    creator: 'GABCDE1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890',
    createdAt: '2024-11-10',
  },
];

// TODO: Replace with real contributor counts from backend
const MOCK_CONTRIBUTOR_COUNTS: Record<string, number> = {
  '1': 42,
  '2': 87,
  '3': 31,
};

type ActionModal =
  | { type: 'withdraw'; pool: Pool }
  | { type: 'archive'; pool: Pool }
  | null;

export default function DashboardPage() {
  const { publicKey, loading, initialize } = useWalletStore();
  const [pools, setPools] = useState<Pool[]>([]);
  const [loadingPools, setLoadingPools] = useState(true);
  const [actionModal, setActionModal] = useState<ActionModal>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  useEffect(() => {
    initialize();
  }, [initialize]);

  useEffect(() => {
    if (!publicKey) return;
    // TODO: Replace with real fetch filtered by creator === publicKey
    const timer = setTimeout(() => {
      setPools(MOCK_CREATOR_POOLS);
      setLoadingPools(false);
    }, 400);
    return () => clearTimeout(timer);
  }, [publicKey]);

  // ── Wallet not connected ───────────────────────────────────────────────
  if (!loading && !publicKey) {
    return (
      <main className="mx-auto max-w-5xl px-6 py-24 text-center">
        <div className="flex flex-col items-center gap-4">
          <div className="flex size-16 items-center justify-center rounded-full bg-brand-100 text-brand-600">
            <LockIcon />
          </div>
          <h1 className="text-2xl font-bold">Connect your wallet</h1>
          <p className="text-[var(--color-text-muted)] max-w-sm">
            Your dashboard is only accessible to pool creators. Connect your
            Stellar wallet to continue.
          </p>
        </div>
      </main>
    );
  }

  // ── Summary metrics ────────────────────────────────────────────────────
  const totalRaised = pools.reduce((s, p) => s + p.raised, 0);
  const activePools = pools.filter((p) => p.status === 'Active').length;
  const totalContributors = Object.values(MOCK_CONTRIBUTOR_COUNTS).reduce(
    (s, n) => s + n,
    0
  );

  // ── Bulk selection helpers ─────────────────────────────────────────────
  const allSelected = pools.length > 0 && selectedIds.size === pools.length;

  function toggleAll() {
    setSelectedIds(allSelected ? new Set() : new Set(pools.map((p) => p.id)));
  }

  function toggleOne(id: string) {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }

  return (
    <main className="mx-auto max-w-5xl px-6 py-10">
      {/* ── Page header ─────────────────────────────────────────────────── */}
      <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">My Pools</h1>
          {publicKey && (
            <div className="mt-1">
              <WalletAddress address={publicKey} className="text-xs" />
            </div>
          )}
        </div>
        <Link
          href="/pools/new"
          className="inline-flex items-center gap-2 rounded-full bg-brand-600 px-5 py-2 text-sm font-semibold text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
          aria-label="Create a new pool"
        >
          <PlusIcon />
          New Pool
        </Link>
      </div>

      {/* ── Summary stats ───────────────────────────────────────────────── */}
      <section
        aria-label="Summary statistics"
        className="mb-8 grid grid-cols-2 gap-4 sm:grid-cols-4"
      >
        {[
          { label: 'Total Pools', value: pools.length },
          { label: 'Active Pools', value: activePools },
          {
            label: 'Total Raised',
            value: `${totalRaised.toLocaleString()} XLM`,
          },
          { label: 'Contributors', value: totalContributors },
        ].map(({ label, value }) => (
          <div
            key={label}
            className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-4"
          >
            <p className="text-xs text-[var(--color-text-muted)]">{label}</p>
            <p className="mt-1 text-xl font-bold text-brand-600">{value}</p>
          </div>
        ))}
      </section>

      {/* ── Bulk action bar ─────────────────────────────────────────────── */}
      {selectedIds.size > 0 && (
        <div
          role="toolbar"
          aria-label="Bulk actions"
          className="mb-4 flex items-center gap-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] px-4 py-2 text-sm"
        >
          <span className="text-[var(--color-text-muted)]">
            {selectedIds.size} selected
          </span>
          <button
            onClick={() => setSelectedIds(new Set())}
            className="ml-auto text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
            aria-label="Clear selection"
          >
            Clear
          </button>
          <button
            className="rounded-lg border border-[var(--color-border)] px-3 py-1 hover:bg-[var(--color-border)] transition-colors"
            aria-label="Archive selected pools"
            onClick={() => {
              // TODO: bulk archive action
              setSelectedIds(new Set());
            }}
          >
            Archive selected
          </button>
        </div>
      )}

      {/* ── Pool list ───────────────────────────────────────────────────── */}
      {loading || loadingPools ? (
        <PoolListSkeleton />
      ) : pools.length === 0 ? (
        <EmptyState />
      ) : (
        <section aria-label="Your pools">
          {/* Table header — desktop only */}
          <div className="mb-2 hidden grid-cols-[auto_1fr_auto_auto_auto] items-center gap-4 px-4 text-xs text-[var(--color-text-muted)] sm:grid">
            <input
              type="checkbox"
              checked={allSelected}
              onChange={toggleAll}
              aria-label="Select all pools"
              className="size-4 rounded accent-brand-600"
            />
            <span>Pool</span>
            <span className="text-right">Progress</span>
            <span className="text-right">Status</span>
            <span className="text-right">Actions</span>
          </div>

          <ul className="flex flex-col gap-3" role="list">
            {pools.map((pool) => (
              <PoolRow
                key={pool.id}
                pool={pool}
                contributors={MOCK_CONTRIBUTOR_COUNTS[pool.id] ?? 0}
                selected={selectedIds.has(pool.id)}
                onToggle={() => toggleOne(pool.id)}
                onWithdraw={() => setActionModal({ type: 'withdraw', pool })}
                onArchive={() => setActionModal({ type: 'archive', pool })}
              />
            ))}
          </ul>
        </section>
      )}

      {/* ── Confirmation modal ──────────────────────────────────────────── */}
      {actionModal && (
        <ConfirmModal
          modal={actionModal}
          onClose={() => setActionModal(null)}
          onConfirm={() => {
            // TODO: wire to real withdraw / archive contract calls
            setActionModal(null);
          }}
        />
      )}
    </main>
  );
}

/* ── PoolRow ──────────────────────────────────────────────────────────────── */

interface PoolRowProps {
  pool: Pool;
  contributors: number;
  selected: boolean;
  onToggle: () => void;
  onWithdraw: () => void;
  onArchive: () => void;
}

function PoolRow({
  pool,
  contributors,
  selected,
  onToggle,
  onWithdraw,
  onArchive,
}: PoolRowProps) {
  const pct = Math.min(100, Math.round((pool.raised / pool.target) * 100));
  const isCompleted = pool.status === 'Completed';

  return (
    <li className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 transition-shadow hover:shadow-sm">
      <div className="flex flex-col gap-4 sm:grid sm:grid-cols-[auto_1fr_auto] sm:items-center sm:gap-4">
        {/* Checkbox + title */}
        <div className="flex items-start gap-3 sm:contents">
          <input
            type="checkbox"
            checked={selected}
            onChange={onToggle}
            aria-label={`Select pool: ${pool.title}`}
            className="mt-1 size-4 flex-shrink-0 rounded accent-brand-600 sm:mt-0"
          />

          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2">
              <Link
                href={`/pools/${pool.id}`}
                className="font-semibold hover:text-brand-600 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
                aria-label={`View pool: ${pool.title}`}
              >
                {pool.title}
              </Link>
              <StatusBadge status={pool.status} />
            </div>
            <p className="mt-0.5 text-sm text-[var(--color-text-muted)] line-clamp-1">
              {pool.description}
            </p>

            {/* Progress */}
            <div className="mt-3">
              <div className="mb-1 flex items-center justify-between text-xs text-[var(--color-text-muted)]">
                <span>
                  {pool.raised.toLocaleString()} /{' '}
                  {pool.target.toLocaleString()} XLM
                </span>
                <span>{pct}%</span>
              </div>
              <div
                className="h-1.5 w-full overflow-hidden rounded-full bg-[var(--color-border)]"
                role="progressbar"
                aria-valuenow={pct}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-label={`${pool.title} progress: ${pct}%`}
              >
                <div
                  className="h-full rounded-full bg-brand-500 transition-all"
                  style={{ width: `${pct}%` }}
                />
              </div>
            </div>

            {/* Meta row */}
            <div className="mt-2 flex flex-wrap gap-3 text-xs text-[var(--color-text-muted)]">
              <span>{contributors} contributors</span>
              <span>{pool.category}</span>
              {pool.createdAt && <span>Created {pool.createdAt}</span>}
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex flex-wrap items-center gap-2 sm:flex-col sm:items-end">
          <Link
            href={`/pools/${pool.id}/edit`}
            className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-xs font-medium hover:bg-[var(--color-surface-raised)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
            aria-label={`Edit pool: ${pool.title}`}
          >
            Edit
          </Link>
          <button
            onClick={onWithdraw}
            disabled={!isCompleted}
            className="rounded-lg bg-brand-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-brand-700 transition-colors disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
            aria-label={
              isCompleted
                ? `Withdraw funds from: ${pool.title}`
                : `Withdraw unavailable — pool is not completed`
            }
          >
            Withdraw
          </button>
          <button
            onClick={onArchive}
            className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-xs font-medium text-error hover:bg-error-light transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-error"
            aria-label={`Archive pool: ${pool.title}`}
          >
            Archive
          </button>
        </div>
      </div>
    </li>
  );
}

/* ── StatusBadge ──────────────────────────────────────────────────────────── */

function StatusBadge({ status }: { status: Pool['status'] }) {
  const styles =
    status === 'Active'
      ? 'bg-success-light text-success-dark'
      : 'bg-[var(--color-surface-raised)] text-[var(--color-text-muted)] border border-[var(--color-border)]';

  return (
    <span
      className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${styles}`}
      aria-label={`Pool status: ${status}`}
    >
      {status}
    </span>
  );
}

/* ── ConfirmModal ─────────────────────────────────────────────────────────── */

function ConfirmModal({
  modal,
  onClose,
  onConfirm,
}: {
  modal: NonNullable<ActionModal>;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const isWithdraw = modal.type === 'withdraw';

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
    >
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/40"
        onClick={onClose}
        aria-hidden="true"
      />

      <div className="relative w-full max-w-sm rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xl">
        <h2 id="modal-title" className="text-base font-semibold">
          {isWithdraw ? 'Withdraw funds' : 'Archive pool'}
        </h2>
        <p className="mt-2 text-sm text-[var(--color-text-muted)]">
          {isWithdraw
            ? `Withdraw all raised funds from "${modal.pool.title}"? This action is irreversible on-chain.`
            : `Archive "${modal.pool.title}"? The pool will no longer accept donations.`}
        </p>
        <div className="mt-5 flex justify-end gap-3">
          <button
            onClick={onClose}
            className="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm hover:bg-[var(--color-surface-raised)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            className={`rounded-lg px-4 py-2 text-sm font-medium text-white transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 ${isWithdraw ? 'bg-brand-600 hover:bg-brand-700 focus-visible:outline-brand-600' : 'bg-error hover:bg-error-dark focus-visible:outline-error'}`}
          >
            {isWithdraw ? 'Confirm Withdraw' : 'Archive'}
          </button>
        </div>
      </div>
    </div>
  );
}

/* ── Skeleton ─────────────────────────────────────────────────────────────── */

function PoolListSkeleton() {
  return (
    <ul
      className="flex flex-col gap-3"
      aria-label="Loading pools"
      aria-busy="true"
    >
      {[1, 2, 3].map((i) => (
        <li
          key={i}
          className="h-32 animate-pulse rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)]"
        />
      ))}
    </ul>
  );
}

/* ── Empty state ──────────────────────────────────────────────────────────── */

function EmptyState() {
  return (
    <div className="flex flex-col items-center gap-4 py-20 text-center">
      <div className="flex size-14 items-center justify-center rounded-full bg-brand-100 text-brand-600">
        <PoolIcon />
      </div>
      <p className="font-semibold">No pools yet</p>
      <p className="text-sm text-[var(--color-text-muted)]">
        Create your first pool to start raising funds on-chain.
      </p>
      <Link
        href="/pools/new"
        className="rounded-full bg-brand-600 px-6 py-2 text-sm font-semibold text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
      >
        Create a Pool
      </Link>
    </div>
  );
}

/* ── Icons ────────────────────────────────────────────────────────────────── */

function PlusIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={2}
      stroke="currentColor"
      className="size-4"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M12 4.5v15m7.5-7.5h-15"
      />
    </svg>
  );
}

function LockIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-7"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 0 0 2.25-2.25v-6.75a2.25 2.25 0 0 0-2.25-2.25H6.75a2.25 2.25 0 0 0-2.25 2.25v6.75a2.25 2.25 0 0 0 2.25 2.25Z"
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
      className="size-7"
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
