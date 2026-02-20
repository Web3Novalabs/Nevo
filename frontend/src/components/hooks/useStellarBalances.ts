"use client";

import { useState, useEffect } from "react";

export interface StellarBalance {
    asset: "XLM" | "USDC" | string;
    balance: string;
}

export interface StellarBalancesState {
    balances: StellarBalance[];
    isLoading: boolean;
    error: string | null;
}

const HORIZON_URL = "https://horizon.stellar.org";

// USDC issuer on Stellar mainnet (Centre / Circle)
const USDC_ISSUER = "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN";

/**
 * Fetch XLM and USDC balances for a given Stellar public key from the
 * Horizon REST API.  No extra SDK is required – a plain `fetch` call is
 * sufficient and keeps the bundle lean.
 *
 * Returns `isLoading: true` while the request is in-flight and re-fetches
 * automatically whenever `publicKey` changes.
 */
export function useStellarBalances(
    publicKey: string | null
): StellarBalancesState {
    const [balances, setBalances] = useState<StellarBalance[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!publicKey) {
            setBalances([]);
            setError(null);
            setIsLoading(false);
            return;
        }

        let cancelled = false;
        setIsLoading(true);
        setError(null);

        (async () => {
            try {
                const res = await fetch(
                    `${HORIZON_URL}/accounts/${publicKey}`
                );

                if (!res.ok) {
                    if (res.status === 404) {
                        // Account not yet funded on the network
                        if (!cancelled) {
                            setBalances([{ asset: "XLM", balance: "0" }]);
                            setError(null);
                        }
                        return;
                    }
                    throw new Error(`Horizon error ${res.status}`);
                }

                const data = await res.json();

                const parsed: StellarBalance[] = (
                    data.balances as Array<{
                        asset_type: string;
                        asset_code?: string;
                        asset_issuer?: string;
                        balance: string;
                    }>
                )
                    .filter((b) => {
                        // Include native XLM always
                        if (b.asset_type === "native") return true;
                        // Include USDC (credit_alphanum4 / alphanum12) from the canonical issuer
                        if (
                            b.asset_code === "USDC" &&
                            b.asset_issuer === USDC_ISSUER
                        )
                            return true;
                        return false;
                    })
                    .map((b) => ({
                        asset: b.asset_type === "native" ? "XLM" : (b.asset_code as string),
                        // Round to 4 decimal places for display
                        balance: parseFloat(b.balance).toFixed(4),
                    }));

                if (!cancelled) {
                    setBalances(parsed);
                }
            } catch (err) {
                if (!cancelled) {
                    setError(
                        err instanceof Error ? err.message : "Failed to fetch balances"
                    );
                }
            } finally {
                if (!cancelled) setIsLoading(false);
            }
        })();

        return () => {
            cancelled = true;
        };
    }, [publicKey]);

    return { balances, isLoading, error };
}
