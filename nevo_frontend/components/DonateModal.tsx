'use client';

import React, { useEffect, useRef, useState } from 'react';
import { useWalletStore } from '@/src/store/walletStore';
import { useDonationsStore } from '@/src/store/donationsStore';
import type { Pool } from '@/src/store/poolsStore';
import { WalletAddress } from './WalletAddress';

type Asset = 'XLM' | 'USDC';
type Step = 'form' | 'loading' | 'success' | 'error';

const MIN_AMOUNT = 1;
const MAX_AMOUNT = 100_000;
// Mock fee estimate (Stellar base fee in XLM)
const TX_FEE_XLM = '0.00001';

interface DonateModalProps {
  pool: Pool;
  onClose: () => void;
}

export function DonateModal({ pool, onClose }: DonateModalProps) {
  const { publicKey, balances } = useWalletStore();
  const { addDonation } = useDonationsStore();

  const [asset, setAsset] = useState<Asset>('XLM');
  const [amount, setAmount] = useState('');
  const [step, setStep] = useState<Step>('form');
  const [errorMsg, setErrorMsg] = useState('');
  const backdropRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus amount input on open
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Close on Escape
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose();
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [onClose]);

  const availableBalance =
    asset === 'XLM' ? (balances?.xlm ?? '0') : (balances?.usdc ?? '0');

  const numAmount = parseFloat(amount);
  const amountValid =
    !isNaN(numAmount) && numAmount >= MIN_AMOUNT && numAmount <= MAX_AMOUNT;
  const amountError =
    amount !== '' && !amountValid
      ? numAmount < MIN_AMOUNT
        ? `Minimum donation is ${MIN_AMOUNT} ${asset}`
        : `Maximum donation is ${MAX_AMOUNT.toLocaleString()} ${asset}`
      : '';

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!amountValid || !publicKey) return;

    setStep('loading');

    // Call the donate API
    try {
      // Import this at the top: import { donate } from '@/lib/api-client';
      // Wait, we can just use the store if donate is added there, or apiClient directly
      const tokenAddress = asset === 'XLM' ? 'native' : 'usdc_address'; // Placeholder for token address
      // await donate(pool.id, amount, tokenAddress); // Uncomment when implemented
      
      const donation = {
        id: `mock-${Date.now()}`,
        poolId: pool.id,
        poolName: pool.title,
        amount,
        asset,
        txHash: `mock-tx-${Math.random().toString(36).slice(2)}`,
        timestamp: new Date().toISOString(),
        status: 'confirmed' as const,
      };
      addDonation(donation);
      setStep('success');
    } catch (err) {
      setErrorMsg('Transaction rejected by the network. Please try again.');
      setStep('error');
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-end justify-center sm:items-center p-0 sm:p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="donate-modal-title"
    >
      {/* Backdrop */}
      <div
        ref={backdropRef}
        className="absolute inset-0 bg-black/50"
        onClick={onClose}
        aria-hidden="true"
      />

      <div className="relative w-full max-w-md max-h-[100dvh] rounded-t-2xl sm:rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xl overflow-y-auto">
        {/* Header */}
        <div className="mb-5 flex items-start justify-between gap-3">
          <div>
            <h2
              id="donate-modal-title"
              className="text-base font-semibold leading-tight"
            >
              Donate to Pool
            </h2>
            <p className="mt-0.5 text-sm text-[var(--color-text-muted)] line-clamp-1">
              {pool.title}
            </p>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close donation modal"
            className="flex-shrink-0 flex min-h-11 min-w-11 items-center justify-center rounded-lg border border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-surface-raised)] transition-colors"
          >
            <CloseIcon />
          </button>
        </div>

        {/* ── Form step ── */}
        {step === 'form' && (
          <form onSubmit={handleSubmit} noValidate>
            {/* Wallet confirmation */}
            {publicKey && (
              <div className="mb-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-3">
                <p className="mb-1.5 text-xs text-[var(--color-text-muted)]">
                  Donating from
                </p>
                <WalletAddress address={publicKey} className="text-xs" />
              </div>
            )}

            {/* Denomination selector */}
            <div className="mb-4">
              <label className="mb-1.5 block text-sm font-medium">
                Currency
              </label>
              <div
                role="radiogroup"
                aria-label="Select currency"
                className="flex gap-2"
              >
                {(['XLM', 'USDC'] as Asset[]).map((a) => (
                  <button
                    key={a}
                    type="button"
                    role="radio"
                    aria-checked={asset === a}
                    onClick={() => setAsset(a)}
                    className={`flex-1 rounded-lg border px-3 py-2 text-sm font-medium transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600 ${
                      asset === a
                        ? 'border-brand-600 bg-brand-50 text-brand-700'
                        : 'border-[var(--color-border)] hover:bg-[var(--color-surface-raised)]'
                    }`}
                  >
                    {a}
                  </button>
                ))}
              </div>
              <p className="mt-1.5 text-xs text-[var(--color-text-muted)]">
                Available:{' '}
                <span className="font-medium">
                  {availableBalance} {asset}
                </span>
              </p>
            </div>

            {/* Amount input */}
            <div className="mb-4">
              <label
                htmlFor="donate-amount"
                className="mb-1.5 block text-sm font-medium"
              >
                Amount
              </label>
              <div className="relative">
                <input
                  ref={inputRef}
                  id="donate-amount"
                  type="number"
                  inputMode="decimal"
                  min={MIN_AMOUNT}
                  max={MAX_AMOUNT}
                  step="any"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder={`Min ${MIN_AMOUNT} ${asset}`}
                  aria-describedby={
                    amountError ? 'amount-error' : 'amount-hint'
                  }
                  aria-invalid={!!amountError}
                  className="w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-2.5 pr-16 text-sm focus:border-brand-600 focus:outline-none focus:ring-1 focus:ring-brand-600 aria-[invalid=true]:border-error"
                />
                <span className="pointer-events-none absolute right-4 top-1/2 -translate-y-1/2 text-sm font-medium text-[var(--color-text-muted)]">
                  {asset}
                </span>
              </div>
              {amountError ? (
                <p
                  id="amount-error"
                  role="alert"
                  className="mt-1 text-xs text-error"
                >
                  {amountError}
                </p>
              ) : (
                <p
                  id="amount-hint"
                  className="mt-1 text-xs text-[var(--color-text-muted)]"
                >
                  Max {MAX_AMOUNT.toLocaleString()} {asset}
                </p>
              )}
            </div>

            {/* Transaction fee estimate */}
            <div className="mb-5 flex items-center justify-between rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] px-4 py-2.5 text-sm">
              <span className="text-[var(--color-text-muted)]">
                Estimated network fee
              </span>
              <span className="font-medium">{TX_FEE_XLM} XLM</span>
            </div>

            <button
              type="submit"
              disabled={!amountValid || !publicKey}
              className="w-full rounded-xl bg-brand-600 px-4 py-2.5 text-sm font-semibold text-white hover:bg-brand-700 transition-colors disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
            >
              Donate {amountValid ? `${amount} ${asset}` : ''}
            </button>

            {!publicKey && (
              <p className="mt-3 text-center text-xs text-[var(--color-text-muted)]">
                Connect your wallet to donate.
              </p>
            )}
          </form>
        )}

        {/* ── Loading step ── */}
        {step === 'loading' && (
          <div
            className="flex flex-col items-center gap-4 py-8"
            aria-live="polite"
            aria-busy="true"
          >
            <div className="size-12 animate-spin rounded-full border-4 border-[var(--color-border)] border-t-brand-600" />
            <p className="text-sm font-medium">Processing transaction…</p>
            <p className="text-xs text-[var(--color-text-muted)]">
              Please wait while your donation is being submitted.
            </p>
          </div>
        )}

        {/* ── Success step ── */}
        {step === 'success' && (
          <div
            className="flex flex-col items-center gap-4 py-6 text-center"
            aria-live="polite"
          >
            <div className="flex size-14 items-center justify-center rounded-full bg-success-light text-success">
              <CheckCircleIcon />
            </div>
            <div>
              <p className="font-semibold">Donation successful!</p>
              <p className="mt-1 text-sm text-[var(--color-text-muted)]">
                You donated{' '}
                <span className="font-medium">
                  {amount} {asset}
                </span>{' '}
                to <span className="font-medium">{pool.title}</span>.
              </p>
            </div>
            <button
              type="button"
              onClick={onClose}
              className="mt-2 w-full rounded-xl bg-brand-600 px-4 py-2.5 text-sm font-semibold text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
            >
              Done
            </button>
          </div>
        )}

        {/* ── Error step ── */}
        {step === 'error' && (
          <div
            className="flex flex-col items-center gap-4 py-6 text-center"
            aria-live="polite"
          >
            <div className="flex size-14 items-center justify-center rounded-full bg-error-light text-error">
              <XCircleIcon />
            </div>
            <div>
              <p className="font-semibold">Donation failed</p>
              <p className="mt-1 text-sm text-[var(--color-text-muted)]">
                {errorMsg}
              </p>
            </div>
            <div className="mt-2 flex w-full gap-3">
              <button
                type="button"
                onClick={onClose}
                className="flex-1 rounded-xl border border-[var(--color-border)] px-4 py-2.5 text-sm font-medium hover:bg-[var(--color-surface-raised)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={() => {
                  setStep('form');
                  setErrorMsg('');
                }}
                className="flex-1 rounded-xl bg-brand-600 px-4 py-2.5 text-sm font-semibold text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
              >
                Try again
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

/* ── Icons ─────────────────────────────────────────────────────────────── */

function CloseIcon() {
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
        d="M6 18 18 6M6 6l12 12"
      />
    </svg>
  );
}

function CheckCircleIcon() {
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
        d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
      />
    </svg>
  );
}

function XCircleIcon() {
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
        d="m9.75 9.75 4.5 4.5m0-4.5-4.5 4.5M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
      />
    </svg>
  );
}
