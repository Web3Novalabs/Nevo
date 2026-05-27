'use client';

import React, { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';
import { useWalletStore } from '@/src/store/walletStore';
import { DonateModal } from '@/components/DonateModal';
import type { Pool } from '@/src/store/poolsStore';

// TODO: Replace with real API call once backend pool endpoints are implemented
const MOCK_POOLS: Pool[] = [
  {
    id: '1',
    title: 'Clean Water Initiative',
    description:
      'Providing clean drinking water to rural communities in need. Every contribution helps us build wells and water purification systems in underserved areas.',
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
    description:
      'Supporting open source contributors building on Stellar. Funds go directly to developers maintaining critical infrastructure.',
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
    description:
      'Building urban gardens to improve food security locally. We partner with city councils to transform unused land into productive green spaces.',
    category: 'Environment',
    status: 'Completed',
    target: 3000,
    raised: 3200,
    imageColor: '#47ae88',
    creator: 'GABCDE1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890',
    createdAt: '2024-11-10',
  },
];

export default function PoolDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { publicKey, initialize } = useWalletStore();
  const [pool, setPool] = useState<Pool | null>(null);
  const [loading, setLoading] = useState(true);
  const [donateOpen, setDonateOpen] = useState(false);

  useEffect(() => {
    initialize();
  }, [initialize]);

  useEffect(() => {
    // TODO: Replace with real fetch by pool id
    const timer = setTimeout(() => {
      setPool(MOCK_POOLS.find((p) => p.id === id) ?? null);
      setLoading(false);
    }, 300);
    return () => clearTimeout(timer);
  }, [id]);

  if (loading) return <PoolDetailSkeleton />;

  if (!pool) {
    return (
      <main className="mx-auto max-w-3xl px-6 py-24 text-center">
        <p className="text-lg font-semibold">Pool not found</p>
        <Link
          href="/pools"
          className="mt-4 inline-block text-sm text-brand-600 hover:underline"
        >
          ← Back to pools
        </Link>
      </main>
    );
  }

  const pct = Math.min(100, Math.round((pool.raised / pool.target) * 100));
  const isActive = pool.status === 'Active';

  return (
    <main className="mx-auto max-w-3xl px-6 py-10">
      {/* Back link */}
      <Link
        href="/pools"
        className="mb-6 inline-flex items-center gap-1.5 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
        aria-label="Back to pools list"
      >
        <ChevronLeftIcon />
        All Pools
      </Link>

      {/* Pool header */}
      <div
        className="mb-6 h-40 w-full rounded-2xl sm:h-52"
        style={{ backgroundColor: pool.imageColor }}
        aria-hidden="true"
      />

      <div className="flex flex-col gap-6 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h1 className="text-2xl font-bold tracking-tight">{pool.title}</h1>
            <StatusBadge status={pool.status} />
          </div>
          <p className="mt-1 text-sm text-[var(--color-text-muted)]">
            {pool.category}
            {pool.createdAt && ` · Created ${pool.createdAt}`}
          </p>
        </div>

        {/* Donate button */}
        <button
          type="button"
          onClick={() => setDonateOpen(true)}
          disabled={!isActive}
          aria-label={
            isActive
              ? `Donate to ${pool.title}`
              : 'This pool is no longer accepting donations'
          }
          className="flex-shrink-0 rounded-full bg-brand-600 px-6 py-2.5 text-sm font-semibold text-white hover:bg-brand-700 transition-colors disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
        >
          Donate
        </button>
      </div>

      {/* Progress */}
      <section
        aria-label="Fundraising progress"
        className="mt-6 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-5"
      >
        <div className="mb-2 flex items-end justify-between">
          <div>
            <span className="text-2xl font-bold text-brand-600">
              {pool.raised.toLocaleString()}
            </span>
            <span className="ml-1 text-sm text-[var(--color-text-muted)]">
              XLM raised
            </span>
          </div>
          <span className="text-sm text-[var(--color-text-muted)]">
            of {pool.target.toLocaleString()} XLM goal
          </span>
        </div>
        <div
          className="h-2 w-full overflow-hidden rounded-full bg-[var(--color-border)]"
          role="progressbar"
          aria-valuenow={pct}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label={`${pct}% of goal reached`}
        >
          <div
            className="h-full rounded-full bg-brand-500 transition-all"
            style={{ width: `${pct}%` }}
          />
        </div>
        <p className="mt-2 text-xs text-[var(--color-text-muted)]">
          {pct}% funded
        </p>
      </section>

      {/* Description */}
      <section aria-label="Pool description" className="mt-6">
        <h2 className="mb-2 text-base font-semibold">About this pool</h2>
        <p className="text-sm leading-relaxed text-[var(--color-text-muted)]">
          {pool.description}
        </p>
      </section>

      {/* Wallet prompt if not connected */}
      {!publicKey && isActive && (
        <p className="mt-6 text-center text-sm text-[var(--color-text-muted)]">
          Connect your wallet to donate to this pool.
        </p>
      )}

      {/* Donation modal */}
      {donateOpen && (
        <DonateModal pool={pool} onClose={() => setDonateOpen(false)} />
      )}
    </main>
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

/* ── Skeleton ─────────────────────────────────────────────────────────────── */

function PoolDetailSkeleton() {
  return (
    <main
      className="mx-auto max-w-3xl px-6 py-10"
      aria-busy="true"
      aria-label="Loading pool details"
    >
      <div className="mb-6 h-4 w-20 animate-pulse rounded bg-[var(--color-border)]" />
      <div className="mb-6 h-40 w-full animate-pulse rounded-2xl bg-[var(--color-border)] sm:h-52" />
      <div className="h-8 w-2/3 animate-pulse rounded bg-[var(--color-border)]" />
      <div className="mt-3 h-4 w-1/3 animate-pulse rounded bg-[var(--color-border)]" />
      <div className="mt-6 h-24 w-full animate-pulse rounded-2xl bg-[var(--color-border)]" />
    </main>
  );
}

/* ── Icons ────────────────────────────────────────────────────────────────── */

function ChevronLeftIcon() {
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
        d="M15.75 19.5 8.25 12l7.5-7.5"
      />
    </svg>
  );
}
