import type { Breadcrumb, ErrorEvent } from '@sentry/nextjs';
import { beforeBreadcrumb, beforeSend } from '@/lib/sentry-scrubber';

const JWT =
  'eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dGhpcy1pcy1hLXNpZ25hdHVyZQ';
const SIGNATURE = 'a'.repeat(128);
const PUBLIC_KEY = `G${'A'.repeat(55)}`;
const CONTRACT_ID = `C${'B'.repeat(55)}`;

function asRecord(value: unknown): Record<string, unknown> {
  return value as Record<string, unknown>;
}

describe('Sentry scrubber', () => {
  const originalNetwork = process.env.NEXT_PUBLIC_STELLAR_NETWORK;
  const originalContractId = process.env.NEXT_PUBLIC_CONTRACT_ID;

  beforeEach(() => {
    process.env.NEXT_PUBLIC_STELLAR_NETWORK = 'testnet';
    process.env.NEXT_PUBLIC_CONTRACT_ID = CONTRACT_ID;
  });

  afterEach(() => {
    if (originalNetwork === undefined) {
      delete process.env.NEXT_PUBLIC_STELLAR_NETWORK;
    } else {
      process.env.NEXT_PUBLIC_STELLAR_NETWORK = originalNetwork;
    }

    if (originalContractId === undefined) {
      delete process.env.NEXT_PUBLIC_CONTRACT_ID;
    } else {
      process.env.NEXT_PUBLIC_CONTRACT_ID = originalContractId;
    }

    delete (window as Window & { freighter?: unknown }).freighter;
  });

  it('scrubs auth headers, JWTs, signatures and public keys from events', () => {
    const event = {
      message: `failed with ${JWT} and ${SIGNATURE} for ${PUBLIC_KEY}`,
      request: {
        url: `https://horizon.example/accounts/${PUBLIC_KEY}?signature=${SIGNATURE}`,
        headers: {
          Authorization: `Bearer ${JWT}`,
          'X-Wallet-Pubkey': PUBLIC_KEY,
        },
        data: {
          accessToken: JWT,
          signature: SIGNATURE,
          publicKey: PUBLIC_KEY,
          safeValue: 'keep me',
        },
      },
      tags: { existing: 'value' },
    } as unknown as ErrorEvent;

    const scrubbed = beforeSend(event);
    const serialized = JSON.stringify(scrubbed);

    expect(serialized).not.toContain(JWT);
    expect(serialized).not.toContain(SIGNATURE);
    expect(serialized).not.toContain(PUBLIC_KEY);
    expect(asRecord(asRecord(scrubbed.request).headers).Authorization).toBe(
      '[REDACTED]'
    );
    expect(asRecord(asRecord(scrubbed.request).data).safeValue).toBe('keep me');
    expect(scrubbed.tags).toEqual({
      existing: 'value',
      network: 'testnet',
      'wallet.type': 'none',
      'contract.id': CONTRACT_ID,
    });
  });

  it('scrubs the wallet type tag without adding wallet secrets', () => {
    (window as Window & { freighter?: unknown }).freighter = {};

    const scrubbed = beforeSend({
      message: 'wallet error',
    } as unknown as ErrorEvent);

    expect(scrubbed.tags?.['wallet.type']).toBe('freighter');
    expect(JSON.stringify(scrubbed)).not.toContain('accessToken');
  });

  it('scrubs request breadcrumbs before they reach Sentry', () => {
    const breadcrumb = {
      category: 'fetch',
      message: 'POST /auth/verify',
      data: {
        headers: { authorization: `Bearer ${JWT}` },
        url: `https://api.example/auth/verify?signature=${SIGNATURE}`,
        signature: SIGNATURE,
        nested: { token: JWT },
      },
    } as unknown as Breadcrumb;

    const scrubbed = beforeBreadcrumb(breadcrumb);
    const serialized = JSON.stringify(scrubbed);

    expect(serialized).not.toContain(JWT);
    expect(serialized).not.toContain(SIGNATURE);
    expect(asRecord(asRecord(scrubbed.data).headers).authorization).toBe(
      '[REDACTED]'
    );
  });

  it('redacts JWTs and signatures found under unknown keys', () => {
    const event = {
      extra: {
        debug: `token=${JWT} signature=${SIGNATURE}`,
      },
    } as unknown as ErrorEvent;

    const serialized = JSON.stringify(beforeSend(event));

    expect(serialized).not.toContain(JWT);
    expect(serialized).not.toContain(SIGNATURE);
  });
});
