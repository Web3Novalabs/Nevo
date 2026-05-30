'use client';

import { useEffect, useState } from 'react';
import { RATE_LIMIT_EVENT, type RateLimitNotification } from '@/lib/rate-limit';

type ActiveNotification = RateLimitNotification & { id: number };

export function RateLimitToast() {
  const [notification, setNotification] = useState<ActiveNotification | null>(
    null
  );
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    function handleRateLimit(event: Event) {
      const detail = (event as CustomEvent<RateLimitNotification>).detail;
      setNotification({ ...detail, id: Date.now() });
      setNow(Date.now());
    }

    window.addEventListener(RATE_LIMIT_EVENT, handleRateLimit);
    return () => window.removeEventListener(RATE_LIMIT_EVENT, handleRateLimit);
  }, []);

  useEffect(() => {
    if (!notification) return;

    const interval = window.setInterval(() => setNow(Date.now()), 1000);
    const dismissAfterMs = Math.max(
      4_000,
      notification.resetAt - Date.now() + 4_000
    );
    const timeout = window.setTimeout(
      () => setNotification(null),
      dismissAfterMs
    );

    return () => {
      window.clearInterval(interval);
      window.clearTimeout(timeout);
    };
  }, [notification]);

  if (!notification) return null;

  const remainingSeconds = Math.max(
    0,
    Math.ceil((notification.resetAt - now) / 1000)
  );

  return (
    <div className="fixed inset-x-4 bottom-4 z-50 sm:inset-x-auto sm:right-6 sm:bottom-6 sm:w-full sm:max-w-sm">
      <div
        role="status"
        aria-live="polite"
        className="rounded-lg border border-warning bg-warning-light p-4 text-warning-dark shadow-lg"
      >
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start">
          <div className="min-w-0 flex-1">
            <p className="text-sm font-semibold">Too many requests</p>
            <p className="mt-1 text-sm leading-5">{notification.message}</p>
            <p className="mt-2 text-xs font-medium">
              {remainingSeconds > 0
                ? `Try again in ${remainingSeconds}s.`
                : 'You can try again now.'}
            </p>
          </div>
          <button
            type="button"
            onClick={() => setNotification(null)}
            className="self-start rounded-md border border-warning/40 px-3 py-1.5 text-xs font-semibold transition-colors hover:bg-warning/10 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-warning-dark"
          >
            Dismiss
          </button>
        </div>
      </div>
    </div>
  );
}
