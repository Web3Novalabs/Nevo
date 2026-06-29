'use client';

import { useCallback, useState } from 'react';
import { signTransaction } from '@/lib/stellar';

type TxStatus = 'idle' | 'signing' | 'submitting' | 'success' | 'error';

type XdrPayload = string | { unsignedXdr: string };

type GetXdrFn = () => Promise<XdrPayload>;
type SubmitXdrFn = (
  signedXdr: string
) => Promise<string | { txHash?: string; hash?: string }>;

function extractUnsignedXdr(payload: XdrPayload): string {
  if (typeof payload === 'string') {
    return payload;
  }

  return payload.unsignedXdr;
}

function extractTxHash(
  result: string | { txHash?: string; hash?: string }
): string {
  if (typeof result === 'string') {
    return result;
  }

  return result.txHash ?? result.hash ?? '';
}

export function useXdrTransaction() {
  const [status, setStatus] = useState<TxStatus>('idle');
  const [txHash, setTxHash] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const submit = useCallback(
    async (getXdr: GetXdrFn, submitXdr: SubmitXdrFn) => {
      setStatus('signing');
      setError(null);
      setTxHash(null);

      try {
        const xdrPayload = await getXdr();
        const unsignedXdr = extractUnsignedXdr(xdrPayload);
        const signedXdr = await signTransaction(unsignedXdr);

        setStatus('submitting');
        const submitResult = await submitXdr(signedXdr);
        const hash = extractTxHash(submitResult);

        setTxHash(hash || null);
        setStatus('success');
        return hash;
      } catch (err) {
        const message =
          err instanceof Error ? err.message : 'Failed to submit transaction';
        setError(message);
        setStatus('error');
        throw err;
      }
    },
    []
  );

  return {
    submit,
    status,
    txHash,
    error,
  };
}
