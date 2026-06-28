import {
  ApiClient,
  RateLimitError,
  verifyAuthSignature,
  UnauthorizedError,
} from '@/lib/api-client';

function jsonResponse(data: unknown, init: ResponseInit = {}) {
  const status = init.status ?? 200;
  return {
    ok: status >= 200 && status < 300,
    status,
    statusText: init.statusText ?? '',
    headers: new Headers(
      init.headers ?? { 'Content-Type': 'application/json' }
    ),
    json: jest.fn().mockResolvedValue(data),
    text: jest.fn().mockResolvedValue(JSON.stringify(data)),
  } as unknown as Response;
}

class TestHeaders {
  private values = new Map<string, string>();

  constructor(init?: HeadersInit) {
    if (!init) return;

    if (init instanceof TestHeaders) {
      init.values.forEach((value, key) => this.values.set(key, value));
      return;
    }

    if (Array.isArray(init)) {
      init.forEach(([key, value]) => this.set(key, value));
      return;
    }

    Object.entries(init as Record<string, string>).forEach(([key, value]) =>
      this.set(key, value)
    );
  }

  get(name: string) {
    return this.values.get(name.toLowerCase()) ?? null;
  }

  has(name: string) {
    return this.values.has(name.toLowerCase());
  }

  set(name: string, value: string) {
    this.values.set(name.toLowerCase(), value);
  }
}

describe('ApiClient rate limiting', () => {
  beforeEach(() => {
    Object.defineProperty(globalThis, 'Headers', {
      value: TestHeaders,
      configurable: true,
    });
    jest.useFakeTimers();
    jest.setSystemTime(new Date('2026-01-01T00:00:00Z'));
    global.fetch = jest.fn();
    window.localStorage.clear();
  });

  afterEach(() => {
    jest.useRealTimers();
    jest.restoreAllMocks();
  });

  it('blocks requests once the client-side window is exhausted', async () => {
    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 1,
      windowMs: 60_000,
    });
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse({ ok: true }));

    await client.post('/pools', { title: 'One' }, { requireAuth: false });

    await expect(
      client.post('/pools', { title: 'Two' }, { requireAuth: false })
    ).rejects.toBeInstanceOf(RateLimitError);
    expect(global.fetch).toHaveBeenCalledTimes(1);
  });

  it('allows requests again after the tracked window resets', async () => {
    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 1,
      windowMs: 1_000,
    });
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse({ ok: true }));

    await client.get('/pools', { requireAuth: false, cacheResponse: false });
    await expect(
      client.get('/pools', { requireAuth: false, cacheResponse: false })
    ).rejects.toBeInstanceOf(RateLimitError);

    jest.advanceTimersByTime(1_000);

    await client.get('/pools', { requireAuth: false, cacheResponse: false });
    expect(global.fetch).toHaveBeenCalledTimes(2);
  });

  it('serves cached GET responses without spending another request', async () => {
    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 1,
      windowMs: 60_000,
    });
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse({ id: 'cached-pool' })
    );

    const first = await client.get('/pools/1', { requireAuth: false });
    const second = await client.get('/pools/1', { requireAuth: false });

    expect(first).toEqual({ id: 'cached-pool' });
    expect(second).toEqual(first);
    expect(global.fetch).toHaveBeenCalledTimes(1);
  });

  it('does not include Authorization header when no JWT is stored', async () => {
    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 10,
      windowMs: 60_000,
    });
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse({ ok: true }));

    await client.get('/public', {
      requireAuth: undefined,
      cacheResponse: false,
    });

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(fetchInit.headers.get('Authorization')).toBeNull();
  });

  it('attaches Authorization header when JWT is stored', async () => {
    window.localStorage.setItem(
      'nevo-wallet',
      JSON.stringify({ state: { accessToken: 'jwt-token-123' } })
    );

    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 10,
      windowMs: 60_000,
    });
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse({ ok: true }));

    await client.post('/protected', { foo: 'bar' }, { cacheResponse: false });

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(fetchInit.headers.get('Authorization')).toBe('Bearer jwt-token-123');
  });

  it('omits Authorization header when requireAuth is false even if JWT is stored', async () => {
    window.localStorage.setItem(
      'nevo-wallet',
      JSON.stringify({ state: { accessToken: 'jwt-token-123' } })
    );

    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 10,
      windowMs: 60_000,
    });
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse({ ok: true }));

    await client.get('/public', { requireAuth: false, cacheResponse: false });

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(fetchInit.headers.get('Authorization')).toBeNull();
  });

  it('converts server 429 responses into retry-aware rate limit errors', async () => {
    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 10,
      windowMs: 60_000,
    });
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse(
        { error: 'Too many requests' },
        {
          status: 429,
          statusText: 'Too Many Requests',
          headers: { 'Retry-After': '5' },
        }
      )
    );

    await expect(
      client.get('/limited', { requireAuth: false, cacheResponse: false })
    ).rejects.toMatchObject({
      name: 'RateLimitError',
      retryAfterMs: 5_000,
      status: 429,
    });
    expect(global.fetch).toHaveBeenCalledTimes(1);
  });
});

describe('verifyAuthSignature', () => {
  beforeEach(() => {
    Object.defineProperty(globalThis, 'Headers', {
      value: TestHeaders,
      configurable: true,
    });
    global.fetch = jest.fn();
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  it('calls POST /auth/verify with JSON body and returns accessToken', async () => {
    const mockTokenResponse = { accessToken: 'mock-access-token' };
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse(mockTokenResponse)
    );

    const result = await verifyAuthSignature('publicKey', 'nonce', 'signature');

    expect(result).toEqual(mockTokenResponse);
    expect(global.fetch).toHaveBeenCalledTimes(1);

    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/auth/verify');
    expect(fetchInit.method).toBe('POST');
    expect(JSON.parse(fetchInit.body)).toEqual({
      publicKey: 'publicKey',
      signature: 'signature',
      message: 'nonce',
    });
  });

  it('throws UnauthorizedError when API returns 401', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse(
        { error: 'Unauthorized' },
        {
          status: 401,
          statusText: 'Unauthorized',
        }
      )
    );

    await expect(
      verifyAuthSignature('publicKey', 'nonce', 'signature')
    ).rejects.toBeInstanceOf(UnauthorizedError);

    expect(global.fetch).toHaveBeenCalledTimes(1);
  });
});
