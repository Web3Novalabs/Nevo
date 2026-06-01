'use client';

import React, { FC, FormEvent, useState } from 'react';

type Status = 'idle' | 'loading' | 'success' | 'error';

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

/** Simulates a newsletter subscription API call. Replace with real endpoint. */
async function subscribeEmail(email: string): Promise<void> {
  // TODO: replace with real API call, e.g. apiClient.post('/newsletter/subscribe', { email })
  await new Promise<void>((resolve) => setTimeout(resolve, 600));
  if (!EMAIL_RE.test(email)) throw new Error('Invalid email');
}

export const NewsletterSignup: FC = () => {
  const [email, setEmail] = useState('');
  const [status, setStatus] = useState<Status>('idle');
  const [validationError, setValidationError] = useState('');

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setValidationError('');

    if (!email.trim()) {
      setValidationError('Email is required.');
      return;
    }
    if (!EMAIL_RE.test(email)) {
      setValidationError('Please enter a valid email address.');
      return;
    }

    setStatus('loading');
    try {
      await subscribeEmail(email);
      setStatus('success');
      setEmail('');
    } catch {
      setStatus('error');
    }
  };

  if (status === 'success') {
    return (
      <div
        role="status"
        aria-live="polite"
        className="flex flex-col items-center gap-1 text-center sm:items-start sm:text-left"
      >
        <p className="text-sm font-semibold text-brand-600">
          You&apos;re subscribed! 🎉
        </p>
        <p className="text-xs text-[var(--color-text-muted)]">
          You&apos;ll receive updates about new pools and announcements.
          Unsubscribe anytime via the link in any email.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2">
      <p className="text-sm font-semibold text-[var(--color-text)]">
        Stay updated
      </p>
      <p className="text-xs text-[var(--color-text-muted)]">
        Get notified about new pools and platform news.
      </p>
      <form
        onSubmit={handleSubmit}
        noValidate
        aria-label="Newsletter signup"
        className="flex flex-col gap-2 sm:flex-row"
      >
        <div className="flex flex-1 flex-col gap-1">
          <label htmlFor="newsletter-email" className="sr-only">
            Email address
          </label>
          <input
            id="newsletter-email"
            type="email"
            name="email"
            autoComplete="email"
            placeholder="you@example.com"
            value={email}
            onChange={(e) => {
              setEmail(e.target.value);
              setValidationError('');
              if (status === 'error') setStatus('idle');
            }}
            aria-invalid={!!validationError}
            aria-describedby={
              validationError
                ? 'newsletter-error'
                : status === 'error'
                  ? 'newsletter-server-error'
                  : undefined
            }
            className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-brand-500"
          />
          {validationError && (
            <p
              id="newsletter-error"
              role="alert"
              className="text-xs text-[var(--color-error,#ef4444)]"
            >
              {validationError}
            </p>
          )}
          {status === 'error' && !validationError && (
            <p
              id="newsletter-server-error"
              role="alert"
              className="text-xs text-[var(--color-error,#ef4444)]"
            >
              Something went wrong. Please try again.
            </p>
          )}
        </div>
        <button
          type="submit"
          disabled={status === 'loading'}
          className="shrink-0 rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-brand-700 disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
        >
          {status === 'loading' ? 'Subscribing…' : 'Subscribe'}
        </button>
      </form>
      <p className="text-xs text-[var(--color-text-muted)]">
        GDPR compliant. Unsubscribe anytime.
      </p>
    </div>
  );
};
