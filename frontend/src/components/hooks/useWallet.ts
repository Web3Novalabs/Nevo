"use client";

import { useState, useCallback, useEffect } from "react";
import {
    getPublicKey,
    connect,
    disconnect,
} from "@/app/stellar-wallets-kit";

export interface WalletState {
    publicKey: string | null;
    isConnected: boolean;
    isLoading: boolean;
    connect: () => Promise<void>;
    disconnect: () => Promise<void>;
}

/**
 * Centralised wallet state hook.
 *
 * - Reads the persisted wallet selection from localStorage on mount.
 * - Exposes `connect` / `disconnect` helpers that keep React state in sync.
 * - All other components should consume this hook instead of calling the
 *   stellar-wallets-kit helpers directly.
 */
export function useWallet(): WalletState {
    const [publicKey, setPublicKey] = useState<string | null>(null);
    const [isLoading, setIsLoading] = useState(true);

    // Restore session on mount
    useEffect(() => {
        let cancelled = false;
        (async () => {
            try {
                const key = await getPublicKey();
                if (!cancelled) setPublicKey(key);
            } catch {
                // Wallet not available or not connected – that's fine
            } finally {
                if (!cancelled) setIsLoading(false);
            }
        })();
        return () => {
            cancelled = true;
        };
    }, []);

    const handleConnect = useCallback(async () => {
        await connect(async () => {
            const key = await getPublicKey();
            setPublicKey(key);
        });
    }, []);

    const handleDisconnect = useCallback(async () => {
        await disconnect(async () => {
            setPublicKey(null);
        });
    }, []);

    return {
        publicKey,
        isConnected: publicKey !== null,
        isLoading,
        connect: handleConnect,
        disconnect: handleDisconnect,
    };
}
