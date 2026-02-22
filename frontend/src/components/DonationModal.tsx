"use client";

import React, { useState } from "react";
import { X, DollarSign, Wallet } from "lucide-react";
import { executeDonation } from "../lib/stellar";
import { getPublicKey } from "../app/stellar-wallets-kit";
import { DonationReceipt } from "./DonationReceipt";

interface DonationModalProps {
  isOpen: boolean;
  onClose: () => void;
  poolTitle: string;
  poolId: string;
  contractId: string;
}

type Asset = "XLM" | "USDC";

export const DonationModal: React.FC<DonationModalProps> = ({
  isOpen,
  onClose,
  poolTitle,
  poolId,
  contractId,
}) => {
  const [amount, setAmount] = useState<string>("");
  const [asset, setAsset] = useState<Asset>("XLM");
  const [isProcessing, setIsProcessing] = useState(false);
  const [showReceipt, setShowReceipt] = useState(false);
  const [txHash, setTxHash] = useState("");
  const [txTimestamp, setTxTimestamp] = useState(new Date());

  if (!isOpen) return null;

  const quickAmounts = ["10", "50", "100"];
  const estimatedFee = asset === "XLM" ? "0.00001 XLM" : "0.00 USDC (Covered)";
  const totalDeduction = amount
    ? asset === "XLM"
      ? (parseFloat(amount) + 0.00001).toFixed(5) + " XLM"
      : parseFloat(amount).toFixed(2) + " USDC"
    : "0.00 " + asset;

  const handleBackdropClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  const handleDonate = async () => {
    setIsProcessing(true);
    try {
      const donor = await getPublicKey();
      if (!donor) {
        alert("Please connect your wallet first");
        return;
      }

      const result = await executeDonation({
        poolId,
        donor,
        asset,
        amount,
        contractId,
      });

      if (result.success) {
        setTxHash(result.hash);
        setTxTimestamp(new Date());
        setShowReceipt(true);
      }
    } catch (error) {
      console.error("Donation failed:", error);
      alert("Transaction failed. Please try again.");
    } finally {
      setIsProcessing(false);
    }
  };

  const handleCloseReceipt = () => {
    setShowReceipt(false);
    onClose();
  };

  return (
    <>
      <DonationReceipt
        isOpen={showReceipt}
        onClose={handleCloseReceipt}
        poolTitle={poolTitle}
        amount={amount}
        asset={asset}
        txHash={txHash}
        timestamp={txTimestamp}
      />
      <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/60 backdrop-blur-sm transition-opacity"
      onClick={handleBackdropClick}
    >
      <div
        className="bg-white dark:bg-[#0F172A] border border-slate-200 dark:border-slate-800 w-full max-w-md rounded-3xl shadow-2xl overflow-hidden flex flex-col transform transition-all"
        role="dialog"
        aria-modal="true"
      >
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-slate-200 dark:border-slate-800/60">
          <div>
            <h2 className="text-xl font-bold text-slate-900 dark:text-white">
              Contribute to Pool
            </h2>
            <p className="text-sm text-slate-500 dark:text-slate-400 mt-1 line-clamp-1">
              {poolTitle}
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-slate-600 dark:hover:text-white transition-colors p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800"
          >
            <X size={20} />
          </button>
        </div>

        {/* Body */}
        <div className="p-6 space-y-6">
          {/* Asset Selector */}
          <div className="space-y-3">
            <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
              Select Asset
            </label>
            <div className="grid grid-cols-2 gap-3">
              {(["XLM", "USDC"] as Asset[]).map((a) => (
                <button
                  key={a}
                  onClick={() => setAsset(a)}
                  className={`flex items-center justify-center space-x-2 py-3 px-4 rounded-xl border transition-all ${asset === a
                      ? "border-blue-500 bg-blue-50/50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-400"
                      : "border-slate-200 dark:border-slate-700 hover:border-slate-300 dark:hover:border-slate-600 text-slate-600 dark:text-slate-400"
                    }`}
                >
                  <Wallet size={16} />
                  <span className="font-semibold">{a}</span>
                </button>
              ))}
            </div>
          </div>

          {/* Amount Input */}
          <div className="space-y-3">
            <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
              Donation Amount
            </label>
            <div className="relative flex items-center">
              <div className="absolute left-4 flex items-center pointer-events-none">
                <DollarSign size={18} className="text-slate-400" />
              </div>
              <input
                type="number"
                placeholder="0.00"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                className="w-full pl-11 pr-16 py-4 text-2xl font-semibold bg-slate-50 dark:bg-slate-900/50 border border-slate-200 dark:border-slate-700 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all text-slate-900 dark:text-white placeholder-slate-400"
              />
              <div className="absolute right-4 flex items-center pointer-events-none">
                <span className="text-slate-500 dark:text-slate-400 font-medium">
                  {asset}
                </span>
              </div>
            </div>
          </div>

          {/* Quick Select Buttons */}
          <div className="flex space-x-3">
            {quickAmounts.map((qAmount) => (
              <button
                key={qAmount}
                onClick={() => setAmount(qAmount)}
                className="flex-1 py-2.5 text-sm font-medium rounded-lg border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors"
              >
                {qAmount} {asset}
              </button>
            ))}
          </div>

          {/* Fee Breakdown */}
          <div className="bg-slate-50 dark:bg-slate-900/50 p-4 rounded-xl space-y-3 text-sm border border-slate-100 dark:border-slate-800/80">
            <div className="flex justify-between text-slate-500 dark:text-slate-400">
              <span>Donation Amount</span>
              <span>
                {amount ? amount : "0"} {asset}
              </span>
            </div>
            <div className="flex justify-between text-slate-500 dark:text-slate-400">
              <span>Estimated Network Fee</span>
              <span>{estimatedFee}</span>
            </div>
            <div className="pt-3 border-t border-slate-200 dark:border-slate-700/50 flex justify-between font-semibold text-slate-900 dark:text-white">
              <span>Total Deduction</span>
              <span>{totalDeduction}</span>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="p-6 border-t border-slate-200 dark:border-slate-800/60 bg-slate-50 dark:bg-slate-900/20">
          <button
            onClick={handleDonate}
            disabled={!amount || parseFloat(amount) <= 0 || isProcessing}
            className="w-full py-4 text-lg font-semibold rounded-xl bg-gradient-to-r from-blue-600 to-cyan-500 hover:from-blue-700 hover:to-cyan-600 text-white disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-blue-500/25 transition-all"
          >
            {isProcessing ? "Processing..." : "Confirm Donation"}
          </button>
        </div>
      </div>
    </div>
    </>
  );
};
