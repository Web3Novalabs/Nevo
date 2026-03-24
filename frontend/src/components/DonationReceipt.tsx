"use client";

import React from "react";
import { Check, Download, Share2, X } from "lucide-react";

interface DonationReceiptProps {
  isOpen: boolean;
  onClose: () => void;
  poolTitle: string;
  amount: string;
  asset: "XLM" | "USDC";
  txHash: string;
  timestamp: Date;
}

export const DonationReceipt: React.FC<DonationReceiptProps> = ({
  isOpen,
  onClose,
  poolTitle,
  amount,
  asset,
  txHash,
  timestamp,
}) => {
  if (!isOpen) return null;

  const shareableUrl = `https://stellar.expert/explorer/public/tx/${txHash}`;

  const handleShare = async () => {
    const shareData = {
      title: "Donation Receipt",
      text: `I donated ${amount} ${asset} to ${poolTitle}!`,
      url: shareableUrl,
    };

    if (navigator.share) {
      await navigator.share(shareData);
    } else {
      await navigator.clipboard.writeText(
        `${shareData.text}\n${shareData.url}`
      );
      alert("Link copied to clipboard!");
    }
  };

  const handleDownload = () => {
    const canvas = document.createElement("canvas");
    canvas.width = 800;
    canvas.height = 600;
    const ctx = canvas.getContext("2d");

    if (ctx) {
      ctx.fillStyle = "#0F172A";
      ctx.fillRect(0, 0, 800, 600);

      ctx.fillStyle = "#10B981";
      ctx.beginPath();
      ctx.arc(400, 150, 50, 0, 2 * Math.PI);
      ctx.fill();

      ctx.fillStyle = "#FFFFFF";
      ctx.font = "bold 32px sans-serif";
      ctx.textAlign = "center";
      ctx.fillText("Donation Successful!", 400, 250);

      ctx.font = "24px sans-serif";
      ctx.fillText(`${amount} ${asset}`, 400, 300);

      ctx.font = "18px sans-serif";
      ctx.fillStyle = "#94A3B8";
      ctx.fillText(poolTitle, 400, 340);
      ctx.fillText(timestamp.toLocaleDateString(), 400, 380);

      ctx.font = "14px monospace";
      ctx.fillText(txHash.slice(0, 32) + "...", 400, 450);

      canvas.toBlob((blob) => {
        if (blob) {
          const url = URL.createObjectURL(blob);
          const a = document.createElement("a");
          a.href = url;
          a.download = `donation-receipt-${txHash.slice(0, 8)}.png`;
          a.click();
          URL.revokeObjectURL(url);
        }
      });
    }
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

        <div className="p-6 space-y-6">
          <div className="flex flex-col items-center space-y-4">
            <div className="w-20 h-20 rounded-full bg-green-500/10 flex items-center justify-center">
              <Check size={40} className="text-green-500" />
            </div>
            <div className="text-center">
              <h3 className="text-2xl font-bold text-slate-900 dark:text-white">
                {amount} {asset}
              </h3>
              <p className="text-slate-500 dark:text-slate-400 mt-1">
                Donated Successfully
              </p>
            </div>
          </div>

          <div className="bg-slate-50 dark:bg-slate-900/50 p-4 rounded-xl space-y-3 text-sm border border-slate-100 dark:border-slate-800/80">
            <div className="flex justify-between">
              <span className="text-slate-500 dark:text-slate-400">Pool</span>
              <span className="text-slate-900 dark:text-white font-medium">
                {poolTitle}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-slate-500 dark:text-slate-400">Date</span>
              <span className="text-slate-900 dark:text-white font-medium">
                {timestamp.toLocaleDateString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-slate-500 dark:text-slate-400">Time</span>
              <span className="text-slate-900 dark:text-white font-medium">
                {timestamp.toLocaleTimeString()}
              </span>
            </div>
            <div className="flex justify-between items-start">
              <span className="text-slate-500 dark:text-slate-400">
                Transaction
              </span>
              <a
                href={shareableUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-500 hover:text-blue-600 font-mono text-xs break-all text-right max-w-[200px]"
              >
                {txHash.slice(0, 16)}...
              </a>
            </div>
          </div>

          <div className="flex space-x-3">
            <button
              onClick={handleShare}
              className="flex-1 flex items-center justify-center space-x-2 py-3 px-4 rounded-xl border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors text-slate-700 dark:text-slate-300"
            >
              <Share2 size={18} />
              <span>Share</span>
            </button>
            <button
              onClick={handleDownload}
              className="flex-1 flex items-center justify-center space-x-2 py-3 px-4 rounded-xl border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors text-slate-700 dark:text-slate-300"
            >
              <Download size={18} />
              <span>Download</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
