'use client';

import React, { useState, useCallback, useEffect, useRef } from 'react';

export interface WalletAddressProps {
  /** The full Stellar wallet address to display */
  address: string;
  /** Whether to truncate the address on mobile. Defaults to true */
  truncate?: boolean;
  /** Number of leading chars to show when truncated. Defaults to 6 */
  leadChars?: number;
  /** Number of trailing chars to show when truncated. Defaults to 6 */
  trailChars?: number;
  /** Additional class names for the root element */
  className?: string;
}

function truncateAddress(address: string, lead: number, trail: number): string {
  if (address.length <= lead + trail + 3) return address;
  return `${address.slice(0, lead)}…${address.slice(-trail)}`;
}

export function WalletAddress({
  address,
  truncate = true,
  leadChars = 6,
  trailChars = 6,
  className = '',
}: WalletAddressProps) {
  const [copied, setCopied] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Clean up timeout on unmount
  useEffect(() => {
    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, []);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(address);
      setCopied(true);
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
      timeoutRef.current = setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback for browsers without clipboard API
      const el = document.createElement('textarea');
      el.value = address;
      el.style.position = 'fixed';
      el.style.opacity = '0';
      document.body.appendChild(el);
      el.select();
      document.execCommand('copy');
      document.body.removeChild(el);
      setCopied(true);
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
      timeoutRef.current = setTimeout(() => setCopied(false), 2000);
    }
  }, [address]);

  const truncated = truncateAddress(address, leadChars, trailChars);

  return (
    <div
      className={`inline-flex items-center gap-2 ${className}`}
      role="group"
      aria-label="Wallet address"
    >
      {truncate ? (
        <>
          {/* Full address — desktop */}
          <span
            className="font-mono text-sm text-[var(--color-text)] bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg px-3 py-1.5 hidden md:inline"
            title={address}
            aria-label={`Full wallet address: ${address}`}
          >
            {address}
          </span>

          {/* Truncated address — mobile */}
          <span
            className="font-mono text-sm text-[var(--color-text)] bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg px-3 py-1.5 md:hidden"
            title={address}
            aria-label={`Wallet address: ${truncated}`}
          >
            {truncated}
          </span>
        </>
      ) : (
        /* Full address always visible */
        <span
          className="font-mono text-sm text-[var(--color-text)] bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg px-3 py-1.5 break-all"
          title={address}
          aria-label={`Full wallet address: ${address}`}
        >
          {address}
        </span>
      )}

      {/* Copy button */}
      <button
        type="button"
        onClick={handleCopy}
        aria-label={copied ? 'Address copied' : 'Copy wallet address'}
        aria-live="polite"
        className="flex-shrink-0 flex items-center justify-center size-8 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-raised)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-border)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
      >
        {copied ? <CheckIcon /> : <CopyIcon />}
      </button>

      {/* Toast notification */}
      <span
        role="status"
        aria-live="polite"
        aria-atomic="true"
        className={`pointer-events-none fixed bottom-5 left-1/2 -translate-x-1/2 z-50 flex items-center gap-2 rounded-full bg-[var(--color-text)] text-[var(--color-surface)] px-4 py-2 text-sm font-medium shadow-lg transition-all duration-300 ${copied ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-2'}`}
      >
        <CheckIcon className="size-4 text-success" />
        Address copied to clipboard
      </span>
    </div>
  );
}

/* ── Icons ─────────────────────────────────────────────────────────────── */

function CopyIcon({ className = 'size-4' }: { className?: string }) {
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

function CheckIcon({ className = 'size-4' }: { className?: string }) {
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
        d="m4.5 12.75 6 6 9-13.5"
      />
    </svg>
  );
}
