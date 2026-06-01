'use client';

import Link from 'next/link';
import { useParams } from 'next/navigation';
import { useEffect, useState } from 'react';
import { DonateModal } from '@/components/DonateModal';
import { EmptyState } from '@/components/EmptyState';
import { SocialShareButtons } from '@/components/SocialShareButtons';
import { WalletAddress } from '@/components/WalletAddress';
import type { Pool } from '@/src/store/poolsStore';
import { useWalletStore } from '@/src/store/walletStore';

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
    status: 'Completed',
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

interface Contributor {
  address: string;
  amount: number;
  donatedAt: string;
}

const MOCK_CONTRIBUTORS: Record<string, Contributor[]> = {
  '1': [
    {
      address: 'GXYZ1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890AB',
      amount: 500,
      donatedAt: '2025-03-05',
    },
    {
      address: 'GABC9876543210ZYXWV9876543210ZYXWV9876543210ZYXWV9876543210ZY',
      amount: 1200,
      donatedAt: '2025-03-12',
    },
  ],
  '2': [
    {
      address: 'GXYZ1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890AB',
      amount: 750,
      donatedAt: '2025-01-20',
    },
  ],
  '3': [
    {
      address: 'GDEF5555555555GHIJK5555555555GHIJK5555555555GHIJK5555555555GH',
      amount: 1000,
      donatedAt: '2024-11-15',
    },
  ],
};

interface TimelineEvent {
  id: string;
  label: string;
  date: string;
  amount?: number;
}

