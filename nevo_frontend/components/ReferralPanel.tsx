'use client';

import React, { FC, useCallback, useEffect, useRef, useState } from 'react';

interface ReferralStats {
  clicks: number;
  signups: number;
}

interface ReferralPanelProps {
  poolId: string;
}

function generateReferralId(poolId: string): string {
  const key = `nevo_ref_${poolId}`;
  let id = sessionStorage.getItem(key);
  if (!id) {
    id = `${poolId}-${Math.random().toString(36).slice(2, 9)}`;
    sessionStorage.setItem(key, id);
  }
  return id;
}

function getStats(referralId: string): ReferralStats {
  try {
    const raw = localStorage.getItem(`nevo_ref_stats_${referralId}`);
    return raw ? (JSON.parse(raw) as ReferralStats) : { clicks: 0, signups: 0 };
  } catch {
    return { clicks: 0, signups: 0 };
  }
}

function recordClick(referralId: string) {
  const stats = getStats(referralId);
  localStorage.setItem(
    `nevo_ref_stats_${referralId}`,
    JSON.stringify({ ...stats, clicks: stats.clicks + 1 })
  );
}

// Lazy initializer — runs only on client (this component is 'use client')
function initReferral(poolId: string) {
  const id = generateReferralId(poolId);
  return {
    referralId: id,
    referralUrl: `${window.location.origin}/pools/${poolId}?ref=${id}`,
    stats: getStats(id),
  };
}

export const ReferralPanel: FC<ReferralPanelProps> = ({ poolId }) => {
  const [referral] = useState(() => initReferral(poolId));
  const [stats, setStats] = useState<ReferralStats>(() =>
    getStats(referral.referralId)
  );
  const [copied, setCopied] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Record a click when this page was reached via a referral link
  useEffect(() => {
    const ref = new URLSearchParams(window.location.search).get('ref');
    if (ref) recordClick(ref);
  }, []);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(referral.referralUrl);
      setCopied(true);
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
      timeoutRef.current = setTimeout(() => setCopied(false), 2000);
    } catch {
      // clipboard unavailable — silently ignore
    }
  };

  const refreshStats = useCallback(() => {
    setStats(getStats(referral.referralId));
  }, [referral.referralId]);

  return (
    <section
      className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-6"
      aria-labelledby="referral-heading"
    >
      <h2 id="referral-heading" className="mb-1 text-sm font-semibold">
        Invite &amp; Referral
      </h2>
      <p className="mb-4 text-xs text-[var(--color-text-muted)]">
        Share your unique link to recruit donors for this pool.
      </p>

      {/* Referral link input + copy button */}
      <div className="flex gap-2">
        <input
          readOnly
          value={referral.referralUrl}
          aria-label="Your referral link"
          className="min-w-0 flex-1 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 font-mono text-xs text-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-brand-500"
          onFocus={(e) => e.currentTarget.select()}
        />
        <button
          type="button"
          onClick={handleCopy}
          aria-label={copied ? 'Link copied!' : 'Copy referral link'}
          className="inline-flex shrink-0 items-center gap-1.5 rounded-lg bg-brand-600 px-3 py-2 text-xs font-semibold text-white transition-colors hover:bg-brand-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
        >
          {copied ? (
            <>
              <CheckIcon className="size-3.5" />
              Copied!
            </>
          ) : (
            <>
              <CopyIcon className="size-3.5" />
              Copy
            </>
          )}
        </button>
      </div>

      {/* Stats */}
      <div
        className="mt-4 grid grid-cols-2 gap-3"
        aria-label="Referral statistics"
      >
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3 text-center">
          <p className="text-xl font-bold text-brand-600">{stats.clicks}</p>
          <p className="text-xs text-[var(--color-text-muted)]">Link Clicks</p>
        </div>
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3 text-center">
          <p className="text-xl font-bold text-brand-600">{stats.signups}</p>
          <p className="text-xs text-[var(--color-text-muted)]">Conversions</p>
        </div>
      </div>

      <button
        type="button"
        onClick={refreshStats}
        className="mt-3 text-xs text-[var(--color-text-muted)] underline-offset-2 hover:underline focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
      >
        Refresh stats
      </button>
    </section>
  );
};

function CopyIcon({ className }: { className?: string }) {
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
        d="M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 0 1-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H6.75a9.06 9.06 0 0 1 1.5.124m7.5 10.376h3.375c.621 0 1.125-.504 1.125-1.125V11.25c0-4.46-3.243-8.161-7.5-8.876a9.06 9.06 0 0 0-1.5-.124H9.375c-.621 0-1.125.504-1.125 1.125v3.5m7.5 10.375H9.375a1.125 1.125 0 0 1-1.125-1.125v-9.25m12 6.625v-1.875a3.375 3.375 0 0 0-3.375-3.375h-1.5a1.125 1.125 0 0 1-1.125-1.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H9.75"
      />
    </svg>
  );
}

function CheckIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={2}
      stroke="currentColor"
      className={className}
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M4.5 12.75l6 6 9-13.5"
      />
    </svg>
  );
}
