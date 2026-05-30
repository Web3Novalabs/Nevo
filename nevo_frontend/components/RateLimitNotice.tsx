'use client';

import { useEffect, useState } from 'react';
import { isRateLimitError } from '@/lib/api-client';

interface RateLimitNoticeProps {
  error: unknown;
  onRetry?: () => void;
  className?: string;
}

export function RateLimitNotice({
  error,
  onRetry,
  className = '',
}: RateLimitNoticeProps) {
  const rateLimitError = isRateLimitError(error) ? error : null;
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    if (!rateLimitError) return;

    function tick() {
      setNow(Date.now());
    }

    tick();
    const interval = window.setInterval(tick, 1000);
    return () => window.clearInterval(interval);
  }, [rateLimitError]);

  if (!rateLimitError) return null;

  const remainingMs = Math.max(0, rateLimitError.resetAt - now);
  const remainingSeconds = Math.ceil(remainingMs / 1000);
  const canRetry = remainingMs <= 0 && Boolean(onRetry);

  return (
    <div
      role="alert"
      className={`rounded-lg border border-warning bg-warning-light p-4 text-warning-dark ${className}`}
    >
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="min-w-0">
          <p className="text-sm font-semibold">Request paused</p>
          <p className="mt-1 text-sm leading-5">{rateLimitError.message}</p>
        </div>
        <button
          type="button"
          onClick={onRetry}
          disabled={!canRetry}
          className="w-full rounded-md bg-warning-dark px-4 py-2 text-sm font-semibold text-warning-light transition-colors disabled:cursor-not-allowed disabled:opacity-60 sm:w-auto"
        >
          {remainingSeconds > 0 ? `Retry in ${remainingSeconds}s` : 'Retry now'}
        </button>
      </div>
    </div>
  );
}