const MOCK_TIMELINE: Record<string, TimelineEvent[]> = {
  '1': [
    { id: 'e1', label: 'Pool created', date: '2025-03-01' },
    {
      id: 'e2',
      label: 'First donation received',
      date: '2025-03-05',
      amount: 500,
    },
  ],
  '2': [
    { id: 'e1', label: 'Pool created', date: '2025-01-15' },
    { id: 'e2', label: 'Goal reached', date: '2025-02-01' },
  ],
  '3': [
    { id: 'e1', label: 'Pool created', date: '2024-11-10' },
    { id: 'e2', label: 'Pool completed', date: '2024-12-31' },
  ],
};

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
  const [donateOpen, setDonateOpen] = useState(false);

  useEffect(() => {
    initialize();
  }, [initialize]);

  useEffect(() => {
    if (!id) return;
    const timer = setTimeout(() => {
      const found = MOCK_POOLS.find((p) => p.id === id) ?? null;
      if (!found) {
        setNotFound(true);
        setPool(null);
      } else {
        setPool(found);
        setContributors(MOCK_CONTRIBUTORS[id] ?? []);
        setTimeline(MOCK_TIMELINE[id] ?? []);
      }
      setLoading(false);
    }, 300);
    return () => clearTimeout(timer);
  }, [id]);

  if (loading) {
    return <PoolDetailSkeleton />;
  }

  if (notFound || !pool) {
    return (
      <main className="mx-auto max-w-3xl px-4 py-10 sm:px-6 sm:py-16">
        <EmptyState
          icon="not-found"
          iconTone="muted"
          title="Pool not found"
          description="This pool does not exist or has been removed."
          action={{ label: 'Browse Pools', href: '/pools' }}
        />
      </main>
    );
  }

  const pct = Math.min(100, Math.round((pool.raised / pool.target) * 100));
  const isOwner = publicKey !== null && publicKey === pool.creator;
  const isCompleted = pool.status === 'Completed';
  const isActive = pool.status === 'Active';
  const lastUpdated = MOCK_LAST_UPDATED[pool.id] ?? pool.createdAt;

  return (
    <main className="mx-auto max-w-5xl px-4 py-8 sm:px-6 sm:py-10">
      <nav
        aria-label="Breadcrumb"
        className="mb-6 flex items-center gap-2 text-sm text-[var(--color-text-muted)]"
      >
        <Link href="/pools" className="hover:text-brand-600 transition-colors">
          Pools
        </Link>
        <ChevronRightIcon />
        <span
          className="font-medium text-[var(--color-text)]"
          aria-current="page"
        >
          {pool.title}
        </span>
      </nav>

      <div className="grid gap-8 lg:grid-cols-[1fr_320px]">
        <div className="flex flex-col gap-8">
          <section aria-labelledby="pool-title">
            <div
              className="mb-6 flex h-40 w-full items-center justify-center rounded-2xl sm:h-56"
              style={{ backgroundColor: pool.imageColor }}
              aria-hidden="true"
            >
              <PoolIcon className="size-16 text-white/60" />
            </div>

            <div className="flex flex-wrap items-start gap-3">
              <h1
                id="pool-title"
                className="flex-1 text-2xl font-bold tracking-tight sm:text-3xl"
              >
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
                  <Link
                    href={`/pools/${pool.id}/edit`}
                    className="w-fit rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-xs font-medium hover:bg-[var(--color-border)] transition-colors"
                  >
                    Edit Pool
                  </Link>
                )}
              </div>
            </section>
          )}

          <section aria-labelledby="contributors-heading">
            <h2
              id="contributors-heading"
              className="mb-4 text-lg font-semibold"
            >
              Contributors
              <span className="ml-2 text-sm font-normal text-[var(--color-text-muted)]">
                ({contributors.length})
              </span>
            </h2>

            {contributors.length === 0 ? (
              <>
                <div className="rounded-xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface-raised)] px-4 py-8 text-center">
                  <p className="font-semibold">No contributions yet</p>
                  <p className="mt-1 text-sm text-[var(--color-text-muted)]">
                    Be the first to support this pool.
                  </p>
                  {isActive && (
                    <button
                      type="button"
                      onClick={() => setDonateOpen(true)}
                      className="mt-4 rounded-full bg-brand-600 px-5 py-2 text-sm font-semibold text-white hover:bg-brand-700 transition-colors"
                    >
                      Donate Now
                    </button>
                  )}
                </div>
                <EmptyState
                  variant="compact"
                  icon="contributors"
                  iconTone="muted"
                  title="No contributions yet"
                  description="Be the first to support this pool."
                  action={
                    isActive
                      ? {
                          label: 'Donate Now',
                          onClick: () => setDonateOpen(true),
                        }
                      : undefined
                  }
                  steps={[
                    { text: 'Connect your Stellar wallet' },
                    { text: 'Choose an amount to donate' },
                    { text: 'Confirm the transaction in Freighter' },
                  ]}
                />
              </>
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
                    >
                      {c.address}
                    </span>
                    <div className="flex items-center gap-4 text-sm">
                      <span className="font-semibold text-brand-600">
                        {c.amount.toLocaleString()} XLM
                      </span>
                      <span className="text-[var(--color-text-muted)]">
                        {c.donatedAt}
                      </span>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </section>

          <section aria-labelledby="timeline-heading">
            <h2 id="timeline-heading" className="mb-4 text-lg font-semibold">
              History
            </h2>

            {timeline.length === 0 ? (
              <>
                <p className="text-sm text-[var(--color-text-muted)]">
                  Pool milestones and donations will appear here as they happen.
                </p>
                <EmptyState
                  variant="compact"
                  icon="history"
                  iconTone="muted"
                  title="No activity yet"
                  description="Pool milestones and donations will appear here as they happen."
                />
              </>
            ) : (
              <ol
                className="relative border-l border-[var(--color-border)] pl-6"
                role="list"
              >
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
            )}
          </section>
        </div>

        <aside
          className="flex flex-col gap-6"
          aria-label="Pool funding details"
        >
          <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-6">
            <p className="text-3xl font-bold text-brand-600">
              {pool.raised.toLocaleString()}{' '}
              <span className="text-lg font-semibold">XLM</span>
            </p>
            <p className="mt-1 text-sm text-[var(--color-text-muted)]">
              raised of {pool.target.toLocaleString()} XLM goal
            </p>

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

            <div className="mt-5 grid grid-cols-2 gap-4 border-t border-[var(--color-border)] pt-5">
              <div>
                <p className="text-lg font-bold">{contributors.length}</p>
                <p className="text-xs text-[var(--color-text-muted)]">
                  Contributors
                </p>
              </div>
              <div>
                <p className="text-lg font-bold">
                  {isCompleted ? 'Done' : 'Active'}
                </p>
                <p className="text-xs text-[var(--color-text-muted)]">Status</p>
              </div>
            </div>

            <button
              type="button"
              onClick={() => setDonateOpen(true)}
              disabled={!isActive}
              className="mt-6 w-full rounded-full bg-brand-600 px-6 py-3 text-sm font-semibold text-white hover:bg-brand-700 transition-colors disabled:cursor-not-allowed disabled:opacity-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
            >
              {isCompleted ? 'Pool Closed' : 'Donate Now'}
            </button>

            {!publicKey && isActive && (
              <p className="mt-3 text-center text-xs text-[var(--color-text-muted)]">
                Connect your wallet to donate to this pool.
              </p>
            )}
          </div>

          <SocialShareButtons
            poolId={pool.id}
            poolTitle={pool.title}
            customMessage={`Check out this amazing pool: "${pool.title}" - Support the cause and make a difference! 🌟`}
          />
        </aside>
      </div>

      {donateOpen && (
        <DonateModal pool={pool} onClose={() => setDonateOpen(false)} />
      )}
    </main>
  );
}

function StatusBadge({ status }: { status: Pool['status'] }) {
  const styles =
    status === 'Active'
      ? 'bg-success-light text-success-dark'
      : 'bg-[var(--color-surface-raised)] text-[var(--color-text-muted)] border border-[var(--color-border)]';

  return (
    <span
      className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${styles}`}
      aria-label={`Pool status: ${status}`}
    >
      {status}
    </span>
  );
}

function PoolDetailSkeleton() {
  return (
    <main
      className="mx-auto max-w-5xl px-4 py-8 sm:px-6 sm:py-10"
      aria-busy="true"
      aria-label="Loading pool details"
    >
      <div className="mb-6 h-4 w-32 animate-pulse rounded-full bg-[var(--color-border)]" />
      <div className="grid gap-8 lg:grid-cols-[1fr_320px]">
        <div className="flex flex-col gap-6">
          <div className="h-56 w-full animate-pulse rounded-2xl bg-[var(--color-border)]" />
          <div className="h-8 w-2/3 animate-pulse rounded-full bg-[var(--color-border)]" />
          <div className="h-20 w-full animate-pulse rounded-xl bg-[var(--color-border)]" />
        </div>
        <div className="h-64 w-full animate-pulse rounded-2xl bg-[var(--color-border)]" />
      </div>
    </main>
  );
}

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
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M6 6h.008v.008H6V6Z"
      />
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
      />
    </svg>
  );
}
