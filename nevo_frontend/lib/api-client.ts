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

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';

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
    defaultTimeout: number = 10000,
    rateLimit: Partial<RateLimitOptions> = DEFAULT_RATE_LIMIT_OPTIONS
  ) {
    this.baseURL =
      baseURL ||
      process.env.NEXT_PUBLIC_API_BASE_URL ||
      'http://localhost:3000';
    this.defaultTimeout = defaultTimeout;
    this.defaultRateLimit = resolveRateLimitOptions(rateLimit);
    this.rateLimiter = new ClientRateLimiter();
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
            finalError = new Error(`Request timed out after ${timeout}ms`);
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

  delete<T>(endpoint: string, config?: RequestConfig) {
    return this.request<T>(endpoint, 'DELETE', config);
  }
}

export const apiClient = new ApiClient();
export {
  RateLimitError,
  getRateLimitRemainingMs,
  isRateLimitError,
} from './rate-limit';

// Add default auth interceptor for wallet signature
apiClient.addRequestInterceptor((config) => {
  if (config.requireAuth !== false) {
    // In a real app, you would get the wallet signature from your auth state/store
    const signature =
      typeof window !== 'undefined'
        ? localStorage.getItem('wallet_signature')
        : null;
    const pubKey =
      typeof window !== 'undefined'
        ? localStorage.getItem('wallet_pubkey')
        : null;

    if (signature && pubKey) {
      const headers = new Headers(config.headers);
      headers.set('X-Wallet-Signature', signature);
      headers.set('X-Wallet-Pubkey', pubKey);
      config.headers = headers;
    }
  }
  return config;
});
