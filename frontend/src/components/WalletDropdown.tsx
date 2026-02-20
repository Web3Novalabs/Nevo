"use client";

import { LogOut, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

interface StellarBalance {
    asset: string;
    balance: string;
}

interface WalletDropdownProps {
    publicKey: string;
    balances: StellarBalance[];
    balancesLoading: boolean;
    onDisconnect: () => Promise<void>;
    onClose: () => void;
}

/**
 * Dropdown panel showing detailed wallet information and actions.
 * Extracted from ConnectWallet to keep component sizes manageable.
 */
export default function WalletDropdown({
    publicKey,
    balances,
    balancesLoading,
    onDisconnect,
    onClose,
}: WalletDropdownProps) {
    const xlm = balances.find((b) => b.asset === "XLM");
    const usdc = balances.find((b) => b.asset === "USDC");

    return (
        <div className="absolute right-0 mt-2 w-64 rounded-xl border border-slate-700 bg-[#0F172A] p-3 shadow-2xl z-50">
            {/* Address section */}
            <div className="mb-3 rounded-lg bg-slate-800/60 px-3 py-2">
                <p className="mb-0.5 text-xs text-slate-500 font-medium uppercase tracking-wider">
                    Wallet Address
                </p>
                <p className="break-all font-mono text-xs text-slate-300 leading-relaxed">
                    {publicKey}
                </p>
            </div>

            {/* Balances section */}
            <div className="mb-4 space-y-2">
                <p className="text-xs font-medium text-slate-500 font-medium uppercase tracking-wider">
                    Asset Balances
                </p>
                {balancesLoading ? (
                    <div className="flex items-center gap-2 py-1 text-slate-400">
                        <Loader2 size={13} className="animate-spin" />
                        <span className="text-xs">Updating balances…</span>
                    </div>
                ) : (
                    <div className="space-y-1.5">
                        {xlm && (
                            <BalanceRow
                                label="XLM"
                                value={xlm.balance}
                                color="text-yellow-400"
                            />
                        )}
                        {usdc ? (
                            <BalanceRow
                                label="USDC"
                                value={usdc.balance}
                                color="text-blue-400"
                            />
                        ) : (
                            <p className="px-3 py-1.5 text-xs text-slate-500 italic bg-slate-800/20 rounded-lg">
                                No USDC trustline found
                            </p>
                        )}
                    </div>
                )}
            </div>

            {/* Action section */}
            <button
                onClick={async () => {
                    onClose();
                    await onDisconnect();
                }}
                className="flex w-full items-center justify-center gap-2 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-sm font-semibold text-red-400 transition-all hover:bg-red-500/20 hover:border-red-500/50"
            >
                <LogOut size={14} />
                Disconnect Wallet
            </button>
        </div>
    );
}

function BalanceRow({
    label,
    value,
    color,
}: {
    label: string;
    value: string;
    color: string;
}) {
    return (
        <div className="flex items-center justify-between rounded-lg bg-slate-800/40 px-3 py-2 transition-colors hover:bg-slate-800/60">
            <span className={cn("text-xs font-bold", color)}>{label}</span>
            <span className="font-mono text-xs font-medium text-white">{value}</span>
        </div>
    );
}
