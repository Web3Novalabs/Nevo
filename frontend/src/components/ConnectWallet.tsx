"use client";

import { Loader2, Wallet, LogOut, ChevronDown } from "lucide-react";
import { useWallet } from "./hooks/useWallet";
import { useStellarBalances } from "./hooks/useStellarBalances";
import { cn } from "@/lib/utils";
import { useState, useRef, useEffect } from "react";

/** Shorten a Stellar public key to "GABCD…WXYZ" format. */
function truncateKey(key: string): string {
  if (key.length <= 12) return key;
  return `${key.slice(0, 4)}…${key.slice(-4)}`;
}

export default function ConnectWallet() {
  const { publicKey, isConnected, isLoading, connect, disconnect } =
    useWallet();
  const { balances, isLoading: balancesLoading } =
    useStellarBalances(publicKey);

  const [dropdownOpen, setDropdownOpen] = useState<boolean>(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown on outside click
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(e.target as Node)
      ) {
        setDropdownOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  if (isLoading) {
    return (
      <div className="flex items-center gap-2 px-4 py-2 text-slate-400">
        <Loader2 size={16} className="animate-spin" />
        <span className="text-sm">Connecting…</span>
      </div>
    );
  }

  if (!isConnected || !publicKey) {
    return (
      <button
        onClick={connect}
        className="flex items-center gap-2 bg-transparent text-[#50C878] border border-[#50C878] hover:bg-[#50C878]/10 px-5 py-2 rounded-lg transition-colors font-medium text-sm"
      >
        <Wallet size={15} />
        Connect Wallet
      </button>
    );
  }

  const xlm = balances.find((b) => b.asset === "XLM");
  const usdc = balances.find((b) => b.asset === "USDC");

  return (
    <div className="relative" ref={dropdownRef}>
      {/* Trigger button */}
      <button
        onClick={() => setDropdownOpen((prev: boolean) => !prev)}
        aria-haspopup="true"
        aria-expanded={dropdownOpen}
        className={cn(
          "flex items-center gap-2 rounded-lg border border-[#50C878]/50 bg-[#1E293B] px-4 py-2 text-sm transition-colors hover:border-[#50C878]",
          dropdownOpen && "border-[#50C878]"
        )}
      >
        {/* Wallet icon */}
        <span className="flex h-6 w-6 items-center justify-center rounded-full bg-[#50C878]/15 text-[#50C878]">
          <Wallet size={13} />
        </span>

        {/* Truncated key */}
        <span className="font-mono text-white" title={publicKey}>
          {truncateKey(publicKey)}
        </span>

        {/* Balance pill */}
        {balancesLoading ? (
          <Loader2 size={12} className="animate-spin text-slate-400" />
        ) : xlm ? (
          <span className="rounded-full bg-[#50C878]/10 px-2 py-0.5 text-xs font-medium text-[#50C878]">
            {xlm.balance} XLM
          </span>
        ) : null}

        <ChevronDown
          size={14}
          className={cn(
            "text-slate-400 transition-transform",
            dropdownOpen && "rotate-180"
          )}
        />
      </button>

      {/* Dropdown */}
      {dropdownOpen && (
        <div className="absolute right-0 mt-2 w-64 rounded-xl border border-slate-700 bg-[#0F172A] p-3 shadow-2xl z-50">
          {/* Address */}
          <div className="mb-3 rounded-lg bg-slate-800/60 px-3 py-2">
            <p className="mb-0.5 text-xs text-slate-500">Wallet Address</p>
            <p className="break-all font-mono text-xs text-slate-300">
              {publicKey}
            </p>
          </div>

          {/* Balances */}
          <div className="mb-3 space-y-2">
            <p className="text-xs font-medium text-slate-500 uppercase tracking-wider">
              Balances
            </p>
            {balancesLoading ? (
              <div className="flex items-center gap-2 text-slate-400">
                <Loader2 size={13} className="animate-spin" />
                <span className="text-xs">Fetching balances…</span>
              </div>
            ) : (
              <>
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
                  <p className="text-xs text-slate-500">No USDC trustline</p>
                )}
              </>
            )}
          </div>

          {/* Disconnect */}
          <button
            onClick={async () => {
              setDropdownOpen(false);
              await disconnect();
            }}
            className="flex w-full items-center justify-center gap-2 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-sm font-medium text-red-400 transition-colors hover:bg-red-500/20"
          >
            <LogOut size={14} />
            Disconnect
          </button>
        </div>
      )}
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
    <div className="flex items-center justify-between rounded-lg bg-slate-800/40 px-3 py-2">
      <span className={cn("text-xs font-semibold", color)}>{label}</span>
      <span className="font-mono text-xs text-white">{value}</span>
    </div>
  );
}
