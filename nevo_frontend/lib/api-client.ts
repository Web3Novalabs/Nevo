import {
  ClientRateLimiter,
  DEFAULT_RATE_LIMIT_OPTIONS,
  RateLimitError,
  type RateLimitOptions,
  isRateLimitError,
  notifyRateLimit,
  parseRetryAfterHeader,
  resolveRateLimitOptions,
} from './rate-limit';
import type {
  ApiPool,
  ApiDonation,
  ApiUser,
  PaginatedResponse,
} from './api-types';
import type { paths } from './api-schema';
import { env, validatePublicEnv } from './env';
import { clearAuth, getToken } from './jwt-storage';
import { toast } from '../components/Toast';

validatePublicEnv();

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';

// ─── Generated OpenAPI helpers ───────────────────────────────────────────────
// These derive request/response shapes directly from the generated schema so
// that calling a route that doesn't exist, or sending the wrong body shape,
// fails typecheck.

export type ApiPath = keyof paths;

type PathItem<Path extends ApiPath> = paths[Path];

type OperationFor<
  Path extends ApiPath,
  Method extends Lowercase<HttpMethod>,
> = PathItem<Path> extends Record<Method, infer Operation> ? Operation : never;

/** JSON request body for a given path + method (never if none declared). */
export type ApiRequestBody<
  Path extends ApiPath,
  Method extends Lowercase<HttpMethod>,
> = OperationFor<Path, Method> extends {
  requestBody: { content: { 'application/json': infer Body } };
}
  ? Body
  : never;

/** JSON response body for a given path + method (never if none declared). */
export type ApiResponseBody<
  Path extends ApiPath,
  Method extends Lowercase<HttpMethod>,
> = OperationFor<Path, Method> extends {
  responses: { 200: { content: { 'application/json': infer Body } } };
}
  ? Body
  : OperationFor<Path, Method> extends {
        responses: { 201: { content: { 'application/json': infer Body } } };
      }
    ? Body
    : never;

/** Query parameters for a given path + method. */
export type ApiQueryParams<
  Path extends ApiPath,
  Method extends Lowercase<HttpMethod>,
> = OperationFor<Path, Method> extends {
  parameters: { query?: infer Query };
}
  ? Query
  : never;

export interface RequestConfig extends Omit<RequestInit, 'method' | 'body'> {
  params?: Record<string, string | number | boolean | undefined>;
  timeout?: number;
  retries?: number;
  retryDelay?: number;
  body?: unknown;
  requireAuth?: boolean;
  rateLimit?: false | Partial<RateLimitOptions>;
  rateLimitKey?: string;
  skipRateLimitNotification?: boolean;
  cacheResponse?: boolean;
  cacheTtlMs?: number;
}

