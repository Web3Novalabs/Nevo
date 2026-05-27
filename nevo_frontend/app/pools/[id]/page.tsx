'use client';

import Link from 'next/link';
import { useParams } from 'next/navigation';
import { useMemo, useState } from 'react';
import { useWalletStore } from '@/src/store/walletStore';
import type { Pool } from '@/src/store/poolsStore';
import { getAccountBalances } from '@/lib/stellar';

type WithdrawStatus = 'idle' | 'submitting' | 'success' | 'error';

const MOCK_POOLS: Pool[] = [
  {
    id: '1',
    title: 'Clean Water Initiative',
    description: 'Providing clean drinking water to rural communities in need.',
    category: 'Humanitarian',
    status: 'Completed',
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
  );
}
