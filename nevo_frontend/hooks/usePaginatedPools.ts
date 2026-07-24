import { useEffect, useState } from 'react';
import { apiClient } from '@/lib/api-client';
import type { Pool } from '@/src/store/poolsStore';

export interface UsePaginatedPoolsParams {
  creator?: string;
  page?: number;
  limit?: number;
}

interface PoolsPageResponse {
  data: Pool[];
  total: number;
  page: number;
  limit: number;
}

export interface UsePaginatedPoolsResult {
  pools: Pool[];
  totalPages: number;
  currentPage: number;
  setPage: (page: number) => void;
  loading: boolean;
  error: string | null;
}

export function usePaginatedPools(
  params: UsePaginatedPoolsParams = {}
): UsePaginatedPoolsResult {
  const { creator, page = 1, limit = 10 } = params;

  const [pools, setPools] = useState<Pool[]>([]);
  const [totalPages, setTotalPages] = useState(1);
  const [currentPage, setCurrentPage] = useState(page);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setCurrentPage(page);
  }, [page]);

  useEffect(() => {
    let cancelled = false;

    // eslint-disable-next-line react-hooks/set-state-in-effect
    setLoading(true);
    setError(null);

    apiClient
      .get<PoolsPageResponse>('/pools', {
        params: { creator, page: currentPage, limit },
      })
      .then((res) => {
        if (cancelled) return;
        setPools(res?.data ?? []);
        setTotalPages(
          res?.total ? Math.max(1, Math.ceil(res.total / limit)) : 1
        );
      })
      .catch((err) => {
        if (cancelled) return;
        const message =
          err instanceof Error ? err.message : 'Failed to load pools';
        setError(message);
        setPools([]);
        setTotalPages(1);
      })
      .finally(() => {
        if (cancelled) return;
        setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [creator, currentPage, limit]);

  return {
    pools,
    totalPages,
    currentPage,
    setPage: setCurrentPage,
    loading,
    error,
  };
}