export class ApiError extends Error {
  constructor(
    public status: number,
    public message: string,
    public data?: unknown
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export class UnauthorizedError extends ApiError {
  constructor(message: string, data?: unknown) {
    super(401, message, data);
    this.name = 'UnauthorizedError';
  }
}

type Interceptor<T> = (data: T) => T | Promise<T>;

type PreparedRequestConfig = RequestConfig & {
  url: string;
  method: string;
  headers: Headers;
  body?: BodyInit;
};

interface CacheEntry<T> {
  data: T;
  expiresAt: number;
}

const DEFAULT_CACHE_TTL_MS = 15_000;
const DEFAULT_RATE_LIMIT_KEY = 'api';

export class ApiClient {
  private baseURL: string;
  private defaultTimeout: number;
  private defaultRateLimit: RateLimitOptions;
  private rateLimiter: ClientRateLimiter;
  private responseCache = new Map<string, CacheEntry<unknown>>();
  private activeRequests = 0;
  private loadingListeners: Set<(loading: boolean) => void> = new Set();

  public requestInterceptors: Interceptor<PreparedRequestConfig>[] = [];
  public responseInterceptors: Interceptor<Response>[] = [];

  constructor(
    baseURL: string = '',
    defaultTimeout: number = 10_000,
    rateLimit: Partial<RateLimitOptions> = DEFAULT_RATE_LIMIT_OPTIONS
  ) {
    this.baseURL = baseURL || env.NEXT_PUBLIC_API_BASE_URL;
    this.defaultTimeout = defaultTimeout;
    this.defaultRateLimit = resolveRateLimitOptions(rateLimit);
    this.rateLimiter = new ClientRateLimiter();
    this.addRequestInterceptor((config) =>
      this.attachAuthorizationHeader(config)
    );
  }

  private attachAuthorizationHeader(
    config: PreparedRequestConfig
  ): PreparedRequestConfig {
    if (config.requireAuth === false) {
      return config;
    }

    const headers = new Headers(config.headers);
    const accessToken = getToken();

    if (accessToken) {
      headers.set('Authorization', `Bearer ${accessToken}`);
    } else {
      headers.delete('Authorization');
    }

    return {
      ...config,
      headers,
    };
  }

  private startRequest() {
    this.activeRequests++;
    if (this.activeRequests === 1) {
      this.notifyLoadingListeners(true);
    }
  }

  private endRequest() {
    this.activeRequests = Math.max(0, this.activeRequests - 1);
    if (this.activeRequests === 0) {
      this.notifyLoadingListeners(false);
    }
  }

  private notifyLoadingListeners(isLoading: boolean) {
    this.loadingListeners.forEach((listener) => listener(isLoading));
  }

  /**
   * Subscribe to global loading state changes.
   * Returns an unsubscribe function.
   */
  public subscribeToLoading(
    listener: (isLoading: boolean) => void
  ): () => void {
    this.loadingListeners.add(listener);
    listener(this.activeRequests > 0);
    return () => {
      this.loadingListeners.delete(listener);
    };
  }

  /**
   * Check if there are any active requests.
   */
  public get isLoading(): boolean {
    return this.activeRequests > 0;
  }

  // Interceptors
  addRequestInterceptor(interceptor: Interceptor<PreparedRequestConfig>) {
    this.requestInterceptors.push(interceptor);
  }

  addResponseInterceptor(interceptor: Interceptor<Response>) {
    this.responseInterceptors.push(interceptor);
  }

  private async applyRequestInterceptors(config: PreparedRequestConfig) {
    let currentConfig = { ...config };
    for (const interceptor of this.requestInterceptors) {
      currentConfig = await interceptor(currentConfig);
    }
    return currentConfig;
  }

  private async applyResponseInterceptors(response: Response) {
    let currentResponse = response;
    for (const interceptor of this.responseInterceptors) {
      currentResponse = await interceptor(currentResponse);
    }
    return currentResponse;
  }

  public getRateLimitStatus(
    key: string = DEFAULT_RATE_LIMIT_KEY,
    options: Partial<RateLimitOptions> = {}
  ) {
    return this.rateLimiter.getStatus(
      key,
      resolveRateLimitOptions({ ...this.defaultRateLimit, ...options })
    );
  }

  public clearResponseCache() {
    this.responseCache.clear();
  }

  public resetRateLimits(key?: string) {
    this.rateLimiter.reset(key);
  }

  private buildUrl(
    endpoint: string,
    params?: Record<string, string | number | boolean | undefined>
  ) {
    let url = `${this.baseURL}${endpoint}`;

    if (!params) return url;

    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined) {
        searchParams.append(key, String(value));
      }
    });

    const queryString = searchParams.toString();
    if (queryString) {
      url += `?${queryString}`;
    }

