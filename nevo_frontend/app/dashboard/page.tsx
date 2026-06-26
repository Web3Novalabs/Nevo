'use client';

import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { useWalletStore } from '@/src/store/walletStore';
import { EmptyState } from '@/components/EmptyState';
import { WalletAddress } from '@/components/WalletAddress';
import ProtectedRoute from '@/components/ProtectedRoute';
import { apiClient } from '@/lib/api-client';
import type { Pool } from '@/src/store/poolsStore';

type ActionModal =
  | { type: 'withdraw'; pool: Pool }
  | { type: 'archive'; pool: Pool }
  | null;

function DashboardPageContent() {
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
    apiClient
      .get<Pool[]>(`/pools?creator=${publicKey}`)
      .then((data) => setPools(data ?? []))
      .catch(() => setPools([]))
      .finally(() => setLoadingPools(false));
  }, [publicKey]);

  // ── Summary metrics ────────────────────────────────────────────────────
  const totalRaised = pools.reduce((s, p) => s + p.raised, 0);
  const activePools = pools.filter((p) => p.status === 'Active').length;

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
        <EmptyState
          icon="pool"
          title="You haven't created any pools yet"
          description="Create your first pool to start raising funds on-chain."
          action={{ label: 'Create Pool', href: '/pools/new' }}
          steps={[
            { text: 'Set a title, goal, and category for your cause' },
            { text: 'Share your pool link with supporters' },
            { text: 'Withdraw funds once your goal is reached' },
          ]}
        />
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
  selected: boolean;
  onToggle: () => void;
  onWithdraw: () => void;
  onArchive: () => void;
}

function PoolRow({
  pool,
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

export default function DashboardPage() {
  return (
    <ProtectedRoute>
      <DashboardPageContent />
    </ProtectedRoute>
  );
}
