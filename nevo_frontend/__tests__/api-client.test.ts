import {
  ApiClient,
  RateLimitError,
  verifyAuthSignature,
  fetchAuthChallenge,
  ApiError,
  UnauthorizedError,
  isRateLimitError,
  donate,
  createPool,
  getCurrentUser,
  getTransactions,
  getPoolComments,
  createPoolComment,
  updatePoolComment,
  deletePoolComment,
} from '@/lib/api-client';
import * as rateLimit from '@/lib/rate-limit';
import { RATE_LIMIT_EVENT, type RateLimitNotification } from '@/lib/rate-limit';

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

  it('throws ApiError when the response is not ok', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse(
        { error: 'Bad Request' },
        { status: 400, statusText: 'Bad Request' }
      )
    );

    await expect(
      verifyAuthSignature('publicKey', 'nonce', 'signature')
    ).rejects.toBeInstanceOf(ApiError);
    expect(global.fetch).toHaveBeenCalledTimes(1);
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

describe('fetchAuthChallenge', () => {
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

  it('calls GET /auth/challenge with publicKey query param and returns nonce and expiresAt', async () => {
    const mockChallenge = { nonce: 'abc123', expiresAt: 1700000000 };
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse(mockChallenge));

    const result = await fetchAuthChallenge('GABC123PUBLIC');

    expect(result).toEqual(mockChallenge);
    expect(global.fetch).toHaveBeenCalledTimes(1);

    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/auth/challenge');
    expect(url).toContain('publicKey=GABC123PUBLIC');
    expect(fetchInit.method).toBe('GET');
  });

  it('throws ApiError when the response is not ok', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse(
        { error: 'Bad Request' },
        { status: 400, statusText: 'Bad Request' }
      )
    );

    await expect(fetchAuthChallenge('INVALID')).rejects.toBeInstanceOf(
      ApiError
    );
    expect(global.fetch).toHaveBeenCalledTimes(1);
  });
});

describe('ApiClient route contracts', () => {
  beforeEach(() => {
    Object.defineProperty(globalThis, 'Headers', {
      value: TestHeaders,
      configurable: true,
    });
    global.fetch = jest.fn();
    window.localStorage.clear();
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  it('donate() posts to /donations with the generated body shape', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse({ id: 'donation-1' })
    );

    await donate({
      poolId: 'pool-1',
      amount: '100',
      asset: 'XLM',
    });

    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/donations');
    expect(fetchInit.method).toBe('POST');
    expect(JSON.parse(fetchInit.body)).toEqual({
      poolId: 'pool-1',
      amount: '100',
      asset: 'XLM',
    });
  });

  it('createPool() posts to /pools with the generated body shape', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse({ id: 'pool-1' })
    );

    await createPool({
      title: 'New pool',
      description: 'A pool',
      targetAmount: '1000',
      asset: 'XLM',
    });

    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/pools');
    expect(fetchInit.method).toBe('POST');
    expect(JSON.parse(fetchInit.body)).toEqual({
      title: 'New pool',
      description: 'A pool',
      targetAmount: '1000',
      asset: 'XLM',
    });
  });

  it('getCurrentUser() calls GET /users/me', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse({ id: 'user-1' })
    );

    const result = await getCurrentUser();

    expect(result).toEqual({ id: 'user-1' });
    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/users/me');
    expect(fetchInit.method).toBe('GET');
  });

  it('getTransactions() calls GET /transactions', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse([]));

    const result = await getTransactions();

    expect(result).toEqual([]);
    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/transactions');
    expect(fetchInit.method).toBe('GET');
  });
});

