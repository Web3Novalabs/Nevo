import {
  ClientRateLimiter,
  DEFAULT_RATE_LIMIT_OPTIONS,
  parseRetryAfterHeader,
  resolveRateLimitOptions,
  RATE_LIMIT_EVENT,
  RateLimitError,
} from './rate-limit';

describe('ClientRateLimiter', () => {
  it('allows requests until the limit is reached and then blocks until the window resets', () => {
    const now = jest.fn();
    const limiter = new ClientRateLimiter(now);
    const options = { maxRequests: 2, windowMs: 1000 };

    now.mockReturnValue(1000);
    const first = limiter.consume('api', options);
    const second = limiter.consume('api', options);
    const third = limiter.consume('api', options);

    expect(first.allowed).toBe(true);
    expect(first.remaining).toBe(1);
    expect(second.allowed).toBe(true);
    expect(second.remaining).toBe(0);
    expect(third.allowed).toBe(false);
    expect(third.remaining).toBe(0);
    expect(third.retryAfterMs).toBe(1000);
    expect(third.resetAt).toBe(2000);

    now.mockReturnValue(1500);
    const status = limiter.getStatus('api', options);
    expect(status.allowed).toBe(false);
    expect(status.remaining).toBe(0);
    expect(status.retryAfterMs).toBe(500);

    now.mockReturnValue(2500);
    const afterReset = limiter.consume('api', options);
    expect(afterReset.allowed).toBe(true);
    expect(afterReset.remaining).toBe(1);
    expect(afterReset.resetAt).toBe(3500);
  });

  it('reports retryAfterMs as the time remaining until resetAt when blocked', () => {
    const now = jest.fn();
    const limiter = new ClientRateLimiter(now);
    const options = { maxRequests: 1, windowMs: 5000 };

    now.mockReturnValue(1000);
    expect(limiter.consume('api', options).allowed).toBe(true);

    now.mockReturnValue(2500);
    const blocked = limiter.consume('api', options);
    expect(blocked.allowed).toBe(false);
    expect(blocked.resetAt).toBe(6000);
    expect(blocked.retryAfterMs).toBe(blocked.resetAt - 2500);
    expect(blocked.retryAfterMs).toBe(3500);
  });

  it('starts a fresh window once resetAt has passed', () => {
    const now = jest.fn();
    const limiter = new ClientRateLimiter(now);
    const options = { maxRequests: 2, windowMs: 1000 };

    now.mockReturnValue(1000);
    limiter.consume('api', options);
    limiter.consume('api', options);
    expect(limiter.consume('api', options).allowed).toBe(false);

    now.mockReturnValue(2000);
    const fresh = limiter.consume('api', options);
    expect(fresh.allowed).toBe(true);
    expect(fresh.remaining).toBe(1);
    expect(fresh.resetAt).toBe(3000);

    const second = limiter.consume('api', options);
    expect(second.allowed).toBe(true);
    expect(second.remaining).toBe(0);
    expect(second.resetAt).toBe(3000);

    expect(limiter.consume('api', options).allowed).toBe(false);
  });

  it('does not consume a slot when calling getStatus', () => {
    const now = jest.fn();
    const limiter = new ClientRateLimiter(now);
    const options = { maxRequests: 3, windowMs: 1000 };

    now.mockReturnValue(1000);
    limiter.consume('api', options);

    const firstStatus = limiter.getStatus('api', options);
    const secondStatus = limiter.getStatus('api', options);
    const thirdStatus = limiter.getStatus('api', options);

    expect(firstStatus.remaining).toBe(2);
    expect(secondStatus.remaining).toBe(2);
    expect(thirdStatus.remaining).toBe(2);

    const consumed = limiter.consume('api', options);
    expect(consumed.allowed).toBe(true);
    expect(consumed.remaining).toBe(1);
    expect(limiter.getStatus('api', options).remaining).toBe(1);
  });

  it('dispatches RATE_LIMIT_EVENT with the expected detail shape', () => {
    const now = jest.fn();
    const limiter = new ClientRateLimiter(now);
    const options = { maxRequests: 1, windowMs: 1000 };

    now.mockReturnValue(1000);
    limiter.consume('api', options);
    const blocked = limiter.consume('api', options);
    expect(blocked.allowed).toBe(false);

    const listener = jest.fn();
    window.addEventListener(RATE_LIMIT_EVENT, listener);
    try {
      const error = new RateLimitError(blocked, 'api');
      limiter.notifyRateLimit(error);
    } finally {
      window.removeEventListener(RATE_LIMIT_EVENT, listener);
    }

    expect(listener).toHaveBeenCalledTimes(1);
    const event = listener.mock.calls[0][0] as CustomEvent;
    expect(event).toBeInstanceOf(CustomEvent);
    expect(event.type).toBe(RATE_LIMIT_EVENT);
    expect(event.detail).toEqual({
      message: expect.any(String),
      retryAfterMs: blocked.retryAfterMs,
      resetAt: blocked.resetAt,
      limit: options.maxRequests,
      endpoint: 'api',
    });
  });

  it('reset(key) clears only that key while reset() clears all keys', () => {
    const now = jest.fn();
    const limiter = new ClientRateLimiter(now);
    const options = { maxRequests: 2, windowMs: 1000 };

    now.mockReturnValue(1000);
    limiter.consume('api', options);
    limiter.consume('api', options);
    limiter.consume('other', options);

    expect(limiter.getStatus('api', options).remaining).toBe(0);
    expect(limiter.getStatus('other', options).remaining).toBe(1);

    limiter.reset('api');

    expect(limiter.getStatus('api', options).remaining).toBe(2);
    expect(limiter.getStatus('other', options).remaining).toBe(1);

    limiter.consume('api', options);
    limiter.consume('other', options);
    expect(limiter.getStatus('api', options).remaining).toBe(1);
    expect(limiter.getStatus('other', options).remaining).toBe(0);

    limiter.reset();

    expect(limiter.getStatus('api', options).remaining).toBe(2);
    expect(limiter.getStatus('other', options).remaining).toBe(2);
  });
});

describe('parseRetryAfterHeader', () => {
  it('parses numeric seconds values', () => {
    expect(parseRetryAfterHeader('2', 1000)).toBe(2000);
    expect(parseRetryAfterHeader('0', 1000)).toBe(0);
  });

  it('parses HTTP-date values relative to the supplied clock', () => {
    const date = new Date(6000).toUTCString();
    expect(parseRetryAfterHeader(date, 1000)).toBe(5000);
  });

  it('returns null for empty or invalid values', () => {
    expect(parseRetryAfterHeader(null, 1000)).toBeNull();
    expect(parseRetryAfterHeader('not-a-date', 1000)).toBeNull();
  });
});

describe('resolveRateLimitOptions', () => {
  it('returns the defaults when no options are provided', () => {
    expect(resolveRateLimitOptions()).toEqual(DEFAULT_RATE_LIMIT_OPTIONS);
  });

  it('falls back to defaults for invalid values and floors valid ones', () => {
    expect(resolveRateLimitOptions({ maxRequests: 0, windowMs: -10 })).toEqual(
      DEFAULT_RATE_LIMIT_OPTIONS
    );
    expect(
      resolveRateLimitOptions({ maxRequests: 3.9, windowMs: 4.2 })
    ).toEqual({
      maxRequests: 3,
      windowMs: 4,
    });
  });
});
