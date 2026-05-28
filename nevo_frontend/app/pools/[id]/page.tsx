'use client';

import Link from 'next/link';
import { useParams } from 'next/navigation';
import { useMemo, useState } from 'react';
import { useWalletStore } from '@/src/store/walletStore';
import type { Pool } from '@/src/store/poolsStore';
import { getAccountBalances } from '@/lib/stellar';

type WithdrawStatus = 'idle' | 'submitting' | 'success' | 'error';

import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { useParams } from 'next/navigation';
import { useWalletStore } from '@/src/store/walletStore';
import { WalletAddress } from '@/components/WalletAddress';
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
    description: 'Providing clean drinking water to rural communities in need.',
    category: 'Humanitarian',
    status: 'Completed',
    description:
      'Providing clean drinking water to rural communities in need. This project partners with local NGOs to install water purification systems and train community members in maintenance and operation.',
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
    description: 'Supporting open source contributors building on Stellar.',
    category: 'Technology',
    status: 'Completed',
    description:
      'Supporting open source contributors building on Stellar. Funds are distributed monthly to active maintainers based on contributions tracked via GitHub.',
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
    description: 'Building urban gardens to improve food security locally.',
    category: 'Environment',
    status: 'Active',
    target: 3000,
    raised: 1200,
    description:
      'Building urban gardens to improve food security locally. Each garden plot is maintained by a volunteer team and produces fresh produce distributed to local food banks.',
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
  const params = useParams<{ id: string }>();
  const { publicKey, refreshBalances } = useWalletStore();

  const pool = useMemo(
    () => MOCK_POOLS.find((item) => item.id === params.id),
    [params.id]
  );

  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [withdrawnAmount, setWithdrawnAmount] = useState(0);
  const [status, setStatus] = useState<WithdrawStatus>('idle');
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [receiptHash, setReceiptHash] = useState<string | null>(null);

  if (!pool) {
    return (
      <main className="mx-auto max-w-4xl px-6 py-12">
        <p className="text-sm text-[var(--color-text-muted)]">
          Pool not found.
        </p>
        <Link
          href="/dashboard"
          className="mt-4 inline-flex rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm hover:bg-[var(--color-surface-raised)] transition-colors"
        >
          Back to dashboard
// TODO: Replace with real contributor data from backend
interface Contributor {
  address: string;
  amount: number;
  donatedAt: string;
}

const MOCK_CONTRIBUTORS: Record<string, Contributor[]> = {
  '1': [
    { address: 'GXYZ1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890AB', amount: 500, donatedAt: '2025-03-05' },
    { address: 'GABC9876543210ZYXWV9876543210ZYXWV9876543210ZYXWV9876543210ZY', amount: 1200, donatedAt: '2025-03-12' },
    { address: 'GDEF5555555555GHIJK5555555555GHIJK5555555555GHIJK5555555555GH', amount: 300, donatedAt: '2025-03-18' },
    { address: 'GLMN7777777777OPQRS7777777777OPQRS7777777777OPQRS7777777777OP', amount: 800, donatedAt: '2025-04-02' },
  ],
  '2': [
    { address: 'GXYZ1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890AB', amount: 750, donatedAt: '2025-01-20' },
    { address: 'GABC9876543210ZYXWV9876543210ZYXWV9876543210ZYXWV9876543210ZY', amount: 2000, donatedAt: '2025-02-01' },
  ],
  '3': [
    { address: 'GDEF5555555555GHIJK5555555555GHIJK5555555555GHIJK5555555555GH', amount: 1000, donatedAt: '2024-11-15' },
    { address: 'GLMN7777777777OPQRS7777777777OPQRS7777777777OPQRS7777777777OP', amount: 900, donatedAt: '2024-12-01' },
    { address: 'GXYZ1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890AB', amount: 1300, donatedAt: '2024-12-20' },
  ],
};

// TODO: Replace with real timeline data from backend
interface TimelineEvent {
  id: string;
  label: string;
  date: string;
  amount?: number;
}

const MOCK_TIMELINE: Record<string, TimelineEvent[]> = {
  '1': [
    { id: 'e1', label: 'Pool created', date: '2025-03-01' },
    { id: 'e2', label: 'First donation received', date: '2025-03-05', amount: 500 },
    { id: 'e3', label: 'Milestone: 25% funded', date: '2025-03-12' },
    { id: 'e4', label: 'Milestone: 50% funded', date: '2025-04-02' },
  ],
  '2': [
    { id: 'e1', label: 'Pool created', date: '2025-01-15' },
    { id: 'e2', label: 'First donation received', date: '2025-01-20', amount: 750 },
    { id: 'e3', label: 'Goal reached', date: '2025-02-01' },
  ],
  '3': [
    { id: 'e1', label: 'Pool created', date: '2024-11-10' },
    { id: 'e2', label: 'First donation received', date: '2024-11-15', amount: 1000 },
    { id: 'e3', label: 'Goal reached', date: '2024-12-20' },
    { id: 'e4', label: 'Pool completed', date: '2024-12-31' },
  ],
};

// TODO: Replace with real last-updated timestamps from backend
const MOCK_LAST_UPDATED: Record<string, string> = {
  '1': '2025-04-15',
  '2': '2025-02-01',
  '3': '2024-12-31',
};

export default function PoolDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { publicKey, initialize } = useWalletStore();

  const [pool, setPool] = useState<Pool | null>(null);
  const [contributors, setContributors] = useState<Contributor[]>([]);
  const [timeline, setTimeline] = useState<TimelineEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [notFound, setNotFound] = useState(false);
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
    if (!id) return;
    // TODO: Replace with real fetch by pool ID
    const timer = setTimeout(() => {
      const found = MOCK_POOLS.find((p) => p.id === id) ?? null;
      if (!found) {
        setNotFound(true);
      } else {
        setPool(found);
        setContributors(MOCK_CONTRIBUTORS[id] ?? []);
        setTimeline(MOCK_TIMELINE[id] ?? []);
      }
    // TODO: Replace with real fetch by pool id
    const timer = setTimeout(() => {
      setPool(MOCK_POOLS.find((p) => p.id === id) ?? null);
      setLoading(false);
    }, 300);
    return () => clearTimeout(timer);
  }, [id]);

  if (loading) {
    return <PoolDetailSkeleton />;
  }

  if (notFound || !pool) {
    return (
      <main className="mx-auto max-w-3xl px-6 py-24 text-center">
        <div className="flex flex-col items-center gap-4">
          <div className="flex size-16 items-center justify-center rounded-full bg-[var(--color-surface-raised)] text-[var(--color-text-muted)]">
            <PoolIcon />
          </div>
          <h1 className="text-2xl font-bold">Pool not found</h1>
          <p className="text-[var(--color-text-muted)]">
            This pool does not exist or has been removed.
          </p>
          <Link
            href="/pools"
            className="mt-2 rounded-full bg-brand-600 px-6 py-2 text-sm font-semibold text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
          >
            Browse Pools
          </Link>
        </div>
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

  const availableBalance = Math.max(0, pool.raised - withdrawnAmount);
  const canWithdraw = pool.status === 'Completed' && availableBalance > 0;

  async function processWithdrawal() {
    if (!publicKey) {
      setErrorMessage('Connect your wallet before withdrawing.');
      setStatus('error');
      return;
    }
    if (!canWithdraw) {
      setErrorMessage('No available balance to withdraw.');
      setStatus('error');
      return;
    }

    setStatus('submitting');
    setErrorMessage(null);
    setReceiptHash(null);

    try {
      // TODO: Replace with real contract withdrawal invocation once available.
      await getAccountBalances(publicKey);
      const networkCheck = await fetch('https://horizon.stellar.org/');
      if (!networkCheck.ok) {
        throw new Error('Stellar network is unavailable right now.');
      }

      await new Promise((resolve) => setTimeout(resolve, 1500));

      const hash = `mock-${crypto.randomUUID().replace(/-/g, '').slice(0, 24)}`;
      setReceiptHash(hash);
      setWithdrawnAmount((prev) => prev + availableBalance);
      await refreshBalances();
      setStatus('success');
      setIsConfirmOpen(false);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Withdrawal failed unexpectedly.';
      setErrorMessage(message);
      setStatus('error');
    }
  }

  return (
    <main className="mx-auto max-w-4xl px-4 py-8 sm:px-6 sm:py-10">
      <div className="mb-4">
        <Link
          href="/dashboard"
          className="text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
        >
          Back to dashboard
        </Link>
      </div>

      <section className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 sm:p-6">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <h1 className="text-2xl font-bold">{pool.title}</h1>
            <p className="mt-2 text-sm text-[var(--color-text-muted)]">
              {pool.description}
            </p>
          </div>
          <span className="inline-flex w-fit rounded-full bg-[var(--color-surface-raised)] px-3 py-1 text-xs font-medium text-[var(--color-text-muted)]">
            {pool.status}
          </span>
        </div>

        <div className="mt-6 grid gap-4 sm:grid-cols-3">
          <Metric
            label="Raised"
            value={`${pool.raised.toLocaleString()} XLM`}
          />
          <Metric
            label="Target"
            value={`${pool.target.toLocaleString()} XLM`}
          />
          <Metric
            label="Available to Withdraw"
            value={`${availableBalance.toLocaleString()} XLM`}
          />
        </div>

        <div className="mt-6 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-4">
          <p className="text-sm text-[var(--color-text-muted)]">
            Withdrawals are only enabled once the pool is completed. Confirm the
            transaction in your wallet to proceed.
          </p>

          <div className="mt-4 flex flex-col gap-3 sm:flex-row sm:items-center">
            <button
              onClick={() => setIsConfirmOpen(true)}
              disabled={!canWithdraw || status === 'submitting'}
              className="rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white hover:bg-brand-700 transition-colors disabled:cursor-not-allowed disabled:opacity-50"
            >
              {status === 'submitting'
                ? 'Processing withdrawal...'
                : 'Withdraw'}
            </button>
            <p className="text-xs text-[var(--color-text-muted)]">
              Connected wallet:{' '}
              {publicKey ? `${publicKey.slice(0, 8)}...` : 'None'}
            </p>
          </div>
        </div>

        {status === 'success' && receiptHash && (
          <div className="mt-4 rounded-xl border border-success/30 bg-success-light p-4">
            <p className="text-sm font-medium text-success-dark">
              Withdrawal confirmed.
            </p>
            <p className="mt-1 break-all text-xs text-success-dark">
              Transaction hash: {receiptHash}
            </p>
          </div>
        )}

        {status === 'error' && errorMessage && (
          <div className="mt-4 rounded-xl border border-error/30 bg-error-light p-4">
            <p className="text-sm font-medium text-error-dark">
              {errorMessage}
            </p>
          </div>
        )}
      </section>

      {isConfirmOpen && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="withdraw-title"
          className="fixed inset-0 z-50 flex items-center justify-center p-4"
        >
          <div
            className="absolute inset-0 bg-black/40"
            onClick={() => setIsConfirmOpen(false)}
            aria-hidden="true"
          />
          <div className="relative w-full max-w-md rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xl">
            <h2 id="withdraw-title" className="text-base font-semibold">
              Confirm withdrawal
            </h2>
            <p className="mt-2 text-sm text-[var(--color-text-muted)]">
              Withdraw {availableBalance.toLocaleString()} XLM from &quot;
              {pool.title}&quot;? This creates an on-chain transaction.
            </p>
            <div className="mt-5 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
              <button
                onClick={() => setIsConfirmOpen(false)}
                className="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm hover:bg-[var(--color-surface-raised)] transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={processWithdrawal}
                disabled={status === 'submitting'}
                className="rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white hover:bg-brand-700 transition-colors disabled:opacity-50"
              >
                {status === 'submitting'
                  ? 'Submitting...'
                  : 'Confirm withdrawal'}
              </button>
            </div>
          </div>
        </div>
  const pct = Math.min(100, Math.round((pool.raised / pool.target) * 100));
  const isOwner = publicKey !== null && publicKey === pool.creator;
  const isCompleted = pool.status === 'Completed';
  const lastUpdated = MOCK_LAST_UPDATED[pool.id] ?? pool.createdAt;

  return (
    <main className="mx-auto max-w-5xl px-6 py-10">
      {/* ── Breadcrumb ──────────────────────────────────────────────────────── */}
      <nav aria-label="Breadcrumb" className="mb-6 flex items-center gap-2 text-sm text-[var(--color-text-muted)]">
        <Link href="/pools" className="hover:text-brand-600 transition-colors">
          Pools
        </Link>
        <ChevronRightIcon />
        <span className="text-[var(--color-text)] font-medium" aria-current="page">
          {pool.title}
        </span>
      </nav>

      <div className="grid gap-8 lg:grid-cols-[1fr_320px]">
        {/* ── Left column ───────────────────────────────────────────────────── */}
        <div className="flex flex-col gap-8">
          {/* Pool header */}
          <section aria-labelledby="pool-title">
            <div
              className="mb-6 flex h-40 w-full items-center justify-center rounded-2xl sm:h-56"
              style={{ backgroundColor: pool.imageColor }}
              aria-hidden="true"
            >
              <PoolIcon className="size-16 text-white/60" />
            </div>

            <div className="flex flex-wrap items-start gap-3">
              <h1 id="pool-title" className="flex-1 text-2xl font-bold tracking-tight sm:text-3xl">
                {pool.title}
              </h1>
              <StatusBadge status={pool.status} />
            </div>

            <div className="mt-2 flex flex-wrap gap-3 text-sm text-[var(--color-text-muted)]">
              <span className="inline-flex items-center gap-1">
                <TagIcon />
                {pool.category}
              </span>
              {pool.createdAt && (
                <span className="inline-flex items-center gap-1">
                  <CalendarIcon />
                  Created {pool.createdAt}
                </span>
              )}
              {lastUpdated && (
                <span className="inline-flex items-center gap-1">
                  <ClockIcon />
                  Updated {lastUpdated}
                </span>
              )}
            </div>

            <p className="mt-4 leading-relaxed text-[var(--color-text-muted)]">
              {pool.description}
            </p>
          </section>

          {/* ── Creator info ──────────────────────────────────────────────── */}
          {pool.creator && (
            <section
              aria-labelledby="creator-heading"
              className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-5"
            >
              <h2 id="creator-heading" className="mb-3 text-sm font-semibold">
                Pool Creator
              </h2>
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <WalletAddress address={pool.creator} />
                {isOwner && (
                  <div className="flex flex-wrap gap-2">
                    <Link
                      href={`/pools/${pool.id}/edit`}
                      className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-xs font-medium hover:bg-[var(--color-border)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
                      aria-label={`Edit pool: ${pool.title}`}
                    >
                      Edit Pool
                    </Link>
                    <button
                      type="button"
                      disabled={!isCompleted}
                      className="rounded-lg bg-brand-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-brand-700 transition-colors disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
                      aria-label={
                        isCompleted
                          ? 'Withdraw funds from this pool'
                          : 'Withdraw unavailable — pool is not completed'
                      }
                    >
                      Withdraw Funds
                    </button>
                  </div>
                )}
              </div>
            </section>
          )}

          {/* ── Contributors ──────────────────────────────────────────────── */}
          <section aria-labelledby="contributors-heading">
            <h2 id="contributors-heading" className="mb-4 text-lg font-semibold">
              Contributors
              <span className="ml-2 text-sm font-normal text-[var(--color-text-muted)]">
                ({contributors.length})
              </span>
            </h2>

            {contributors.length === 0 ? (
              <p className="text-sm text-[var(--color-text-muted)]">
                No contributions yet. Be the first to donate!
              </p>
            ) : (
              <ul className="flex flex-col gap-2" role="list">
                {contributors.map((c, i) => (
                  <li
                    key={i}
                    className="flex flex-col gap-1 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3 sm:flex-row sm:items-center sm:justify-between"
                  >
                    <span
                      className="max-w-xs truncate font-mono text-xs text-[var(--color-text-muted)]"
                      title={c.address}
                      aria-label={`Contributor address: ${c.address}`}
                    >
                      {c.address}
                    </span>
                    <div className="flex items-center gap-4 text-sm">
                      <span className="font-semibold text-brand-600">
                        {c.amount.toLocaleString()} XLM
                      </span>
                      <span className="text-[var(--color-text-muted)]">{c.donatedAt}</span>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </section>

          {/* ── Timeline ──────────────────────────────────────────────────── */}
          {timeline.length > 0 && (
            <section aria-labelledby="timeline-heading">
              <h2 id="timeline-heading" className="mb-4 text-lg font-semibold">
                History
              </h2>
              <ol className="relative border-l border-[var(--color-border)] pl-6" role="list">
                {timeline.map((event) => (
                  <li key={event.id} className="mb-6 last:mb-0">
                    <div
                      className="absolute -left-1.5 size-3 rounded-full border-2 border-[var(--color-surface)] bg-brand-500"
                      aria-hidden="true"
                    />
                    <time
                      dateTime={event.date}
                      className="mb-1 block text-xs text-[var(--color-text-muted)]"
                    >
                      {event.date}
                    </time>
                    <p className="text-sm font-medium">
                      {event.label}
                      {event.amount !== undefined && (
                        <span className="ml-2 font-normal text-brand-600">
                          +{event.amount.toLocaleString()} XLM
                        </span>
                      )}
                    </p>
                  </li>
                ))}
              </ol>
            </section>
          )}
        </div>

        {/* ── Right sidebar ─────────────────────────────────────────────────── */}
        <aside className="flex flex-col gap-6" aria-label="Pool funding details">
          {/* Funding card */}
          <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-6">
            <p className="text-3xl font-bold text-brand-600">
              {pool.raised.toLocaleString()}{' '}
              <span className="text-lg font-semibold">XLM</span>
            </p>
            <p className="mt-1 text-sm text-[var(--color-text-muted)]">
              raised of {pool.target.toLocaleString()} XLM goal
            </p>

            {/* Progress bar */}
            <div className="mt-4">
              <div
                className="h-3 w-full overflow-hidden rounded-full bg-[var(--color-border)]"
                role="progressbar"
                aria-valuenow={pct}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-label={`Funding progress: ${pct}%`}
              >
                <div
                  className="h-full rounded-full bg-brand-500 transition-all"
                  style={{ width: `${pct}%` }}
                />
              </div>
              <p className="mt-1 text-right text-xs font-semibold text-brand-600">
                {pct}%
              </p>
            </div>

            {/* Stats row */}
            <div className="mt-5 grid grid-cols-2 gap-4 border-t border-[var(--color-border)] pt-5">
              <div>
                <p className="text-lg font-bold">{contributors.length}</p>
                <p className="text-xs text-[var(--color-text-muted)]">Contributors</p>
              </div>
              <div>
                <p className="text-lg font-bold">{isCompleted ? '✓ Done' : 'Active'}</p>
                <p className="text-xs text-[var(--color-text-muted)]">Status</p>
              </div>
            </div>

            {/* Donate button */}
            <button
              type="button"
              disabled={isCompleted}
              className="mt-6 w-full rounded-full bg-brand-600 px-6 py-3 text-sm font-semibold text-white hover:bg-brand-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
              aria-label={
                isCompleted
                  ? 'This pool is no longer accepting donations'
                  : `Donate to ${pool.title}`
              }
            >
              {isCompleted ? 'Pool Closed' : 'Donate Now'}
            </button>

            {!isCompleted && (
              <p className="mt-3 text-center text-xs text-[var(--color-text-muted)]">
                Transactions settled on Stellar · Near-zero fees
              </p>
            )}
          </div>

          {/* Pool metadata card */}
          <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-5">
            <h2 className="mb-3 text-sm font-semibold">Pool Details</h2>
            <dl className="flex flex-col gap-2 text-sm">
              <div className="flex justify-between">
                <dt className="text-[var(--color-text-muted)]">Category</dt>
                <dd className="font-medium">{pool.category}</dd>
              </div>
              {pool.createdAt && (
                <div className="flex justify-between">
                  <dt className="text-[var(--color-text-muted)]">Created</dt>
                  <dd className="font-medium">{pool.createdAt}</dd>
                </div>
              )}
              {lastUpdated && (
                <div className="flex justify-between">
                  <dt className="text-[var(--color-text-muted)]">Last update</dt>
                  <dd className="font-medium">{lastUpdated}</dd>
                </div>
              )}
              <div className="flex justify-between">
                <dt className="text-[var(--color-text-muted)]">Pool ID</dt>
                <dd className="font-mono text-xs font-medium">{pool.id}</dd>
              </div>
            </dl>
          </div>
        </aside>
      </div>
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

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl border border-[var(--color-border)] p-4">
      <p className="text-xs text-[var(--color-text-muted)]">{label}</p>
      <p className="mt-1 text-lg font-semibold text-brand-600">{value}</p>
    </div>
/* ── StatusBadge ──────────────────────────────────────────────────────────── */

function StatusBadge({ status }: { status: Pool['status'] }) {
  const styles =
    status === 'Active'
      ? 'bg-success-light text-success-dark'
      : 'bg-[var(--color-surface-raised)] text-[var(--color-text-muted)] border border-[var(--color-border)]';

  return (
    <span
      className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${styles}`}
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
      className="mx-auto max-w-5xl px-6 py-10"
      aria-busy="true"
      aria-label="Loading pool details"
    >
      <div className="mb-6 h-4 w-32 animate-pulse rounded-full bg-[var(--color-border)]" />
      <div className="grid gap-8 lg:grid-cols-[1fr_320px]">
        <div className="flex flex-col gap-6">
          <div className="h-56 w-full animate-pulse rounded-2xl bg-[var(--color-border)]" />
          <div className="h-8 w-2/3 animate-pulse rounded-full bg-[var(--color-border)]" />
          <div className="h-20 w-full animate-pulse rounded-xl bg-[var(--color-border)]" />
          <div className="h-32 w-full animate-pulse rounded-xl bg-[var(--color-border)]" />
        </div>
        <div className="flex flex-col gap-4">
          <div className="h-64 w-full animate-pulse rounded-2xl bg-[var(--color-border)]" />
          <div className="h-32 w-full animate-pulse rounded-2xl bg-[var(--color-border)]" />
        </div>
      </div>
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

function PoolIcon({ className = 'size-7' }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className={className}
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

function ChevronRightIcon() {
function ChevronLeftIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={2}
      stroke="currentColor"
      className="size-3.5"
      aria-hidden="true"
    >
      <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
    </svg>
  );
}

function TagIcon() {
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
        d="M9.568 3H5.25A2.25 2.25 0 0 0 3 5.25v4.318c0 .597.237 1.17.659 1.591l9.581 9.581c.699.699 1.78.872 2.607.33a18.095 18.095 0 0 0 5.223-5.223c.542-.827.369-1.908-.33-2.607L11.16 3.66A2.25 2.25 0 0 0 9.568 3Z"
      />
      <path strokeLinecap="round" strokeLinejoin="round" d="M6 6h.008v.008H6V6Z" />
    </svg>
  );
}

function CalendarIcon() {
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
        d="M6.75 3v2.25M17.25 3v2.25M3 18.75V7.5a2.25 2.25 0 0 1 2.25-2.25h13.5A2.25 2.25 0 0 1 21 7.5v11.25m-18 0A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75m-18 0v-7.5A2.25 2.25 0 0 1 5.25 9h13.5A2.25 2.25 0 0 1 21 11.25v7.5"
      />
    </svg>
  );
}

function ClockIcon() {
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
        d="M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
        d="M15.75 19.5 8.25 12l7.5-7.5"
      />
    </svg>
  );
}
