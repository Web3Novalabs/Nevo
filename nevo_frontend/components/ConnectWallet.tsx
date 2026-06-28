'use client';

import { useEffect, useState } from 'react';
import { useWalletStore } from '@/src/store/walletStore';
import { Spinner } from '@/components/Spinner';
import { isFreighterInstalled } from '@/app/stellar-wallets-kit';

const FREIGHTER_INSTALL_URL =
  'https://www.freighter.app/';

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
  const [verifying, setVerifying] = useState(false);
  const [open, setOpen] = useState(false);
  // undefined = not yet checked (SSR), true/false = detection result
  const [freighterInstalled, setFreighterInstalled] = useState<
    boolean | undefined
  >(undefined);

  useEffect(() => {
    setFreighterInstalled(isFreighterInstalled());
    initialize();
  }, [initialize]);

  async function handleConnect() {
    // Guard: do not attempt the flow if the extension is missing
    if (!freighterInstalled) return;

    setError(null);
    setVerifying(true);
    try {
      await connectWallet();
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Connection failed';
      // Freighter surfaces rejection as an error message string
      const isRejected =
        message.toLowerCase().includes('rejected') ||
        message.toLowerCase().includes('denied') ||
        message.toLowerCase().includes('cancelled') ||
        message.toLowerCase().includes('user declined');
      setError(isRejected ? 'Signature request was rejected.' : message);
    } finally {
      setVerifying(false);
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

  // Wait until the client has run the detection before rendering the button
  // to avoid a flash of the wrong state on hydration.
  if (freighterInstalled === undefined) {
    return (
      <div
        className="h-8 w-28 animate-pulse rounded-full bg-zinc-200 dark:bg-zinc-800"
        aria-label="Loading wallet"
      />
    );
  }

  if (!freighterInstalled) {
    return (
      <p className="text-xs text-[var(--color-text-muted)]" role="alert">
        Freighter wallet not found.{' '}
        <a
          href={FREIGHTER_INSTALL_URL}
          target="_blank"
          rel="noopener noreferrer"
          className="underline hover:text-brand-600 transition-colors"
        >
          Install Freighter
        </a>
      </p>
    );
  }

  if (!publicKey) {
    return (
      <div className="flex flex-col items-end gap-1">
        <button
          onClick={handleConnect}
          disabled={verifying}
          aria-busy={verifying}
          className="flex items-center gap-2 rounded-full bg-brand-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600 disabled:opacity-70 disabled:cursor-not-allowed"
        >
          {verifying ? (
            <>
              <Spinner size="sm" color="white" label="Verifying…" />
              Verifying…
            </>
          ) : (
            'Connect Wallet'
          )}
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
        aria-haspopup="dialog"
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
          >
            Disconnect
          </button>
        </div>
      )}
    </div>
  );
}
