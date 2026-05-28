export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';

export interface RequestConfig extends Omit<RequestInit, 'method' | 'body'> {
  params?: Record<string, string | number | boolean | undefined>;
  timeout?: number;
  retries?: number;
  retryDelay?: number;
  body?: unknown;
  requireAuth?: boolean;
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

class ApiClient {
  private baseURL: string;
  private defaultTimeout: number;
  private activeRequests = 0;
  private loadingListeners: Set<(loading: boolean) => void> = new Set();

  public requestInterceptors: Interceptor<
    RequestConfig & { url: string; method: string }
  >[] = [];
  public responseInterceptors: Interceptor<Response>[] = [];

  constructor(baseURL: string = '', defaultTimeout: number = 10000) {
    this.baseURL =
      baseURL ||
      process.env.NEXT_PUBLIC_API_BASE_URL ||
      'http://localhost:3000';
    this.defaultTimeout = defaultTimeout;
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
  addRequestInterceptor(
    interceptor: Interceptor<RequestConfig & { url: string; method: string }>
  ) {
    this.requestInterceptors.push(interceptor);
  }

  addResponseInterceptor(interceptor: Interceptor<Response>) {
    this.responseInterceptors.push(interceptor);
  }

  private async applyRequestInterceptors(
    config: RequestConfig & { url: string; method: string }
  ) {
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

  // Core request method
  async request<T>(
    endpoint: string,
    method: HttpMethod,
    config: RequestConfig = {}
  ): Promise<T> {
    this.startRequest();
    try {
      const {
        params,
        timeout = this.defaultTimeout,
        retries = 3,
        retryDelay = 1000,
        body,
        ...customInit
      } = config;

      let url = `${this.baseURL}${endpoint}`;

      // Add query params
      if (params) {
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
      }

      // Default headers
      const headers = new Headers(customInit.headers || {});
      if (!headers.has('Content-Type') && body && !(body instanceof FormData)) {
        headers.set('Content-Type', 'application/json');
      }
      if (!headers.has('Accept')) {
        headers.set('Accept', 'application/json');
      }

      let requestConfig: RequestConfig & { url: string; method: string } = {
        ...customInit,
        url,
        method,
        headers,
        body: body
          ? body instanceof FormData
            ? body
            : JSON.stringify(body)
          : undefined,
      } as RequestConfig & { url: string; method: string };

      // Apply request interceptors
      requestConfig = await this.applyRequestInterceptors(requestConfig);

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

          return await response.json();
        } catch (error) {
          clearTimeout(timeoutId);

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
            finalError.status < 500 &&
            finalError.status !== 429
          ) {
            throw finalError;
          }

          attempt++;
          if (attempt > retries) {
            throw finalError;
          }

          // Wait before retrying
          await new Promise((resolve) =>
            setTimeout(resolve, retryDelay * attempt)
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
