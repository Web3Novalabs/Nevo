export interface RateLimitOptions {
  maxRequests: number;
  windowMs: number;
}

export interface RateLimitResult extends RateLimitOptions {
  allowed: boolean;
  remaining: number;
  resetAt: number;
  retryAfterMs: number;
}

interface WindowCounter {
  count: number;
  resetAt: number;
}

export interface RateLimitNotification {
  message: string;
  retryAfterMs: number;
  resetAt: number;
  limit: number;
  endpoint?: string;
}

export const RATE_LIMIT_EVENT = 'nevo:api-rate-limit';

const DEFAULT_MAX_REQUESTS = 30;
const DEFAULT_WINDOW_MS = 60_000;

function readPositiveEnvNumber(name: string, fallback: number): number {
  if (typeof process === 'undefined') return fallback;
  const value = process.env[name];
  if (!value) return fallback;

  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

export const DEFAULT_RATE_LIMIT_OPTIONS: RateLimitOptions = {
  maxRequests: readPositiveEnvNumber(
    'NEXT_PUBLIC_API_RATE_LIMIT_MAX_REQUESTS',
    DEFAULT_MAX_REQUESTS
  ),
  windowMs: readPositiveEnvNumber(
    'NEXT_PUBLIC_API_RATE_LIMIT_WINDOW_MS',
    DEFAULT_WINDOW_MS
  ),
};

export function resolveRateLimitOptions(
  options: Partial<RateLimitOptions> = {}
): RateLimitOptions {
  return {
    maxRequests:
      typeof options.maxRequests === 'number' && options.maxRequests > 0
        ? Math.floor(options.maxRequests)
        : DEFAULT_RATE_LIMIT_OPTIONS.maxRequests,
    windowMs:
      typeof options.windowMs === 'number' && options.windowMs > 0
        ? Math.floor(options.windowMs)
        : DEFAULT_RATE_LIMIT_OPTIONS.windowMs,
  };
}

export class RateLimitError extends Error {
  readonly status = 429;
  readonly retryAfterMs: number;
  readonly resetAt: number;
  readonly remaining: number;
  readonly limit: number;
  readonly windowMs: number;
  readonly endpoint?: string;

  constructor(result: RateLimitResult, endpoint?: string) {
    const seconds = Math.max(1, Math.ceil(result.retryAfterMs / 1000));
    super(
      `You're sending requests too quickly. Please try again in ${seconds} second${
        seconds === 1 ? '' : 's'
      }.`
    );
    this.name = 'RateLimitError';
    this.retryAfterMs = result.retryAfterMs;
    this.resetAt = result.resetAt;
    this.remaining = result.remaining;
    this.limit = result.maxRequests;
    this.windowMs = result.windowMs;
    this.endpoint = endpoint;
  }
}

export function isRateLimitError(error: unknown): error is RateLimitError {
  return (
    error instanceof RateLimitError ||
    (typeof error === 'object' &&
      error !== null &&
      'name' in error &&
      error.name === 'RateLimitError' &&
      'retryAfterMs' in error &&
      'resetAt' in error)
  );
}

export function getRateLimitRemainingMs(error: RateLimitError): number {
  return Math.max(0, error.resetAt - Date.now());
}

export function parseRetryAfterHeader(
  retryAfter: string | null,
  now: number = Date.now()
): number | null {
  if (!retryAfter) return null;

  const seconds = Number(retryAfter);
  if (Number.isFinite(seconds) && seconds >= 0) {
    return seconds * 1000;
  }

  const dateMs = Date.parse(retryAfter);
  if (Number.isFinite(dateMs)) {
    return Math.max(0, dateMs - now);
  }

  return null;
}

export function notifyRateLimit(error: RateLimitError): void {
  if (typeof window === 'undefined') return;

  const detail: RateLimitNotification = {
    message: error.message,
    retryAfterMs: error.retryAfterMs,
    resetAt: error.resetAt,
    limit: error.limit,
    endpoint: error.endpoint,
  };

  window.dispatchEvent(new CustomEvent(RATE_LIMIT_EVENT, { detail }));
}

export class ClientRateLimiter {
  private windows = new Map<string, WindowCounter>();

  constructor(private readonly now: () => number = () => Date.now()) {}

  consume(key: string, options: RateLimitOptions): RateLimitResult {
    const currentTime = this.now();
    const window = this.getWindow(key, currentTime, options.windowMs);

    if (window.count >= options.maxRequests) {
      return {
        ...options,
        allowed: false,
        remaining: 0,
        resetAt: window.resetAt,
        retryAfterMs: Math.max(0, window.resetAt - currentTime),
      };
    }

    window.count += 1;

    return {
      ...options,
      allowed: true,
      remaining: Math.max(0, options.maxRequests - window.count),
      resetAt: window.resetAt,
      retryAfterMs: 0,
    };
  }

  getStatus(key: string, options: RateLimitOptions): RateLimitResult {
    const currentTime = this.now();
    const window = this.getWindow(key, currentTime, options.windowMs);

    return {
      ...options,
      allowed: window.count < options.maxRequests,
      remaining: Math.max(0, options.maxRequests - window.count),
      resetAt: window.resetAt,
      retryAfterMs:
        window.count >= options.maxRequests
          ? Math.max(0, window.resetAt - currentTime)
          : 0,
    };
  }

  reset(key?: string): void {
    if (key) {
      this.windows.delete(key);
      return;
    }

    this.windows.clear();
  }

  private getWindow(
    key: string,
    currentTime: number,
    windowMs: number
  ): WindowCounter {
    const existing = this.windows.get(key);

    if (existing && currentTime < existing.resetAt) {
      return existing;
    }

    const freshWindow = {
      count: 0,
      resetAt: currentTime + windowMs,
    };
    this.windows.set(key, freshWindow);
    return freshWindow;
  }
}