    return url;
  }

  private buildRequestConfig(
    endpoint: string,
    method: HttpMethod,
    config: RequestConfig
  ): PreparedRequestConfig {
    const { params, body, ...customInit } = config;
    const headers = new Headers(customInit.headers || {});

    if (
      !headers.has('Content-Type') &&
      body !== undefined &&
      !(body instanceof FormData)
    ) {
      headers.set('Content-Type', 'application/json');
    }
    if (!headers.has('Accept')) {
      headers.set('Accept', 'application/json');
    }

    return {
      ...customInit,
      url: this.buildUrl(endpoint, params),
      method,
      headers,
      body:
        body !== undefined
          ? body instanceof FormData
            ? body
            : JSON.stringify(body)
          : undefined,
    } as PreparedRequestConfig;
  }

  private getCacheKey(config: PreparedRequestConfig) {
    const walletKey = config.headers.get('X-Wallet-Pubkey') ?? 'anonymous';
    return `${config.method}:${config.url}:wallet=${walletKey}`;
  }

  private getCachedResponse<T>(key: string): T | undefined {
    const cached = this.responseCache.get(key);
    if (!cached) return undefined;

    if (Date.now() >= cached.expiresAt) {
      this.responseCache.delete(key);
      return undefined;
    }

    return cached.data as T;
  }

  private setCachedResponse<T>(key: string, data: T, cacheTtlMs: number) {
    if (cacheTtlMs <= 0) return;

    this.responseCache.set(key, {
      data,
      expiresAt: Date.now() + cacheTtlMs,
    });
  }

  private enforceRateLimit(
    config: PreparedRequestConfig,
    endpoint: string,
    notify: boolean
  ) {
    if (config.rateLimit === false) return;

    const options = resolveRateLimitOptions({
      ...this.defaultRateLimit,
      ...config.rateLimit,
    });
    const result = this.rateLimiter.consume(
      config.rateLimitKey ?? DEFAULT_RATE_LIMIT_KEY,
      options
    );

    if (!result.allowed) {
      const error = new RateLimitError(result, endpoint);
      if (notify) {
        notifyRateLimit(error);
      }
      throw error;
    }
  }

  private createServerRateLimitError(
    response: Response,
    endpoint: string
  ): RateLimitError {
    const retryAfterMs =
      parseRetryAfterHeader(response.headers.get('Retry-After')) ??
      this.defaultRateLimit.windowMs;

    return new RateLimitError(
      {
        ...this.defaultRateLimit,
        allowed: false,
        remaining: 0,
        resetAt: Date.now() + retryAfterMs,
        retryAfterMs,
      },
      endpoint
    );
  }

  private getBackoffMs(retryDelay: number, attempt: number) {
    return retryDelay * 2 ** Math.max(0, attempt - 1);
  }

  // Core request method
  async request<T>(
    endpoint: string,
    method: HttpMethod,
    config: RequestConfig = {}
  ): Promise<T> {
    this.startRequest();
    try {
      const {
        timeout = this.defaultTimeout,
        retries = 3,
        retryDelay = 1000,
        cacheResponse = true,
        cacheTtlMs = DEFAULT_CACHE_TTL_MS,
        skipRateLimitNotification = false,
      } = config;

      let requestConfig = this.buildRequestConfig(endpoint, method, config);

      // Apply request interceptors
      requestConfig = await this.applyRequestInterceptors(requestConfig);

      const shouldCache = method === 'GET' && cacheResponse;
      const cacheKey = shouldCache ? this.getCacheKey(requestConfig) : null;
      if (cacheKey) {
        const cached = this.getCachedResponse<T>(cacheKey);
        if (cached !== undefined) return cached;
      }

      this.enforceRateLimit(
        requestConfig,
        endpoint,
        !skipRateLimitNotification
      );

      let attempt = 0;
      while (attempt <= retries) {
        const abortController = new AbortController();
        const timeoutId = setTimeout(() => abortController.abort(), timeout);

        try {
          const fetchInit: RequestInit = {
            method: requestConfig.method,
            headers: requestConfig.headers,
            body: requestConfig.body as BodyInit,
            signal: abortController.signal,
            credentials: requestConfig.credentials,
            cache: requestConfig.cache,
            mode: requestConfig.mode,
          };

          let response = await fetch(requestConfig.url, fetchInit);
          clearTimeout(timeoutId);

          // Apply response interceptors
          response = await this.applyResponseInterceptors(response);

          if (response.status === 429) {
            const rateLimitError = this.createServerRateLimitError(
              response,
              endpoint
            );
            if (!skipRateLimitNotification) {
              notifyRateLimit(rateLimitError);
            }
            throw rateLimitError;
          }

          if (!response.ok) {
            let errorData;
            try {
              errorData = await response.json();
            } catch {
              errorData = await response.text();
            }
            if (response.status >= 500 && response.status < 600) {
              if (typeof window !== 'undefined') {
                toast('Something went wrong. Please try again.', 'error');
              }
            }
            if (response.status === 401) {
              throw new UnauthorizedError(response.statusText, errorData);
            }
            throw new ApiError(response.status, response.statusText, errorData);
          }

          // Handle 204 No Content
          if (response.status === 204) {
            return {} as T;
          }

          const data = (await response.json()) as T;
          if (cacheKey) {
            this.setCachedResponse(cacheKey, data, cacheTtlMs);
          }

          return data;
        } catch (error) {
          clearTimeout(timeoutId);

          if (isRateLimitError(error)) {
            throw error;
          }

          let finalError = error as Error | ApiError;
          if (
            error &&
            typeof error === 'object' &&
            'name' in error &&
            error.name === 'AbortError'
          ) {
            finalError = new Error(
              `Request timed out. Please check your connection.`
            );
          }

          // Don't retry on client errors (4xx) except 429 Too Many Requests
          if (
            finalError instanceof ApiError &&
            finalError.status >= 400 &&
            finalError.status < 500
          ) {
            throw finalError;
          }

          attempt++;
          if (attempt > retries) {
            throw finalError;
          }

          // Wait before retrying
          await new Promise((resolve) =>
            setTimeout(resolve, this.getBackoffMs(retryDelay, attempt))
          );
        }
      }

      throw new Error('Request failed');
    } finally {
      this.endRequest();
    }
  }

  // Convenience methods
  get<T>(endpoint: string, config?: Omit<RequestConfig, 'body'>) {
    return this.request<T>(endpoint, 'GET', config);
  }

  post<T>(endpoint: string, body?: unknown, config?: RequestConfig) {
    return this.request<T>(endpoint, 'POST', { ...config, body });
  }

  put<T>(endpoint: string, body?: unknown, config?: RequestConfig) {
    return this.request<T>(endpoint, 'PUT', { ...config, body });
  }

  patch<T>(endpoint: string, body?: unknown, config?: RequestConfig) {
    return this.request<T>(endpoint, 'PATCH', { ...config, body });
  }

  delete<T>(endpoint: string, config?: RequestConfig) {
    return this.request<T>(endpoint, 'DELETE', config);
  }

  // ─── Typed helpers ─────────────────────────────────────────────────────────
  // These constrain the endpoint to a path that actually exists in the
  // generated OpenAPI schema and infer the request/response shapes from it.

  typedGet<Path extends ApiPath>(
    endpoint: Path,
    config?: Omit<RequestConfig, 'body'> & {
      params?: ApiQueryParams<Path, 'get'>;
    }
  ): Promise<ApiResponseBody<Path, 'get'>> {
    return this.request<ApiResponseBody<Path, 'get'>>(endpoint, 'GET', config);
  }

  typedPost<Path extends ApiPath>(
    endpoint: Path,
    body?: ApiRequestBody<Path, 'post'>,
    config?: RequestConfig
  ): Promise<ApiResponseBody<Path, 'post'>> {
    return this.request<ApiResponseBody<Path, 'post'>>(endpoint, 'POST', {
      ...config,
      body,
    });
  }

  typedPatch<Path extends ApiPath>(
    endpoint: Path,
    body?: ApiRequestBody<Path, 'patch'>,
    config?: RequestConfig
  ): Promise<ApiResponseBody<Path, 'patch'>> {
    return this.request<ApiResponseBody<Path, 'patch'>>(endpoint, 'PATCH', {
      ...config,
      body,
    });
  }

  typedDelete<Path extends ApiPath>(
    endpoint: Path,
    config?: RequestConfig
  ): Promise<ApiResponseBody<Path, 'delete'>> {
    return this.request<ApiResponseBody<Path, 'delete'>>(
      endpoint,
      'DELETE',
      config
    );
  }
}

