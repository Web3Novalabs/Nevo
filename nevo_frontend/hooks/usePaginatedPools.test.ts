import { renderHook, waitFor, act } from '@testing-library/react';
import { usePaginatedPools } from './usePaginatedPools';
import { apiClient } from '@/lib/api-client';

jest.mock('@/lib/api-client', () => ({
  apiClient: {
    get: jest.fn(),
  },
}));

const mockedGet = apiClient.get as jest.Mock;

type PoolsResponse = {
  pools: Array<{ id: string; name: string }>;
  total: number;
};

const makeResponse = (total: number, pools: Array<{ id: string; name: string }> = []): PoolsResponse => ({
  pools,
  total,
});

describe('usePaginatedPools', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('fetches on mount with the initial page and toggles loading', async () => {
    mockedGet.mockResolvedValueOnce(makeResponse(5, [{ id: '1', name: 'Pool 1' }]));

    const { result } = renderHook(() => usePaginatedPools({ page: 1, limit: 10 }));

    expect(result.current.loading).toBe(true);
    expect(mockedGet).toHaveBeenCalledTimes(1);
    expect(mockedGet).toHaveBeenCalledWith(expect.any(String), expect.objectContaining({ params: expect.objectContaining({ page: 1 }) }));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.pools).toEqual([{ id: '1', name: 'Pool 1' }]);
    expect(result.current.totalPages).toBe(1);
    expect(result.current.error).toBeNull();
  });

  it('updates currentPage and triggers a new fetch when the page prop changes', async () => {
    mockedGet.mockResolvedValueOnce(makeResponse(25, [{ id: '1', name: 'Pool 1' }]));

    const { result, rerender } = renderHook(
      ({ page }: { page: number }) => usePaginatedPools({ page, limit: 10 }),
      { initialProps: { page: 1 } },
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.currentPage).toBe(1);
    expect(mockedGet).toHaveBeenCalledTimes(1);

    mockedGet.mockResolvedValueOnce(makeResponse(25, [{ id: '2', name: 'Pool 2' }]));

    rerender({ page: 2 });

    await waitFor(() => {
      expect(result.current.currentPage).toBe(2);
    });

    expect(mockedGet).toHaveBeenCalledTimes(2);
    expect(mockedGet).toHaveBeenLastCalledWith(
      expect.any(String),
      expect.objectContaining({ params: expect.objectContaining({ page: 2 }) }),
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.pools).toEqual([{ id: '2', name: 'Pool 2' }]);
  });

  it('does not call setState when an in-flight request is cancelled by unmount', async () => {
    let resolveRequest: (value: PoolsResponse) => void = () => {};
    const pending = new Promise<PoolsResponse>((resolve) => {
      resolveRequest = resolve;
    });
    mockedGet.mockReturnValueOnce(pending);

    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

    const { unmount } = renderHook(() => usePaginatedPools({ page: 1, limit: 10 }));

    expect(mockedGet).toHaveBeenCalledTimes(1);

    unmount();

    await act(async () => {
      resolveRequest(makeResponse(10, [{ id: '1', name: 'Pool 1' }]));
      await pending;
    });

    expect(consoleErrorSpy).not.toHaveBeenCalled();

    consoleErrorSpy.mockRestore();
  });

  it('does not apply a stale response when the dependency changes (cancelled flag short-circuits)', async () => {
    let resolveFirst: (value: PoolsResponse) => void = () => {};
    const firstPending = new Promise<PoolsResponse>((resolve) => {
      resolveFirst = resolve;
    });
    mockedGet.mockReturnValueOnce(firstPending);

    const { result, rerender } = renderHook(
      ({ page }: { page: number }) => usePaginatedPools({ page, limit: 10 }),
      { initialProps: { page: 1 } },
    );

    expect(mockedGet).toHaveBeenCalledTimes(1);

    mockedGet.mockResolvedValueOnce(makeResponse(20, [{ id: '2', name: 'Pool 2' }]));

    rerender({ page: 2 });

    await waitFor(() => {
      expect(result.current.currentPage).toBe(2);
    });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.pools).toEqual([{ id: '2', name: 'Pool 2' }]);

    await act(async () => {
      resolveFirst(makeResponse(20, [{ id: '1', name: 'Stale Pool' }]));
      await firstPending;
    });

    expect(result.current.pools).toEqual([{ id: '2', name: 'Pool 2' }]);
    expect(result.current.currentPage).toBe(2);
  });

  it('sets error to the Error message and resets pools/totalPages on rejection', async () => {
    mockedGet.mockRejectedValueOnce(new Error('Network down'));

    const { result } = renderHook(() => usePaginatedPools({ page: 1, limit: 10 }));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe('Network down');
    expect(result.current.pools).toEqual([]);
    expect(result.current.totalPages).toBe(1);
  });

  it('sets fallback error for non-Error rejection and resets pools/totalPages', async () => {
    mockedGet.mockRejectedValueOnce('boom');

    const { result } = renderHook(() => usePaginatedPools({ page: 1, limit: 10 }));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe('Failed to load pools');
    expect(result.current.pools).toEqual([]);
    expect(result.current.totalPages).toBe(1);
  });

  it.each([
    [0, 10, 1],
    [1, 10, 1],
    [10, 10, 1],
    [11, 10, 2],
    [25, 10, 3],
    [100, 25, 4],
  ])('computes totalPages as Math.max(1, Math.ceil(total/limit)) for total=%i limit=%i', async (total, limit, expected) => {
    mockedGet.mockResolvedValueOnce(makeResponse(total));

    const { result } = renderHook(() => usePaginatedPools({ page: 1, limit }));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.totalPages).toBe(expected);
    expect(result.current.totalPages).toBe(Math.max(1, Math.ceil(total / limit)));
  });
});