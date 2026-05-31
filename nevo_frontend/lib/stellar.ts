import {
  ClientRateLimiter,
  RateLimitError,
  notifyRateLimit,
  parseRetryAfterHeader,
  resolveRateLimitOptions,
} from './rate-limit';

export interface AccountBalances {
  xlm: string;
  usdc: string;
}

const HORIZON = 'https://horizon.stellar.org';
const ZERO_BALANCES: AccountBalances = { xlm: '0', usdc: '0' };
const HORIZON_RATE_LIMIT = resolveRateLimitOptions({
  maxRequests: 60,
  windowMs: 60_000,
});
const horizonRateLimiter = new ClientRateLimiter();

function createHorizonRateLimitError(
  retryAfterMs: number,
  endpoint: string
): RateLimitError {
  return new RateLimitError(
    {
      ...HORIZON_RATE_LIMIT,
      allowed: false,
      remaining: 0,
      resetAt: Date.now() + retryAfterMs,
      retryAfterMs,
    },
    endpoint
  );
}

function enforceHorizonRateLimit(endpoint: string): void {
  const result = horizonRateLimiter.consume(
    'stellar:horizon',
    HORIZON_RATE_LIMIT
  );
  if (!result.allowed) {
    const error = new RateLimitError(result, endpoint);
    notifyRateLimit(error);
    throw error;
  }
}

/** Fetches XLM and USDC balances for a Stellar public key via Horizon. */
export async function getAccountBalances(
  publicKey: string
): Promise<AccountBalances> {
  const endpoint = `${HORIZON}/accounts/${publicKey}`;

  try {
    enforceHorizonRateLimit(endpoint);

    const res = await fetch(endpoint);
    if (res.status === 429) {
      const retryAfterMs =
        parseRetryAfterHeader(res.headers.get('Retry-After')) ??
        HORIZON_RATE_LIMIT.windowMs;
      notifyRateLimit(createHorizonRateLimitError(retryAfterMs, endpoint));
      return ZERO_BALANCES;
    }

    if (!res.ok) return ZERO_BALANCES;
    const data = await res.json();
    let xlm = '0';
    let usdc = '0';
    for (const b of data.balances ?? []) {
      if (b.asset_type === 'native') xlm = parseFloat(b.balance).toFixed(2);
      if (b.asset_code === 'USDC') usdc = parseFloat(b.balance).toFixed(2);
    }
    return { xlm, usdc };
  } catch {
    return ZERO_BALANCES;
  }
}
