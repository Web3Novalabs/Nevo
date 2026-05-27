"use client";

import { useEffect } from "react";
import { LogOut, Wallet } from "lucide-react";
import { toast } from "sonner";
import { useWalletStore } from "@/store";

export default function ConnectWallet() {
  const { publicKey, balances, loading, initialize, connectWallet, disconnectWallet } =
    useWalletStore();

  useEffect(() => {
    initialize();
  }, [initialize]);

  async function handleConnect() {
    await connectWallet(() => toast.success("Wallet connected successfully!"));
  }

  async function handleDisconnect() {
    await disconnectWallet();
    toast.info("Wallet disconnected.");
  }

  const truncateKey = (key: string) => `${key.slice(0, 4)}...${key.slice(-4)}`;

  if (loading) {
    return <div className="animate-pulse bg-slate-700/50 h-10 w-32 rounded-lg" />;
  }

  return (
    <div id="connect-wrap" className="flex items-center gap-4" aria-live="polite">
      {publicKey ? (
        <div className="flex items-center gap-3 bg-slate-800/50 border border-slate-700 rounded-full pl-3 pr-1 py-1">
          <div className="flex flex-col items-start px-2">
            <span className="text-[10px] text-slate-400 font-medium uppercase tracking-wider">
              {truncateKey(publicKey)}
            </span>
            <div className="flex gap-2 text-xs font-bold text-white">
              <span>{parseFloat(balances?.XLM || "0").toFixed(2)} XLM</span>
              <span className="text-slate-500">|</span>
              <span className="text-emerald-400">
                {parseFloat(balances?.USDC || "0").toFixed(2)} USDC
              </span>
            </div>
          </div>
          <button
            onClick={handleDisconnect}
            className="p-2 hover:bg-red-500/20 text-slate-400 hover:text-red-400 rounded-full transition-colors"
            title="Disconnect Wallet"
          >
            <LogOut size={16} />
          </button>
        </div>
      ) : (
        <button
          onClick={handleConnect}
          className="flex items-center gap-2 bg-[#50C878] hover:bg-[#45b76b] text-white px-5 py-2 rounded-lg transition-all duration-300 font-semibold shadow-[0_0_15px_rgba(80,200,120,0.3)] hover:shadow-[0_0_20px_rgba(80,200,120,0.5)]"
        >
          <Wallet size={18} />
          Connect Wallet
        </button>
      )}
    </div>
  );
}
