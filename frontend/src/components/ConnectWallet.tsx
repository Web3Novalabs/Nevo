"use client";

import { Loader2, Wallet, LogOut, ChevronDown } from "lucide-react";
import { useWallet } from "./hooks/useWallet";
import { useStellarBalances } from "./hooks/useStellarBalances";
import { cn } from "@/lib/utils";
import { useState, useRef, useEffect } from "react";
import WalletDropdown from "./WalletDropdown";

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
        <WalletDropdown
          publicKey={publicKey}
          balances={balances}
          balancesLoading={balancesLoading}
          onDisconnect={disconnect}
          onClose={() => setDropdownOpen(false)}
        />
      )}
    </div>
  );
}
