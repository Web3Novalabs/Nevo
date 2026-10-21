import { renderHook, act } from '@testing-library/react';
import { useCopyToClipboard } from './useCopyToClipboard';

describe('useCopyToClipboard', () => {
  beforeEach(() => {
    jest.useFakeTimers();
    Object.assign(navigator, {
      clipboard: {
        writeText: jest.fn().mockResolvedValue(undefined),
      },
    });
    document.execCommand = jest.fn();
  });

  afterEach(() => {
    act(() => {
      jest.runOnlyPendingTimers();
    });
    jest.useRealTimers();
    jest.restoreAllMocks();
  });

  it('copies via the Clipboard API and marks the state as copied', async () => {
    const { result } = renderHook(() => useCopyToClipboard(1000));

    await act(async () => {
      await result.current.copy('hello');
    });

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('hello');
    expect(result.current.copied).toBe(true);
    expect(result.current.error).toBeNull();
  });

  it('falls back to execCommand when the Clipboard API is unavailable', async () => {
    Object.assign(navigator, {
      clipboard: undefined,
    });

    const { result } = renderHook(() => useCopyToClipboard(1000));

    await act(async () => {
      await result.current.copy('fallback');
    });

    expect(document.execCommand).toHaveBeenCalledWith('copy');
    expect(result.current.copied).toBe(true);
    expect(result.current.error).toBeNull();
  });

  it('resets copied state after the timeout expires', async () => {
    const { result } = renderHook(() => useCopyToClipboard(1000));

    await act(async () => {
      await result.current.copy('reset');
    });

    expect(result.current.copied).toBe(true);

    act(() => {
      jest.advanceTimersByTime(1000);
    });

    expect(result.current.copied).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('falls back to a temporary textarea when writeText rejects', async () => {
    (navigator.clipboard.writeText as jest.Mock).mockRejectedValue(
      new Error('denied')
    );

    const { result } = renderHook(() => useCopyToClipboard(1000));

    await act(async () => {
      await result.current.copy('fallback');
    });

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('fallback');
    expect(document.execCommand).toHaveBeenCalledWith('copy');
    expect(result.current.copied).toBe(true);
    expect(result.current.error).toBeNull();
    expect(document.querySelector('textarea')).toBeNull();
  });

  it('sets an error when both the Clipboard API and execCommand fail', async () => {
    (navigator.clipboard.writeText as jest.Mock).mockRejectedValue(
      new Error('denied')
    );
    (document.execCommand as jest.Mock).mockImplementation(() => {
      throw new Error('execCommand failed');
    });

    const { result } = renderHook(() => useCopyToClipboard(1000));

    await act(async () => {
      await result.current.copy('fail');
    });

    expect(result.current.copied).toBe(false);
    expect(result.current.error).toBe('Failed to copy');
    expect(document.querySelector('textarea')).toBeNull();
  });

  it('clears the pending reset timeout when copy is called again', async () => {
    const { result } = renderHook(() => useCopyToClipboard(1000));

    await act(async () => {
      await result.current.copy('a');
    });

    expect(result.current.copied).toBe(true);

    act(() => {
      jest.advanceTimersByTime(500);
    });

    await act(async () => {
      await result.current.copy('b');
    });

    expect(result.current.copied).toBe(true);

    // Advancing past the first resetDelay should not reset copied,
    // since the first timeout was cleared by the second copy() call.
    act(() => {
      jest.advanceTimersByTime(600);
    });

    expect(result.current.copied).toBe(true);

    // Only after the second resetDelay elapses does copied reset.
    act(() => {
      jest.advanceTimersByTime(400);
    });

    expect(result.current.copied).toBe(false);
  });
});
