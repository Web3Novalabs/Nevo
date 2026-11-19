import {
  ClientRateLimiter,
  RateLimitError,
  notifyRateLimit,
  parseRetryAfterHeader,
  resolveRateLimitOptions,
} from './rate-limit';
import { signTransaction as freighterSignTransaction } from '@stellar/freighter-api';
import { env } from './env';

export interface AccountBalances {
  xlm: string;
  usdc: string;
}

/** USDC issuer per Stellar network. */
export const USDC_ISSUER: Record<string, string> = {
  testnet: 'GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN',
  mainnet: 'GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN',
};

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
  const endpoint = `${env.horizonUrl}/accounts/${publicKey}`;

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
    const usdcIssuer = USDC_ISSUER[env.network];
    let xlm = '0';
    let usdc = '0';
    for (const b of data.balances ?? []) {
      if (b.asset_type === 'native') xlm = parseFloat(b.balance).toFixed(2);
      if (
        b.asset_code === 'USDC' &&
        b.asset_issuer === usdcIssuer
      ) {
        usdc = parseFloat(b.balance).toFixed(2);
      }
    }
    return { xlm, usdc };
  } catch {
    return ZERO_BALANCES;
  }
}

export async function signTransaction(xdr: string): Promise<string> {
  const result = await freighterSignTransaction(xdr, {
    networkPassphrase: env.networkPassphrase,
  });

  if (result.error) {
    throw new Error(result.error);
  }

  return result.signedTxXdr;
}
