'use client';

import React, { useEffect, useState, Suspense } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import Link from 'next/link';
import { useDonationsStore } from '@/src/store/donationsStore';
import type { Donation } from '@/src/store/donationsStore';

function ReceiptContent() {
  const searchParams = useSearchParams();
  const txHashParam = searchParams.get('txHash');
  const router = useRouter();
  const { history } = useDonationsStore();
  const [donation, setDonation] = useState<Donation | null>(null);

  useEffect(() => {
    if (!txHashParam) {
      router.replace('/');
      return;
    }
    const found = history.find((d) => d.txHash === txHashParam);
    if (!found) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setDonation({
        id: 'mock-id',
        poolId: '1',
        poolName: 'Unknown Pool',
        amount: '0',
        asset: 'XLM',
        txHash: txHashParam,
        timestamp: new Date().toISOString(),
        status: 'confirmed',
      });
    } else {
      setDonation(found);
    }
  }, [txHashParam, history, router]);

  if (!donation) {
    return (
      <div className="flex justify-center py-20">
        <div className="size-8 animate-spin rounded-full border-4 border-[var(--color-border)] border-t-brand-600" />
      </div>
    );
  }

  const handlePrint = () => {
    window.print();
  };

  const shareText = `I just donated ${donation.amount} ${donation.asset} to "${donation.poolName}" on Nevo! Join me in supporting this cause:`;
  const shareUrl =
    typeof window !== 'undefined'
      ? window.location.origin + `/pools/${donation.poolId}`
      : '';
  const twitterUrl = `https://twitter.com/intent/tweet?text=${encodeURIComponent(shareText)}&url=${encodeURIComponent(shareUrl)}`;

  return (
    <main className="mx-auto max-w-2xl px-4 py-12 sm:px-6">
      <div className="print-container overflow-hidden rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm">
        {/* Header */}
        <div className="bg-brand-600 px-6 py-8 text-center text-white">
          <div className="mx-auto mb-4 flex size-16 items-center justify-center rounded-full bg-white/20">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth={2}
              stroke="currentColor"
              className="size-8"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
              />
            </svg>
          </div>
          <h1 className="text-2xl font-bold tracking-tight">
            Donation Receipt
          </h1>
          <p className="mt-1 text-white/80">Thank you for your contribution</p>
        </div>

        {/* Details */}
        <div className="px-6 py-8 sm:px-8">
          <div className="mb-8 text-center">
            <p className="text-sm font-medium text-[var(--color-text-muted)]">
              Donation Amount
            </p>
            <p className="mt-1 text-4xl font-bold text-brand-600">
              {donation.amount} <span className="text-xl">XLM</span>
            </p>
          </div>

          <dl className="grid gap-x-4 gap-y-6 sm:grid-cols-2">
            <div>
              <dt className="text-xs font-medium text-[var(--color-text-muted)]">
                Pool Name
              </dt>
              <dd className="mt-1 text-sm font-semibold">
                {donation.poolName}
              </dd>
            </div>
            <div>
              <dt className="text-xs font-medium text-[var(--color-text-muted)]">
                Date & Time
              </dt>
              <dd className="mt-1 text-sm font-semibold">
                {new Date(donation.timestamp).toLocaleString()}
              </dd>
            </div>
            <div className="sm:col-span-2">
              <dt className="text-xs font-medium text-[var(--color-text-muted)]">
                Transaction Hash
              </dt>
              <dd className="mt-1 font-mono text-xs font-semibold break-all text-[var(--color-text)]">
                {donation.txHash}
              </dd>
            </div>
            <div className="sm:col-span-2">
              <dt className="text-xs font-medium text-[var(--color-text-muted)]">
                Status
              </dt>
              <dd className="mt-1 flex items-center gap-2 text-sm font-semibold text-success-dark">
                <span className="flex size-2 rounded-full bg-success-dark"></span>
                Confirmed on Stellar
              </dd>
            </div>
          </dl>
        </div>

        {/* Actions (Hidden on Print) */}
        <div className="border-t border-[var(--color-border)] bg-[var(--color-surface-raised)] px-6 py-6 sm:px-8 print:hidden">
          <div className="grid gap-3 sm:grid-cols-2">
            <button
              onClick={handlePrint}
              className="flex w-full items-center justify-center gap-2 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-2.5 text-sm font-medium hover:bg-[var(--color-surface-raised)] transition-colors"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                strokeWidth={1.5}
                stroke="currentColor"
                className="size-4"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3"
                />
              </svg>
              Download PDF
            </button>
            <a
              href={twitterUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="flex w-full items-center justify-center gap-2 rounded-xl bg-[#1DA1F2] px-4 py-2.5 text-sm font-semibold text-white hover:bg-[#1a8cd8] transition-colors"
            >
              <svg
                fill="currentColor"
                viewBox="0 0 24 24"
                className="size-4"
                aria-hidden="true"
              >
                <path d="M23.953 4.57a10 10 0 01-2.825.775 4.958 4.958 0 002.163-2.723c-.951.555-2.005.959-3.127 1.184a4.92 4.92 0 00-8.384 4.482C7.69 8.095 4.067 6.13 1.64 3.162a4.822 4.822 0 00-.666 2.475c0 1.71.87 3.213 2.188 4.096a4.904 4.904 0 01-2.228-.616v.06a4.923 4.923 0 003.946 4.827 4.996 4.996 0 01-2.212.085 4.936 4.936 0 004.604 3.417 9.867 9.867 0 01-6.102 2.105c-.39 0-.779-.023-1.17-.067a13.995 13.995 0 007.557 2.209c9.053 0 13.998-7.496 13.998-13.985 0-.21 0-.42-.015-.63A9.935 9.935 0 0024 4.59z" />
              </svg>
              Share on Twitter
            </a>
          </div>
          <div className="mt-4 flex justify-center">
            <Link
              href={`/pools/${donation.poolId}`}
              className="text-sm font-medium text-brand-600 hover:text-brand-700"
            >
              Return to Pool
            </Link>
          </div>
          <div className="mt-4 flex justify-center border-t border-[var(--color-border)] pt-4">
            <a
              href={`https://stellar.expert/explorer/testnet/tx/${donation.txHash}`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text)] underline"
            >
              View on Stellar Explorer
            </a>
          </div>
        </div>
      </div>
      <style
        dangerouslySetInnerHTML={{
          __html: `
        @media print {
          body * {
            visibility: hidden;
          }
          .print-container, .print-container * {
            visibility: visible;
          }
          .print-container {
            position: absolute;
            left: 0;
            top: 0;
            width: 100%;
            border: none;
            box-shadow: none;
          }
        }
      `,
        }}
      />
    </main>
  );
}

export default function ReceiptPage() {
  return (
    <Suspense
      fallback={
        <div className="flex justify-center py-20">
          <div className="size-8 animate-spin rounded-full border-4 border-[var(--color-border)] border-t-brand-600" />
        </div>
      }
    >
      <ReceiptContent />
    </Suspense>
  );
}