describe('ApiClient pool comments', () => {
  beforeEach(() => {
    Object.defineProperty(globalThis, 'Headers', {
      value: TestHeaders,
      configurable: true,
    });
    global.fetch = jest.fn();
    window.localStorage.clear();
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  it('getPoolComments() calls GET /pools/:id/comments', async () => {
    const comments = [{ id: 'c1', body: 'hello' }];
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse(comments));

    const result = await getPoolComments('pool-1');

    expect(result).toEqual(comments);
    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/pools/pool-1/comments');
    expect(fetchInit.method).toBe('GET');
  });

  it('createPoolComment() posts to /pools/:id/comments', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse({ id: 'c1', body: 'hello' })
    );

    const result = await createPoolComment('pool-1', { body: 'hello' });

    expect(result).toEqual({ id: 'c1', body: 'hello' });
    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/pools/pool-1/comments');
    expect(fetchInit.method).toBe('POST');
    expect(JSON.parse(fetchInit.body)).toEqual({ body: 'hello' });
  });

  it('updatePoolComment() patches /pools/:id/comments/:commentId', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse({ id: 'c1', body: 'edited' })
    );

    const result = await updatePoolComment('pool-1', 'c1', { body: 'edited' });

    expect(result).toEqual({ id: 'c1', body: 'edited' });
    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/pools/pool-1/comments/c1');
    expect(fetchInit.method).toBe('PATCH');
    expect(JSON.parse(fetchInit.body)).toEqual({ body: 'edited' });
  });

  it('deletePoolComment() deletes /pools/:id/comments/:commentId', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse({}, { status: 204, statusText: 'No Content' })
    );

    await deletePoolComment('pool-1', 'c1');

    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/pools/pool-1/comments/c1');
    expect(fetchInit.method).toBe('DELETE');
  });

  it('createPoolComment() requires authentication', async () => {
    await expect(
      createPoolComment('pool-1', { body: 'hello' })
    ).rejects.toBeInstanceOf(UnauthorizedError);
    expect(global.fetch).not.toHaveBeenCalled();
  });
  it('parses the paginated response from fetchMyDonations', async () => {
    window.localStorage.setItem(
      'nevo-wallet',
      JSON.stringify({ state: { accessToken: 'jwt-token-123' } })
    );

    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 10,
      windowMs: 60_000,
    });
    const payload = {
      data: [{ id: 'donation-1', amount: '10' }],
      total: 42,
      page: 2,
      limit: 10,
    };
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse(payload));

    const result = await client.fetchMyDonations({ page: 2, limit: 10 });

    expect(result).toEqual(payload);
    expect(result.data).toHaveLength(1);
    expect(result.total).toBe(42);
    expect(result.page).toBe(2);
    expect(result.limit).toBe(10);

    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/users/me/donations');
    expect(url).toContain('page=2');
    expect(url).toContain('limit=10');
    expect(fetchInit.method).toBe('GET');
  });

  it('fetches pool comments with pagination params', async () => {
    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 10,
      windowMs: 60_000,
    });
    const payload = {
      data: [{ id: 'comment-1', text: 'hi', parentId: null }],
      total: 1,
      page: 1,
      limit: 20,
    };
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse(payload));

    const result = await client.getPoolComments('pool-1', {
      page: 1,
      limit: 20,
    });

    expect(result).toEqual(payload);

    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/pools/pool-1/comments');
    expect(url).toContain('page=1');
    expect(url).toContain('limit=20');
    expect(fetchInit.method).toBe('GET');
  });

  it('posts a pool comment to the comments endpoint', async () => {
    window.localStorage.setItem(
      'nevo-wallet',
      JSON.stringify({ state: { accessToken: 'jwt-token-123' } })
    );

    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 10,
      windowMs: 60_000,
    });
    const created = { id: 'comment-1', text: 'hello', parentId: null };
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse(created));

    const result = await client.createPoolComment('pool-1', {
      text: 'hello',
    });

    expect(result).toEqual(created);

    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/pools/pool-1/comments');
    expect(fetchInit.method).toBe('POST');
    expect(JSON.parse(fetchInit.body)).toEqual({ text: 'hello' });
    expect(fetchInit.headers.get('Authorization')).toBe(
      'Bearer jwt-token-123'
    );
  });

  it('updates a pool comment with PATCH', async () => {
    window.localStorage.setItem(
      'nevo-wallet',
      JSON.stringify({ state: { accessToken: 'jwt-token-123' } })
    );

    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 10,
      windowMs: 60_000,
    });
    const updated = { id: 'comment-1', text: 'edited', parentId: null };
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse(updated));

    const result = await client.updatePoolComment('pool-1', 'comment-1', {
      text: 'edited',
    });

    expect(result).toEqual(updated);

    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/pools/pool-1/comments/comment-1');
    expect(fetchInit.method).toBe('PATCH');
    expect(JSON.parse(fetchInit.body)).toEqual({ text: 'edited' });
  });

  it('deletes a pool comment with DELETE', async () => {
    window.localStorage.setItem(
      'nevo-wallet',
      JSON.stringify({ state: { accessToken: 'jwt-token-123' } })
    );

    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 10,
      windowMs: 60_000,
    });
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse({ ok: true }));

    await client.deletePoolComment('pool-1', 'comment-1');

    const [url, fetchInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toContain('/pools/pool-1/comments/comment-1');
    expect(fetchInit.method).toBe('DELETE');
  });
});

