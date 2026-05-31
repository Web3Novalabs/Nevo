import { useState, useCallback, useEffect, useRef } from 'react';
import {
  ApiError,
  RateLimitError,
  apiClient,
  getRateLimitRemainingMs,
  isRateLimitError,
} from '../lib/api-client';

interface RateLimitState<T> {
  error: RateLimitError;
  remainingSeconds: number;
  canRetry: boolean;
  retry: () => Promise<T | null>;
}

interface UseApiResponse<T, Args extends unknown[] = unknown[]> {
  data: T | null;
  error: ApiError | Error | null;
  isLoading: boolean;
  rateLimit: RateLimitState<T> | null;
  execute: (...args: Args) => Promise<T | null>;
  reset: () => void;
}

export function useApi<T, Args extends unknown[] = unknown[]>(
  apiFunction: (...args: Args) => Promise<T>,
  options: {
    onSuccess?: (data: T) => void;
    onError?: (error: ApiError | Error) => void;
    initialData?: T;
  } = {}
): UseApiResponse<T, Args> {
  const [data, setData] = useState<T | null>(options.initialData ?? null);
  const [error, setError] = useState<ApiError | Error | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [rateLimitError, setRateLimitError] = useState<RateLimitError | null>(
    null
  );
  const [rateLimitRemainingMs, setRateLimitRemainingMs] = useState(0);
  const lastArgsRef = useRef<Args | null>(null);

  const execute = useCallback(
    async (...args: Args): Promise<T | null> => {
      lastArgsRef.current = args;
      try {
        setIsLoading(true);
        setError(null);
        setRateLimitError(null);
        const result = await apiFunction(...args);
        setData(result);
        if (options.onSuccess) {
          options.onSuccess(result);
        }
        return result;
      } catch (err) {
        const errorObject = err instanceof Error ? err : new Error(String(err));
        setError(errorObject);
        if (isRateLimitError(errorObject)) {
          setRateLimitError(errorObject);
          setRateLimitRemainingMs(getRateLimitRemainingMs(errorObject));
        } else {
          setRateLimitError(null);
          setRateLimitRemainingMs(0);
        }
        if (options.onError) {
          options.onError(errorObject);
        }
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    [apiFunction, options]
  );

  useEffect(() => {
    if (!rateLimitError) return;
    const currentError = rateLimitError;

    function tick() {
      setRateLimitRemainingMs(getRateLimitRemainingMs(currentError));
    }

    tick();
    const interval = window.setInterval(tick, 1000);
    return () => window.clearInterval(interval);
  }, [rateLimitError]);

  const retry = useCallback(() => {
    if (!lastArgsRef.current) return Promise.resolve(null);
    return execute(...lastArgsRef.current);
  }, [execute]);

  const reset = useCallback(() => {
    setData(options.initialData ?? null);
    setError(null);
    setRateLimitError(null);
    setRateLimitRemainingMs(0);
    setIsLoading(false);
  }, [options.initialData]);

  return {
    data,
    error,
    isLoading,
    rateLimit: rateLimitError
      ? {
          error: rateLimitError,
          remainingSeconds: Math.ceil(rateLimitRemainingMs / 1000),
          canRetry: rateLimitRemainingMs <= 0,
          retry,
        }
      : null,
    execute,
    reset,
  };
}

/**
 * Hook to subscribe to the global API client's loading state.
 * Returns true if there are any active in-flight requests.
 */
export function useApiClientLoading(): boolean {
  const [isLoading, setIsLoading] = useState(apiClient.isLoading);

  useEffect(() => {
    return apiClient.subscribeToLoading(setIsLoading);
  }, []);

  return isLoading;
}
