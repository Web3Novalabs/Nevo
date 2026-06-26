'use client';

import React, { useEffect, useId, useState } from 'react';
import Link from 'next/link';
import { useParams, useRouter } from 'next/navigation';
import { DonateModal } from '@/components/DonateModal';
import { EmptyState } from '@/components/EmptyState';
import { WalletAddress } from '@/components/WalletAddress';
import { CopyButton } from '@/components/CopyButton';
import { usePoolsStore } from '@/src/store/poolsStore';
import type { Pool } from '@/src/store/poolsStore';
import { useWalletStore } from '@/src/store/walletStore';

// Removed MOCK_POOLS

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

// ── Comments ────────────────────────────────────────────────────────────────

interface Comment {
  id: string;
  poolId: string;
  authorAddress: string;
  text: string;
  createdAt: string;
  updatedAt?: string;
  parentId: string | null;
  replies: Comment[];
}

// TODO: Replace with real API call to GET /pools/:id/comments
const MOCK_COMMENTS: Record<string, Comment[]> = {
  '1': [
    {
      id: 'c1',
      poolId: '1',
      authorAddress: 'GXYZ1234567890ABCDE1234567890ABCDE1234567890ABCDE1234567890AB',
      text: 'Amazing initiative! How are the funds being allocated?',
      createdAt: '2025-03-06T10:00:00Z',
      parentId: null,
      replies: [
        {
          id: 'c1r1',
          poolId: '1',
          authorAddress: 'GABC9876543210ZYXWV9876543210ZYXWV9876543210ZYXWV9876543210ZY',
          text: 'Funds go directly to vetted local partners. You can track every withdrawal on-chain!',
          createdAt: '2025-03-06T11:30:00Z',
          parentId: 'c1',
          replies: [],
        },
      ],
    },
    {
      id: 'c2',
      poolId: '1',
      authorAddress: 'GABC9876543210ZYXWV9876543210ZYXWV9876543210ZYXWV9876543210ZY',
      text: 'Love that everything is transparent on-chain. Keep up the great work!',
      createdAt: '2025-03-10T14:20:00Z',
      parentId: null,
      replies: [],
    },
  ],
};

const MOCK_LAST_UPDATED: Record<string, string> = {
  '1': '2025-04-15',
  '2': '2025-02-01',
  '3': '2024-12-31',
};