describe('ApiClient RateLimitError round-trip', () => {
  let events: CustomEvent<RateLimitNotification>[];
  const onRateLimit = (event: Event) =>
    events.push(event as CustomEvent<RateLimitNotification>);

  beforeEach(() => {
    Object.defineProperty(globalThis, 'Headers', {
      value: TestHeaders,
      configurable: true,
    });
    global.fetch = jest.fn();
    events = [];
    window.addEventListener(RATE_LIMIT_EVENT, onRateLimit);
    jest.spyOn(rateLimit, 'notifyRateLimit');
  });

  afterEach(() => {
    window.removeEventListener(RATE_LIMIT_EVENT, onRateLimit);
    jest.restoreAllMocks();
  });

  async function captureError(promise: Promise<unknown>) {
    try {
      await promise;
    } catch (error) {
      return error;
    }
    throw new Error('Expected request to reject');
  }

  it('throws a RateLimitError and notifies when the local limit is hit', async () => {
    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 1,
      windowMs: 60_000,
    });
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse({ ok: true }));

    await client.get('/pools', { requireAuth: false, cacheResponse: false });
    const error = await captureError(
      client.get('/pools', { requireAuth: false, cacheResponse: false })
    );

    expect(error).toBeInstanceOf(RateLimitError);
    expect(isRateLimitError(error)).toBe(true);
    expect(rateLimit.notifyRateLimit).toHaveBeenCalledTimes(1);
    expect(rateLimit.notifyRateLimit).toHaveBeenCalledWith(error);
    expect(events).toHaveLength(1);
    expect(events[0].detail).toMatchObject({
      message: (error as RateLimitError).message,
      limit: 1,
      endpoint: '/pools',
    });
  });

  it('throws a RateLimitError and notifies on a server 429', async () => {
    const client = new ApiClient('https://api.test', 1000, {
      maxRequests: 10,
      windowMs: 60_000,
    });
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse(
        { error: 'Too many requests' },
        { status: 429, headers: { 'Retry-After': '5' } }
      )
    );

    const error = await captureError(
      client.get('/limited', { requireAuth: false, cacheResponse: false })
    );

    expect(error).toBeInstanceOf(RateLimitError);
    expect(isRateLimitError(error)).toBe(true);
    expect(rateLimit.notifyRateLimit).toHaveBeenCalledTimes(1);
    expect(rateLimit.notifyRateLimit).toHaveBeenCalledWith(error);
    expect(events).toHaveLength(1);
    expect(events[0].detail).toMatchObject({
      retryAfterMs: 5_000,
      endpoint: '/limited',
    });
  });
});
