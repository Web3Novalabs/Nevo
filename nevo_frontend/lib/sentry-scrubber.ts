import type { Breadcrumb, ErrorEvent } from '@sentry/nextjs';

const REDACTED = '[REDACTED]';

const SENSITIVE_KEY_PATTERN =
  /authorization|auth|token|jwt|secret|password|signature|signedxdr|xdr|nonce/i;
const BEARER_PATTERN = /\b(bearer\s+)[^\s,;'"]+/gi;
const JWT_PATTERN = /\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/g;
const SIGNATURE_PATTERN = /\b[0-9a-fA-F]{128}\b/g;
const PUBLIC_KEY_PATTERN = /\bG[A-Z2-7]{55}\b/g;
const SENSITIVE_QUERY_PATTERN =
  /([?&][^=&]*?(?:signature|sig|token|jwt|authorization)[^=&]*=)([^&#]+)/gi;

function scrubString(value: string): string {
  return value
    .replace(BEARER_PATTERN, `$1${REDACTED}`)
    .replace(JWT_PATTERN, REDACTED)
    .replace(SENSITIVE_QUERY_PATTERN, `$1${REDACTED}`)
    .replace(SIGNATURE_PATTERN, REDACTED)
    .replace(PUBLIC_KEY_PATTERN, REDACTED);
}

function scrubValue(value: unknown, seen: WeakSet<object>): unknown {
  if (typeof value === 'string') {
    return scrubString(value);
  }

  if (Array.isArray(value)) {
    return value.map((item) => scrubValue(item, seen));
  }

  if (value && typeof value === 'object') {
    if (seen.has(value)) {
      return REDACTED;
    }
    seen.add(value);

    return Object.fromEntries(
      Object.entries(value).map(([key, entry]) => [
        key,
        SENSITIVE_KEY_PATTERN.test(key) ? REDACTED : scrubValue(entry, seen),
      ])
    );
  }

  return value;
}

function getWalletType(): string {
  if (typeof window !== 'undefined' && 'freighter' in window) {
    return 'freighter';
  }
  return 'none';
}

export function scrubEvent(event: ErrorEvent): ErrorEvent {
  return scrubValue(event, new WeakSet<object>()) as ErrorEvent;
}

export function scrubBreadcrumb(breadcrumb: Breadcrumb): Breadcrumb {
  return scrubValue(breadcrumb, new WeakSet<object>()) as Breadcrumb;
}

export function beforeSend(event: ErrorEvent): ErrorEvent {
  const scrubbed = scrubEvent(event);

  return {
    ...scrubbed,
    tags: {
      ...scrubbed.tags,
      network: process.env.NEXT_PUBLIC_STELLAR_NETWORK ?? 'testnet',
      'wallet.type': getWalletType(),
      'contract.id': process.env.NEXT_PUBLIC_CONTRACT_ID ?? 'unset',
    },
  };
}

export function beforeBreadcrumb(breadcrumb: Breadcrumb): Breadcrumb {
  return scrubBreadcrumb(breadcrumb);
}