export default function PoolDetailPage() {
  const { id } = useParams<{ id: string }>();
  const router = useRouter();
  const { publicKey, initialize } = useWalletStore();
  const {
    currentPool: pool,
    poolLoading: loading,
    fetchPool,
  } = usePoolsStore();
  const [contributors, setContributors] = useState<Contributor[]>([]);
  const [comments, setComments] = useState<Comment[]>([]);
  const [donateOpen, setDonateOpen] = useState(false);

  useEffect(() => {
    initialize();
  }, [initialize]);

  useEffect(() => {
    if (!id) return;
    const loadPool = async () => {
      const p = await fetchPool(Number(id));
      if (!p) {
        router.replace('/pools');
      } else {
        setContributors(MOCK_CONTRIBUTORS[id] ?? []);
        // TODO: replace with real API call: apiClient.get(`/pools/${id}/comments`)
        setComments(MOCK_COMMENTS[id] ?? []);
      }
    };
    loadPool();
  }, [id, fetchPool, router]);

  if (loading) {
    return <PoolDetailSkeleton />;
  }

  if (!pool) {
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

  const timeline: TimelineEvent[] = [];
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

          <section aria-labelledby="comments-heading">
            <h2 id="comments-heading" className="mb-4 text-lg font-semibold">
              Discussion
              <span className="ml-2 text-sm font-normal text-[var(--color-text-muted)]">
                ({comments.reduce((sum, c) => sum + 1 + c.replies.length, 0)})
              </span>
            </h2>
            <CommentsSection
              poolId={id}
              comments={comments}
              setComments={setComments}
              currentUserAddress={publicKey}
            />
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

            <div className="mt-4">
              <CopyButton
                text={
                  typeof window !== 'undefined'
                    ? window.location.href
                    : `/pools/${pool.id}`
                }
                label="Copy Pool Link"
                copiedLabel="Link Copied!"
                className="w-full justify-center"
              />
            </div>

            {!publicKey && isActive && (
              <p className="mt-3 text-center text-xs text-[var(--color-text-muted)]">
                Connect your wallet to donate to this pool.
              </p>
            )}
          </div>
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

// ── CommentsSection ──────────────────────────────────────────────────────────

interface CommentsSectionProps {
  poolId: string;
  comments: Comment[];
  setComments: React.Dispatch<React.SetStateAction<Comment[]>>;
  currentUserAddress: string | null;
}

function CommentsSection({
  poolId,
  comments,
  setComments,
  currentUserAddress,
}: CommentsSectionProps) {
  function handleAddComment(text: string) {
    if (!currentUserAddress) return;
    const newComment: Comment = {
      id: `c-${Date.now()}`,
      poolId,
      authorAddress: currentUserAddress,
      text,
      createdAt: new Date().toISOString(),
      parentId: null,
      replies: [],
    };
    // TODO: replace with real API call: apiClient.post(`/pools/${poolId}/comments`, { text })
    setComments((prev) => [newComment, ...prev]);
  }

  function handleAddReply(parentId: string, text: string) {
    if (!currentUserAddress) return;
    const newReply: Comment = {
      id: `c-${Date.now()}`,
      poolId,
      authorAddress: currentUserAddress,
      text,
      createdAt: new Date().toISOString(),
      parentId,
      replies: [],
    };
    // TODO: replace with real API call: apiClient.post(`/pools/${poolId}/comments`, { text, parentId })
    setComments((prev) =>
      prev.map((c) =>
        c.id === parentId ? { ...c, replies: [...c.replies, newReply] } : c
      )
    );
  }

  function handleEditComment(id: string, text: string, isReply: boolean, parentId?: string) {
    // TODO: replace with real API call: apiClient.put(`/pools/${poolId}/comments/${id}`, { text })
    if (isReply && parentId) {
      setComments((prev) =>
        prev.map((c) =>
          c.id === parentId
            ? {
                ...c,
                replies: c.replies.map((r) =>
                  r.id === id ? { ...r, text, updatedAt: new Date().toISOString() } : r
                ),
              }
            : c
        )
      );
    } else {
      setComments((prev) =>
        prev.map((c) =>
          c.id === id ? { ...c, text, updatedAt: new Date().toISOString() } : c
        )
      );
    }
  }

  function handleDeleteComment(id: string, isReply: boolean, parentId?: string) {
    // TODO: replace with real API call: apiClient.delete(`/pools/${poolId}/comments/${id}`)
    if (isReply && parentId) {
      setComments((prev) =>
        prev.map((c) =>
          c.id === parentId
            ? { ...c, replies: c.replies.filter((r) => r.id !== id) }
            : c
        )
      );
    } else {
      setComments((prev) => prev.filter((c) => c.id !== id));
    }
  }

  return (
    <div className="flex flex-col gap-5">
      <CommentForm
        onSubmit={handleAddComment}
        currentUserAddress={currentUserAddress}
        placeholder="Share a thought or ask a question…"
      />

      {comments.length === 0 ? (
        <div className="rounded-xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface-raised)] px-4 py-8 text-center">
          <p className="font-semibold">No comments yet</p>
          <p className="mt-1 text-sm text-[var(--color-text-muted)]">
            Be the first to start the discussion.
          </p>
        </div>
      ) : (
        <ul className="flex flex-col gap-4" role="list">
          {comments.map((comment) => (
            <CommentItem
              key={comment.id}
              comment={comment}
              currentUserAddress={currentUserAddress}
              onReply={(text) => handleAddReply(comment.id, text)}
              onEdit={(text) => handleEditComment(comment.id, text, false)}
              onDelete={() => handleDeleteComment(comment.id, false)}
              onEditReply={(replyId, text) =>
                handleEditComment(replyId, text, true, comment.id)
              }
              onDeleteReply={(replyId) =>
                handleDeleteComment(replyId, true, comment.id)
              }
            />
          ))}
        </ul>
      )}
    </div>
  );
}

// ── CommentForm ──────────────────────────────────────────────────────────────

interface CommentFormProps {
  onSubmit: (text: string) => void;
  currentUserAddress: string | null;
  placeholder?: string;
  initialValue?: string;
  onCancel?: () => void;
  submitLabel?: string;
}

function CommentForm({
  onSubmit,
  currentUserAddress,
  placeholder = 'Write a comment…',
  initialValue = '',
  onCancel,
  submitLabel = 'Post',
}: CommentFormProps) {
  const [text, setText] = useState(initialValue);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const trimmed = text.trim();
    if (!trimmed) return;
    onSubmit(trimmed);
    setText('');
  }

  if (!currentUserAddress) {
    return (
      <p className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] px-4 py-3 text-sm text-[var(--color-text-muted)]">
        Connect your wallet to join the discussion.
      </p>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-2">
      <label className="sr-only" htmlFor="comment-input">
        {placeholder}
      </label>
      <textarea
        id="comment-input"
        value={text}
        onChange={(e) => setText(e.target.value)}
        placeholder={placeholder}
        rows={3}
        className="w-full resize-none rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3 text-sm placeholder:text-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-brand-500"
        aria-label={placeholder}
      />
      <div className="flex items-center justify-end gap-2">
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-xs font-medium hover:bg-[var(--color-surface-raised)] transition-colors"
          >
            Cancel
          </button>
        )}
        <button
          type="submit"
          disabled={!text.trim()}
          className="rounded-full bg-brand-600 px-4 py-1.5 text-xs font-semibold text-white hover:bg-brand-700 transition-colors disabled:cursor-not-allowed disabled:opacity-50"
        >
          {submitLabel}
        </button>
      </div>
    </form>
  );
}

