'use client';

import { Suspense, useEffect, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import Link from 'next/link';
import { useWalletStore } from '@/src/store/walletStore';
import ConnectWallet from '@/components/ConnectWallet';

function LoginPageContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { publicKey, loading, initialize } = useWalletStore();
  const [accepted, setAccepted] = useState(false);

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

        <div className="mt-8">
          <label className="flex items-start gap-3">
            <input
              type="checkbox"
              checked={accepted}
              onChange={(e) => setAccepted(e.target.checked)}
              className="mt-1 h-4 w-4 rounded text-brand-600"
            />
            <div className="text-sm text-[var(--color-text-muted)]">
              I agree to the{' '}
              <Link href="/terms" className="font-medium text-brand-600">
                Terms of Service
              </Link>{' '}
              and{' '}
              <Link href="/privacy" className="font-medium text-brand-600">
                Privacy Policy
              </Link>
              .
            </div>
          </label>

          <div className="mt-4 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
            {accepted ? (
              <ConnectWallet />
            ) : (
              <div className="flex items-center justify-center">
                <button
                  disabled
                  className="rounded-full bg-[var(--color-border)] px-6 py-3 text-sm font-medium text-[var(--color-text-muted)]"
                >
                  Accept terms to connect
                </button>
              </div>
            )}
          </div>
        </div>

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
          <Link
            href="/"
            className="font-medium hover:text-brand-600 transition-colors"
          >
            ← Back to home
          </Link>
        </p>
      </div>
    </main>
  );
}

export default function LoginPage() {
  return (
    <Suspense fallback={null}>
      <LoginPageContent />
    </Suspense>
  );
}