export const apiClient = new ApiClient();
export {
  RateLimitError,
  getRateLimitRemainingMs,
  isRateLimitError,
} from './rate-limit';
export type {
  ApiPool,
  ApiDonation,
  ApiUser,
  PaginatedResponse,
} from './api-types';

// ─── JWT helpers ────────────────────────────────────────────────────────────
// The token lives in exactly one place: lib/jwt-storage.ts. Re-exported here
// for backwards compatibility with existing imports.

export { getToken as getJwt, setToken as setJwt, clearAuth as clearJwt };

// ─── 401 response interceptor ────────────────────────────────────────────────
// On any 401 outside the auth endpoints themselves, clear all auth state
// (token + wallet store) via the single clearAuth() helper and redirect.

const AUTH_ENDPOINTS = ['/auth/challenge', '/auth/verify'];

apiClient.addResponseInterceptor((response) => {
  if (response.status === 401) {
    const url = response.url ?? '';
    const isAuthEndpoint = AUTH_ENDPOINTS.some((ep) => url.includes(ep));

    if (!isAuthEndpoint) {
      clearAuth();

      if (typeof window !== 'undefined') {
        window.location.href = '/login';
      }
    }
  }
  return response;
});

// ─── API call functions ──────────────────────────────────────────────────────
// Every function below is typed against the generated OpenAPI schema. Passing
// an endpoint that doesn't exist, or a body of the wrong shape, is a type error.

