import { parseTrustProxy, validateEnvironment } from './environment';

const required = {
  CONTRACT_ID: 'configured-contract',
  JWT_SECRET: 'configured-secret',
  STELLAR_NETWORK: 'testnet',
};

describe('validateEnvironment', () => {
  it('reports the names of missing settings without needing a database connection', () => {
    expect(() => validateEnvironment({ NODE_ENV: 'production' })).toThrow(
      /Invalid server configuration:.*CONTRACT_ID.*JWT_SECRET.*STELLAR_NETWORK.*DB_HOST.*DB_PORT.*DB_USER.*DB_PASSWORD.*DB_NAME/,
    );
  });

  it('accepts either a URL or the full set of individual database settings', () => {
    expect(() =>
      validateEnvironment({
        ...required,
        DATABASE_URL: 'postgres://localhost/nevo',
      }),
    ).not.toThrow();
    expect(() =>
      validateEnvironment({
        ...required,
        DB_HOST: 'localhost',
        DB_PORT: '5432',
        DB_USER: 'nevo',
        DB_PASSWORD: 'configured-password',
        DB_NAME: 'nevo',
      }),
    ).not.toThrow();
  });

  it('rejects blank values and invalid networks', () => {
    expect(() =>
      validateEnvironment({
        ...required,
        CONTRACT_ID: ' ',
        STELLAR_NETWORK: 'invalid',
        DATABASE_URL: 'postgres://localhost/nevo',
      }),
    ).toThrow(/CONTRACT_ID.*STELLAR_NETWORK/);
  });

  it('permits test startup without production configuration', () => {
    expect(validateEnvironment({ NODE_ENV: 'test' })).toEqual({
      NODE_ENV: 'test',
    });
  });
});

describe('parseTrustProxy', () => {
  it('accepts named networks, addresses, CIDRs, and a hop count', () => {
    expect(parseTrustProxy('loopback, 10.0.0.0/24, 2001:db8::1')).toEqual([
      'loopback',
      '10.0.0.0/24',
      '2001:db8::1',
    ]);
    expect(parseTrustProxy('2')).toBe(2);
    expect(parseTrustProxy(undefined)).toBe(false);
  });

  it('rejects invalid and unrestricted settings', () => {
    expect(() => parseTrustProxy('10.0.0.0/33')).toThrow(/TRUST_PROXY/);
    expect(() => parseTrustProxy('true')).toThrow(/TRUST_PROXY/);
  });
});
