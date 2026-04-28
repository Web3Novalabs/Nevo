'use client';

import { useEffect, useState } from 'react';
import { useWalletStore } from '@/src/store/walletStore';

function shortKey(key: string) {
  return `${key.slice(0, 4)}…${key.slice(-4)}`;
}

export default function ConnectWallet() {
  const {
    publicKey,
    balances,
    loading,
    connectWallet,
    disconnectWallet,
    initialize,
  } = useWalletStore();
  const [error, setError] = useState<string | null>(null);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    initialize();
  }, [initialize]);

  async function handleConnect() {
    setError(null);
    try {
      await connectWallet();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Connection failed');
    }
  }

  if (loading) {
    return (
      <div
        className="h-8 w-28 animate-pulse rounded-full bg-zinc-200 dark:bg-zinc-800"
        aria-label="Loading wallet"
      />
    );
  }

  if (!publicKey) {
    return (
      <div className="flex flex-col items-end gap-1">
        <button
          onClick={handleConnect}
          className="rounded-full bg-brand-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
          aria-label="Connect Freighter wallet"
        >
          Connect Wallet
        </button>
        {error && (
          <p className="text-xs text-red-500" role="alert">
            {error}
          </p>
        )}
      </div>
    );
  }

  return (
    <div className="relative">
      <button
        onClick={() => setOpen((o) => !o)}
        className="flex items-center gap-2 rounded-full border border-[var(--color-border)] bg-[var(--color-surface-raised)] px-3 py-1.5 text-sm font-medium hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
        aria-haspopup="true"
        aria-expanded={open}
        aria-label={`Wallet connected: ${publicKey}`}
      >
        <span className="size-2 rounded-full bg-brand-500" aria-hidden="true" />
        {shortKey(publicKey)}
      </button>

      {open && (
        <div
          className="absolute right-0 mt-2 w-56 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-lg z-50"
          role="dialog"
          aria-label="Wallet details"
        >
          <p className="mb-2 text-xs text-[var(--color-text-muted)] break-all">
            {publicKey}
          </p>
          {balances && (
            <div className="mb-3 space-y-1 text-sm">
              <div className="flex justify-between">
                <span className="text-[var(--color-text-muted)]">XLM</span>
                <span className="font-medium">{balances.xlm}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[var(--color-text-muted)]">USDC</span>
                <span className="font-medium">{balances.usdc}</span>
              </div>
            </div>
          )}
          <button
            onClick={() => {
              disconnectWallet();
              setOpen(false);
            }}
            className="w-full rounded-lg border border-[var(--color-border)] py-1.5 text-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-950 transition-colors"
            aria-label="Disconnect wallet"
          >
            Disconnect
          </button>
        </div>
      )}
    </div>
  );
}
