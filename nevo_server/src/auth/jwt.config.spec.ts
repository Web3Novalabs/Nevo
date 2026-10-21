import { Logger } from '@nestjs/common';

describe('getJwtSecret()', () => {
  const ORIGINAL_ENV = { ...process.env };

  let warnSpy: jest.SpyInstance;

  beforeEach(() => {
    jest.resetModules();
    warnSpy = jest.spyOn(Logger.prototype, 'warn').mockImplementation(() => undefined);
  });

  afterEach(() => {
    warnSpy.mockRestore();
    process.env = { ...ORIGINAL_ENV };
  });

  const loadModule = async () => {
    return await import('./jwt.config');
  };

  it('returns JWT_SECRET_FALLBACK when process.env.JWT_SECRET is unset', async () => {
    delete process.env.JWT_SECRET;

    const { getJwtSecret, JWT_SECRET_FALLBACK } = await loadModule();

    expect(getJwtSecret()).toBe(JWT_SECRET_FALLBACK);
    expect(JWT_SECRET_FALLBACK).toBe('dev-secret');
  });

  it('logs a warning mentioning JWT_SECRET and the insecure default when falling back', async () => {
    delete process.env.JWT_SECRET;

    const { getJwtSecret } = await loadModule();

    getJwtSecret();

    expect(warnSpy).toHaveBeenCalledTimes(1);
    const message = String(warnSpy.mock.calls[0][0]);
    expect(message).toMatch(/JWT_SECRET/);
    expect(message).toMatch(/insecure|default|fallback/i);
  });

  it('returns the exact env value verbatim and logs no warning when JWT_SECRET is set', async () => {
    const secret = 'super-secret-value-123';
    process.env.JWT_SECRET = secret;

    const { getJwtSecret } = await loadModule();

    expect(getJwtSecret()).toBe(secret);
    expect(warnSpy).not.toHaveBeenCalled();
  });

  // The fallback is only acceptable in development. This test captures the
  // current behavior for NODE_ENV=production: the fallback is still returned.
  // NOTE: This should be disabled outside development in the future. When a
  // NODE_ENV check is added to getJwtSecret(), this expectation should be
  // updated to assert that it throws (or otherwise refuses to fall back)
  // outside development.
  it('documents current fallback behavior for NODE_ENV=production (should be disabled outside development)', async () => {
    delete process.env.JWT_SECRET;
    process.env.NODE_ENV = 'production';

    const { getJwtSecret, JWT_SECRET_FALLBACK } = await loadModule();

    // Current behavior: fallback is still returned regardless of NODE_ENV.
    // This is flagged as a security concern that should be addressed.
    expect(getJwtSecret()).toBe(JWT_SECRET_FALLBACK);
  });
});