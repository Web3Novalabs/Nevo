'use client';

import { useState, useCallback, useRef } from 'react';

interface CopyState {
  copied: boolean;
  error: string | null;
}

export function useCopyToClipboard(resetDelay = 2000) {
  const [state, setState] = useState<CopyState>({ copied: false, error: null });
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const copy = useCallback(
    async (text: string) => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);

      try {
        await navigator.clipboard.writeText(text);
        setState({ copied: true, error: null });
        timeoutRef.current = setTimeout(
          () => setState({ copied: false, error: null }),
          resetDelay
        );
      } catch {
        const el = document.createElement('textarea');
        el.value = text;
        el.style.position = 'fixed';
        el.style.opacity = '0';
        document.body.appendChild(el);
        el.select();
        try {
          document.execCommand('copy');
          setState({ copied: true, error: null });
          timeoutRef.current = setTimeout(
            () => setState({ copied: false, error: null }),
            resetDelay
          );
        } catch {
          setState({ copied: false, error: 'Failed to copy' });
        } finally {
          document.body.removeChild(el);
        }
      }
    },
    [resetDelay]
  );

  return { ...state, copy };
}