// ── CommentItem ───────────────────────────────────────────────────────────────

interface CommentItemProps {
  comment: Comment;
  currentUserAddress: string | null;
  onReply: (text: string) => void;
  onEdit: (text: string) => void;
  onDelete: () => void;
  onEditReply: (replyId: string, text: string) => void;
  onDeleteReply: (replyId: string) => void;
  isReply?: boolean;
}

function formatCommentDate(iso: string) {
  return new Date(iso).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function CommentItem({
  comment,
  currentUserAddress,
  onReply,
  onEdit,
  onDelete,
  onEditReply,
  onDeleteReply,
  isReply = false,
}: CommentItemProps) {
  const [showReplyForm, setShowReplyForm] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const isOwn = currentUserAddress !== null && currentUserAddress === comment.authorAddress;
  const shortAddress = `${comment.authorAddress.slice(0, 6)}…${comment.authorAddress.slice(-4)}`;

  return (
    <li
      className={`rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 ${isReply ? 'ml-6 border-l-2 border-l-brand-200' : ''}`}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex flex-col gap-0.5">
          <span
            className="font-mono text-xs font-medium text-brand-600"
            title={comment.authorAddress}
          >
            {shortAddress}
          </span>
          <time
            dateTime={comment.createdAt}
            className="text-xs text-[var(--color-text-muted)]"
          >
            {formatCommentDate(comment.createdAt)}
            {comment.updatedAt && (
              <span className="ml-1 italic">(edited)</span>
            )}
          </time>
        </div>
        {isOwn && (
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setIsEditing(true)}
              className="text-xs text-[var(--color-text-muted)] hover:text-brand-600 transition-colors"
              aria-label="Edit comment"
            >
              Edit
            </button>
            <button
              type="button"
              onClick={onDelete}
              className="text-xs text-[var(--color-text-muted)] hover:text-red-600 transition-colors"
              aria-label="Delete comment"
            >
              Delete
            </button>
          </div>
        )}
      </div>

      {isEditing ? (
        <div className="mt-2">
          <CommentForm
            onSubmit={(text) => {
              onEdit(text);
              setIsEditing(false);
            }}
            currentUserAddress={currentUserAddress}
            placeholder="Edit your comment…"
            initialValue={comment.text}
            onCancel={() => setIsEditing(false)}
            submitLabel="Save"
          />
        </div>
      ) : (
        <p className="mt-2 text-sm text-[var(--color-text)]">{comment.text}</p>
      )}

      {!isReply && (
        <div className="mt-3">
          <button
            type="button"
            onClick={() => setShowReplyForm((v) => !v)}
            className="text-xs text-[var(--color-text-muted)] hover:text-brand-600 transition-colors"
          >
            {showReplyForm ? 'Cancel reply' : `Reply${comment.replies.length > 0 ? ` (${comment.replies.length})` : ''}`}
          </button>

          {showReplyForm && (
            <div className="mt-3">
              <CommentForm
                onSubmit={(text) => {
                  onReply(text);
                  setShowReplyForm(false);
                }}
                currentUserAddress={currentUserAddress}
                placeholder="Write a reply…"
                onCancel={() => setShowReplyForm(false)}
                submitLabel="Reply"
              />
            </div>
          )}

          {comment.replies.length > 0 && (
            <ul className="mt-3 flex flex-col gap-3" role="list">
              {comment.replies.map((reply) => (
                <CommentItem
                  key={reply.id}
                  comment={reply}
                  currentUserAddress={currentUserAddress}
                  onReply={() => {}}
                  onEdit={(text) => onEditReply(reply.id, text)}
                  onDelete={() => onDeleteReply(reply.id)}
                  onEditReply={() => {}}
                  onDeleteReply={() => {}}
                  isReply
                />
              ))}
            </ul>
          )}
        </div>
      )}
    </li>
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
