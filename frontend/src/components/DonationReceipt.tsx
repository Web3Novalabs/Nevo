"use client";

import React, { useRef } from "react";
import { X, Download, Share2, CheckCircle, ExternalLink } from "lucide-react";

interface DonationReceiptProps {
  isOpen: boolean;
  onClose: () => void;
  poolTitle: string;
  amount: string;
  asset: "XLM" | "USDC";
  transactionHash: string;
  timestamp: Date;
  donorAddress: string;
}

export const DonationReceipt: React.FC<DonationReceiptProps> = ({
  isOpen,
  onClose,
  poolTitle,
  amount,
  asset,
  transactionHash,
  timestamp,
  donorAddress,
}) => {
  const receiptRef = useRef<HTMLDivElement>(null);

  if (!isOpen) return null;

  const shortHash = `${transactionHash.slice(0, 8)}...${transactionHash.slice(-8)}`;
  const shortAddress = `${donorAddress.slice(0, 8)}...${donorAddress.slice(-8)}`;
  const stellarExpertUrl = `https://stellar.expert/explorer/public/tx/${transactionHash}`;

  const handleShare = async () => {
    const shareData = {
      title: "Donation Receipt",
      text: `I just donated ${amount} ${asset} to ${poolTitle}!`,
      url: stellarExpertUrl,
    };

    if (navigator.share) {
      try {
        await navigator.share(shareData);
      } catch (err) {
        console.log("Share cancelled");
      }
    } else {
      await navigator.clipboard.writeText(
        `${shareData.text}\n${shareData.url}`
      );
      alert("Link copied to clipboard!");
    }
  };

  const handleDownload = async () => {
    if (!receiptRef.current) return;

    const html2canvas = (await import("html2canvas")).default;
    const canvas = await html2canvas(receiptRef.current, {
      backgroundColor: "#0F172A",
      scale: 2,
    });

    const link = document.createElement("a");
    link.download = `donation-receipt-${transactionHash.slice(0, 8)}.png`;
    link.href = canvas.toDataURL();
    link.click();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/60 backdrop-blur-sm">
      <div className="bg-white dark:bg-[#0F172A] border border-slate-200 dark:border-slate-800 w-full max-w-md rounded-3xl shadow-2xl overflow-hidden">
        <div className="flex justify-between items-center p-6 border-b border-slate-200 dark:border-slate-800/60">
          <h2 className="text-xl font-bold text-slate-900 dark:text-white">
            Donation Receipt
          </h2>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-slate-600 dark:hover:text-white transition-colors p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800"
          >
            <X size={20} />
          </button>
        </div>

        <div ref={receiptRef} className="p-6 space-y-6">
          <div className="flex flex-col items-center text-center space-y-4">
            <div className="w-16 h-16 rounded-full bg-green-100 dark:bg-green-900/30 flex items-center justify-center">
              <CheckCircle className="w-10 h-10 text-green-600 dark:text-green-400" />
            </div>
            <div>
              <h3 className="text-2xl font-bold text-slate-900 dark:text-white">
                {amount} {asset}
              </h3>
              <p className="text-slate-500 dark:text-slate-400 mt-1">
                Successfully donated
              </p>
            </div>
          </div>

          <div className="bg-slate-50 dark:bg-slate-900/50 p-4 rounded-xl space-y-3 border border-slate-100 dark:border-slate-800/80">
            <div className="flex justify-between text-sm">
              <span className="text-slate-500 dark:text-slate-400">Pool</span>
              <span className="text-slate-900 dark:text-white font-medium text-right max-w-[200px] truncate">
                {poolTitle}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-slate-500 dark:text-slate-400">From</span>
              <span className="text-slate-900 dark:text-white font-mono text-xs">
                {shortAddress}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-slate-500 dark:text-slate-400">Date</span>
              <span className="text-slate-900 dark:text-white">
                {timestamp.toLocaleDateString()} {timestamp.toLocaleTimeString()}
              </span>
            </div>
            <div className="pt-3 border-t border-slate-200 dark:border-slate-700/50">
              <div className="flex justify-between items-center text-sm">
                <span className="text-slate-500 dark:text-slate-400">
                  Transaction
                </span>
                <a
                  href={stellarExpertUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-600 dark:text-blue-400 hover:underline font-mono text-xs flex items-center gap-1"
                >
                  {shortHash}
                  <ExternalLink size={12} />
                </a>
              </div>
            </div>
          </div>
        </div>

        <div className="p-6 border-t border-slate-200 dark:border-slate-800/60 bg-slate-50 dark:bg-slate-900/20 space-y-3">
          <button
            onClick={handleShare}
            className="w-full py-3 text-base font-semibold rounded-xl bg-gradient-to-r from-blue-600 to-cyan-500 hover:from-blue-700 hover:to-cyan-600 text-white shadow-lg shadow-blue-500/25 transition-all flex items-center justify-center gap-2"
          >
            <Share2 size={18} />
            Share Donation
          </button>
          <button
            onClick={handleDownload}
            className="w-full py-3 text-base font-semibold rounded-xl border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-all flex items-center justify-center gap-2"
          >
            <Download size={18} />
            Download Receipt
          </button>
        </div>
      </div>
    </div>
  );
};