export function updateProfile(
  displayName: string
): Promise<ApiResponseBody<'/users/me', 'patch'>> {
  return apiClient.typedPatch('/users/me', { displayName }, {
    requireAuth: true,
  });
}

export type CreatePoolPayload = ApiRequestBody<'/pools', 'post'>;

export type CreatePoolResponse = ApiResponseBody<'/pools', 'post'>;

export function fetchMyDonations(
  limit?: number
): Promise<ApiResponseBody<'/users/me/donations', 'get'>> {
  return apiClient.typedGet(
  params?: Record<string, string | number | boolean | undefined>
): Promise<PaginatedResponse<ApiDonation>> {
  return apiClient.get<PaginatedResponse<ApiDonation>>(
    '/users/me/donations',
    params ? { params } : undefined
  );
}

export function fetchPoolDonations(
  poolId: string | number,
  params?: Record<string, string | number | boolean | undefined>
): Promise<PaginatedResponse<ApiDonation>> {
  return apiClient.get<PaginatedResponse<ApiDonation>>(
    `/pools/${poolId}/donations`,
    params ? { params } : undefined
  );
}

export function fetchCreatorPools(publicKey: string): Promise<ApiPool[]> {
  return apiClient.get<ApiPool[]>('/pools', {
    params: { creator: publicKey },
  });
}

export interface TransactionQueryParams {
  page?: number;
  limit?: number;
  type?: string;
  status?: string;
  poolId?: string | number;
  asset?: string;
  from?: string;
  to?: string;
}

export function fetchTransactions(
  params: TransactionQueryParams = {}
): Promise<PaginatedResponse<ApiDonation>> {
  return apiClient.get<PaginatedResponse<ApiDonation>>('/transactions', {
    params: params as Record<string, string | number | boolean | undefined>,
    requireAuth: true,
  });
export function fetchCreatorPools(
  publicKey: string
): Promise<ApiResponseBody<'/pools', 'get'>> {
  return apiClient.typedGet('/pools', { params: { creator: publicKey } });
}

export function fetchPools(
  params?: ApiQueryParams<'/pools', 'get'>
): Promise<ApiResponseBody<'/pools', 'get'>> {
  return apiClient.typedGet('/pools', { params });
}

export function fetchPool(
  id: string | number
): Promise<ApiResponseBody<'/pools/{id}', 'get'>> {
  return apiClient.typedGet(`/pools/${id}` as '/pools/{id}');
}

export function createPool(
  payload: CreatePoolPayload
): Promise<CreatePoolResponse> {
  return apiClient.typedPost('/pools', payload, { requireAuth: true });
}

export function closePool(
  poolId: string | number
): Promise<ApiResponseBody<'/pools/{id}/close', 'post'>> {
  return apiClient.typedPost(
    `/pools/${poolId}/close` as '/pools/{id}/close',
    undefined,
    { requireAuth: true }
  );
}

export function withdrawPool(
  poolId: string | number
): Promise<ApiResponseBody<'/pools/{id}/withdraw', 'post'>> {
  return apiClient.typedPost(
    `/pools/${poolId}/withdraw` as '/pools/{id}/withdraw',
    undefined,
    { requireAuth: true }
  );
}

export function donate(
  poolId: number,
  amount: string,
  tokenAddress: string
): Promise<ApiResponseBody<'/pools/{id}/donations', 'post'>> {
  return apiClient.typedPost(
    `/pools/${poolId}/donations` as '/pools/{id}/donations',
    { amount, tokenAddress },
    { requireAuth: true }
  );
}

export function submitSignedXdr(
  xdr: string
): Promise<ApiResponseBody<'/transactions/submit', 'post'>> {
  return apiClient.typedPost('/transactions/submit', { xdr });
}

export function verifyAuthSignature(
  publicKey: string,
  nonce: string,
  signature: string
): Promise<ApiResponseBody<'/auth/verify', 'post'>> {
  return apiClient.typedPost('/auth/verify', {
    publicKey,
    signature,
    message: nonce,
  });
}

export type AuthChallenge = ApiResponseBody<'/auth/challenge', 'get'>;

export function fetchAuthChallenge(
  publicKey: string
): Promise<AuthChallenge> {
  return apiClient.typedGet('/auth/challenge', {
    params: { publicKey },
    requireAuth: false,
    cacheResponse: false,
  });
}

// ─── Pool comments ──────────────────────────────────────────────────────────

export type PoolComment = ApiResponseBody<'/pools/{id}/comments', 'get'> extends
  (infer Item)[]
  ? Item
  : never;

export type CreateCommentPayload = ApiRequestBody<
  '/pools/{id}/comments',
  'post'
>;

export type UpdateCommentPayload = ApiRequestBody<
  '/pools/{id}/comments/{commentId}',
  'patch'
>;

export function fetchPoolComments(
  poolId: string | number
): Promise<PoolComment[]> {
  return apiClient.typedGet(
    `/pools/${poolId}/comments` as '/pools/{id}/comments'
  );
}

export function createPoolComment(
  poolId: string | number,
  payload: CreateCommentPayload
): Promise<PoolComment> {
  return apiClient.typedPost(
    `/pools/${poolId}/comments` as '/pools/{id}/comments',
    payload,
// ─── Comments ────────────────────────────────────────────────────────────────

export interface ApiComment {
  id: string;
  poolId: number;
  authorWallet: string;
  text: string;
  parentId: string | null;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  replies?: ApiComment[];
}

export interface CommentThread {
  data: ApiComment[];
  total: number;
  page: number;
  limit: number;
}

export function fetchComments(
  poolId: string | number,
  params?: Record<string, string | number | boolean | undefined>
): Promise<CommentThread> {
  return apiClient.get<CommentThread>(
    `/pools/${poolId}/comments`,
    params ? { params } : undefined
  );
}

export function postComment(
  poolId: string | number,
  text: string,
  parentId?: string | null
): Promise<ApiComment> {
  return apiClient.post<ApiComment>(
    `/pools/${poolId}/comments`,
    { text, parentId: parentId ?? null },
    { requireAuth: true }
  );
}

export function updatePoolComment(
  poolId: string | number,
  commentId: string | number,
  payload: UpdateCommentPayload
): Promise<PoolComment> {
  return apiClient.typedPatch(
    `/pools/${poolId}/comments/${commentId}` as '/pools/{id}/comments/{commentId}',
    payload,
    { requireAuth: true }
  );
}

export function deletePoolComment(
  poolId: string | number,
  commentId: string | number
): Promise<void> {
  return apiClient.typedDelete(
    `/pools/${poolId}/comments/${commentId}` as '/pools/{id}/comments/{commentId}',
    { requireAuth: true }
  );
export function updateComment(
  poolId: string | number,
  commentId: string,
  text: string
): Promise<ApiComment> {
  return apiClient.request<ApiComment>(
    `/pools/${poolId}/comments/${commentId}`,
    'PATCH',
    { body: { text }, requireAuth: true }
  );
}

export function deleteComment(
  poolId: string | number,
  commentId: string
): Promise<void> {
  return apiClient.delete<void>(`/pools/${poolId}/comments/${commentId}`, {
    requireAuth: true,
  });
}

// ─── Type exports for main branch compatibility ─────────────────────────────

export interface ApiDonation {
  id: string;
  type: 'donation' | 'pool_creation' | 'withdrawal';
  amount: string;
  asset: string;
  recipient: string;
  date: string;
  status: 'completed' | 'pending' | 'failed';
  txHash: string;
}

export interface ApiProfile {
  publicKey: string;
  displayName: string | null;
  createdAt: string;
}