import { createPool, CreatePoolParams } from '@/lib/api-client';

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

const TEST_PUBKEY = 'GAVQGTVZ25JJSLUC72LD5T73CKE4EHUEKOKY7CKUF4GSESY5V726ISXJ';

describe('createPool', () => {
  const fullParams: CreatePoolParams = {
    contractPoolId: '1',
    creatorWallet: TEST_PUBKEY,
    goal: '10000000',
    title: 'Test Pool',
    description: 'A test pool for unit testing',
    category: 'Technology',
    imageUrl: 'https://example.com/banner.jpg',
  };

  const minimalParams: CreatePoolParams = {
    contractPoolId: '2',
    creatorWallet: TEST_PUBKEY,
    goal: '5000000',
  };

  beforeEach(() => {
    global.fetch = jest.fn();
    window.localStorage.clear();
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  it('calls POST /pools with all fields and returns the created pool', async () => {
    const expectedResponse = {
      id: '550e8400-e29b-41d4-a716-446655440000',
      ...fullParams,
      status: 'Active',
      raised: '0',
    };

    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse(expectedResponse)
    );

    const result = await createPool(fullParams);

    // Verify fetch was called once
    expect(global.fetch).toHaveBeenCalledTimes(1);

    // Verify request details
    const [url, requestInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toMatch(/\/pools$/);
    expect(requestInit.method).toBe('POST');
    expect(JSON.parse(requestInit.body)).toEqual(fullParams);

    // Verify Content-Type header (apiClient sets it via Headers)
    const headers = new Headers(requestInit.headers);
    expect(headers.get('Content-Type')).toBe('application/json');

    // Verify response
    expect(result).toEqual(expectedResponse);
  });

  it('works with only required fields (no optionals)', async () => {
    const response = { id: 'uuid-abc', ...minimalParams, status: 'Active' };
    (global.fetch as jest.Mock).mockResolvedValue(jsonResponse(response));

    const result = await createPool(minimalParams);

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [, requestInit] = (global.fetch as jest.Mock).mock.calls[0];
    expect(JSON.parse(requestInit.body)).toEqual(minimalParams);
    expect(result).toEqual(response);
  });

  it('throws an error when the API responds with 4xx', async () => {
    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse(
        { error: 'Validation failed', message: 'Title is required' },
        { status: 422, statusText: 'Unprocessable Entity' }
      )
    );

    await expect(createPool(fullParams)).rejects.toMatchObject({
      status: 422,
    });
  });

  it('throws a network error when fetch fails', async () => {
    (global.fetch as jest.Mock).mockRejectedValue(new Error('Network failure'));

    await expect(createPool(fullParams)).rejects.toThrow('Network failure');
  }, 10000);

  it('throws a rate limit error when server responds with 429', async () => {
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

    await expect(createPool(fullParams)).rejects.toMatchObject({
      name: 'RateLimitError',
      status: 429,
    });
  });

  it('includes the XLM goal in stroops format', async () => {
    const paramsWithStroops: CreatePoolParams = {
      contractPoolId: '3',
      creatorWallet: TEST_PUBKEY,
      goal: '50000000', // 5 XLM in stroops
      title: 'Stroop Test',
    };

    (global.fetch as jest.Mock).mockResolvedValue(
      jsonResponse({ id: 'uuid-456', ...paramsWithStroops })
    );

    await createPool(paramsWithStroops);

    const [, requestInit] = (global.fetch as jest.Mock).mock.calls[0];
    const body = JSON.parse(requestInit.body);
    expect(body.goal).toBe('50000000');
  });
});
