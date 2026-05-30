'use client';

import React, { useEffect, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import Link from 'next/link';
import { useWalletStore } from '@/src/store/walletStore';
import ConnectWallet from '@/components/ConnectWallet';

export default function LoginPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { publicKey, loading, initialize } = useWalletStore();
  const [error, setError] = useState<string | null>(null);

  const from = searchParams.get('from') || '/dashboard';

  useEffect(() => {
    initialize();
  }, [initialize]);

  useEffect(() => {
    if (!loading && publicKey) {
      router.push(from);
    }
  }, [loading, publicKey, from, router]);

  return (
    <main className="flex min-h-[calc(100vh-56px)] items-center justify-center px-6 py-12">
      <div className="w-full max-w-sm">
        <div className="text-center">
          <h1 className="text-2xl font-bold tracking-tight">Sign In</h1>
          <p className="mt-2 text-sm text-[var(--color-text-muted)]">
            Connect your Stellar wallet to continue
          </p>
        </div>

        <div className="mt-8 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
          <ConnectWallet />
        </div>

        {error && (
          <p className="mt-4 text-sm text-center text-[var(--color-error)]">
            {error}
          </p>
        )}

        <p className="mt-8 text-center text-sm text-[var(--color-text-muted)]">
          Don&apos;t have a wallet?{' '}
          <a
            href="https://www.freighter.app/"
            target="_blank"
            rel="noopener noreferrer"
            className="font-medium text-brand-600 hover:text-brand-700 transition-colors"
          >
            Install Freighter
          </a>
        </p>

        <p className="mt-4 text-center text-sm text-[var(--color-text-muted)]">
          <Link href="/" className="font-medium hover:text-brand-600 transition-colors">
            ← Back to home
          </Link>
        </p>
      </div>
    </main>
  );
}
