import { useState, useCallback, useEffect } from 'react';
import { ApiError, apiClient } from '../lib/api-client';

interface UseApiResponse<T, Args extends unknown[] = unknown[]> {
  data: T | null;
  error: ApiError | Error | null;
  isLoading: boolean;
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

  const execute = useCallback(
    async (...args: Args): Promise<T | null> => {
      try {
        setIsLoading(true);
        setError(null);
        const result = await apiFunction(...args);
        setData(result);
        if (options.onSuccess) {
          options.onSuccess(result);
        }
        return result;
      } catch (err) {
        const errorObject = err instanceof Error ? err : new Error(String(err));
        setError(errorObject);
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

  const reset = useCallback(() => {
    setData(options.initialData ?? null);
    setError(null);
    setIsLoading(false);
  }, [options.initialData]);

  return {
    data,
    error,
    isLoading,
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
