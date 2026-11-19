describe('public env configuration', () => {
  const originalEnv = process.env;

  beforeEach(() => {
    jest.resetModules();
    process.env = { ...originalEnv };
    delete process.env.NEXT_PUBLIC_API_BASE_URL;
    delete process.env.NEXT_PUBLIC_STELLAR_NETWORK;
    delete process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE;
    delete process.env.NEXT_PUBLIC_HORIZON_URL;
  });

  afterAll(() => {
    process.env = originalEnv;
  });

  describe('env object', () => {
    it('falls back to default values when public env vars are missing', async () => {
      const { env } = await import('../lib/env');

      expect(env.NEXT_PUBLIC_API_BASE_URL).toBe('http://localhost:3000');
      expect(env.NEXT_PUBLIC_STELLAR_NETWORK).toBe('testnet');
    });

    it('uses process.env values when they are set', async () => {
      process.env.NEXT_PUBLIC_API_BASE_URL = 'https://api.example.com';
      process.env.NEXT_PUBLIC_STELLAR_NETWORK = 'public';

      const { env } = await import('../lib/env');

      expect(env.NEXT_PUBLIC_API_BASE_URL).toBe('https://api.example.com');
      expect(env.NEXT_PUBLIC_STELLAR_NETWORK).toBe('public');
    });

    it('treats an empty NEXT_PUBLIC_NETWORK_PASSPHRASE as unset and falls back to testnet', async () => {
      process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE = '';

      const { env } = await import('../lib/env');

      expect(env.NEXT_PUBLIC_STELLAR_NETWORK).toBe('testnet');
      expect(env.NEXT_PUBLIC_NETWORK_PASSPHRASE).toBe(Networks.TESTNET);
    });

    it('uses the public passphrase when NEXT_PUBLIC_STELLAR_NETWORK is public', async () => {
      process.env.NEXT_PUBLIC_STELLAR_NETWORK = 'public';
      process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE = '';

      const { env } = await import('../lib/env');

      expect(env.NEXT_PUBLIC_NETWORK_PASSPHRASE).toBe(Networks.PUBLIC);
    });

    it('overrides a passphrase that mismatches NEXT_PUBLIC_STELLAR_NETWORK', async () => {
      process.env.NEXT_PUBLIC_STELLAR_NETWORK = 'testnet';
      process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE = Networks.PUBLIC;

      const { env } = await import('../lib/env');

      expect(env.NEXT_PUBLIC_NETWORK_PASSPHRASE).toBe(Networks.TESTNET);
    });

    it('derives the testnet Horizon URL by default', async () => {
      const { env } = await import('../lib/env');

      expect(env.NEXT_PUBLIC_HORIZON_URL).toBe('https://horizon-testnet.stellar.org');
    });

    it('derives the public Horizon URL for the public network', async () => {
      process.env.NEXT_PUBLIC_STELLAR_NETWORK = 'public';

      const { env } = await import('../lib/env');

      expect(env.NEXT_PUBLIC_HORIZON_URL).toBe('https://horizon.stellar.org');
    });

    it('lets NEXT_PUBLIC_HORIZON_URL override the derived URL', async () => {
      process.env.NEXT_PUBLIC_HORIZON_URL = 'https://horizon.example.com';

      const { env } = await import('../lib/env');

      expect(env.NEXT_PUBLIC_HORIZON_URL).toBe('https://horizon.example.com');
    });
  });

  describe('validatePublicEnv', () => {
    it('completes without throwing when all required vars are present', async () => {
      process.env.NEXT_PUBLIC_API_BASE_URL = 'https://api.example.com';
      process.env.NEXT_PUBLIC_STELLAR_NETWORK = 'public';

      const { validatePublicEnv } = await import('../lib/env');

      expect(() => validatePublicEnv()).not.toThrow();
    });

    it('completes without throwing when vars are missing (falls back to defaults)', async () => {
      const { validatePublicEnv } = await import('../lib/env');

      expect(() => validatePublicEnv()).not.toThrow();
    });

    it('completes without throwing when NEXT_PUBLIC_NETWORK_PASSPHRASE is empty', async () => {
      process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE = '';

      const { validatePublicEnv } = await import('../lib/env');

      expect(() => validatePublicEnv()).not.toThrow();
    });

    it('rejects an unknown NEXT_PUBLIC_STELLAR_NETWORK', async () => {
      process.env.NEXT_PUBLIC_STELLAR_NETWORK = 'mainnet';

      const { validatePublicEnv } = await import('../lib/env');

      expect(() => validatePublicEnv()).toThrow();
    });
  });
});
