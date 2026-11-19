'use client';

import { useEffect, useRef, useState } from 'react';
import { useWalletStore } from '@/src/store/walletStore';
import { Spinner } from '@/components/Spinner';
import {
  SUPPORTED_WALLETS,
  isWalletAvailable,
  type WalletId,
} from '@/app/stellar-wallets-kit';

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
  const [pickerOpen, setPickerOpen] = useState(false);
  // undefined = not yet checked (SSR), otherwise per-wallet detection results
  const [availability, setAvailability] = useState<
    Record<WalletId, boolean> | undefined
  >(undefined);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    const results = {} as Record<WalletId, boolean>;
    for (const wallet of SUPPORTED_WALLETS) {
      results[wallet.id] = isWalletAvailable(wallet.id);
    }
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setAvailability(results);
    initialize();
  }, [initialize]);

  useEffect(() => {
    if (!open) return;

    function handleClickOutside(event: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setOpen(false);
      }
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        setOpen(false);
        triggerRef.current?.focus();
      }
    }

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [open]);

  async function handleConnect(walletId: WalletId) {
    setError(null);
    setVerifying(true);
    try {
      await connectWallet(walletId);
      setPickerOpen(false);
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Connection failed';
      // Wallets surface rejection as an error message string
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
  if (availability === undefined) {
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
          type="button"
          onClick={() => {
            setError(null);
            setPickerOpen(true);
          }}
          disabled={verifying}
          aria-busy={verifying}
          aria-haspopup="dialog"
          aria-expanded={pickerOpen}
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
        {error && !pickerOpen && (
          <p className="text-xs text-red-500" role="alert">
            {error}
          </p>
        )}

        {pickerOpen && (
          <div
            className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
            onClick={() => {
              if (!verifying) setPickerOpen(false);
            }}
          >
            <div
              className="w-full max-w-sm rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-lg"
              role="dialog"
              aria-modal="true"
              aria-label="Choose a wallet"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="mb-3 flex items-center justify-between">
                <h2 className="text-sm font-semibold">Connect a wallet</h2>
                <button
                  type="button"
                  onClick={() => setPickerOpen(false)}
                  disabled={verifying}
                  aria-label="Close wallet picker"
                  className="rounded p-1 text-[var(--color-text-muted)] hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors disabled:opacity-50"
                >
                  ✕
                </button>
              </div>
              <ul className="space-y-2">
                {SUPPORTED_WALLETS.map((wallet) => {
                  const installed = availability[wallet.id];
                  return (
                    <li key={wallet.id}>
                      <button
                        type="button"
                        onClick={() => handleConnect(wallet.id)}
                        disabled={verifying}
                        className="flex w-full items-center justify-between rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm font-medium hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors disabled:opacity-70 disabled:cursor-not-allowed"
                      >
                        <span>{wallet.name}</span>
                        {!installed && (
                          <span className="text-xs font-normal text-[var(--color-text-muted)]">
                            Not detected
                          </span>
                        )}
                      </button>
                    </li>
                  );
                })}
              </ul>
              {verifying && (
                <div className="mt-3 flex items-center gap-2 text-xs text-[var(--color-text-muted)]">
                  <Spinner size="sm" label="Verifying…" />
                  Verifying…
                </div>
              )}
              {error && (
                <p className="mt-3 text-xs text-red-500" role="alert">
                  {error}
                </p>
              )}
            </div>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        type="button"
        ref={triggerRef}
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
            type="button"
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
