"use client";

import { useState } from "react";
import { DonationModal } from "@/components";
import ConnectWallet from "@/components/ConnectWallet";

export default function ExampleDonationPage() {
  const [isModalOpen, setIsModalOpen] = useState(false);

  return (
    <div className="min-h-screen bg-slate-50 dark:bg-slate-900 p-8">
      <div className="max-w-2xl mx-auto space-y-8">
        <div className="text-center space-y-4">
          <h1 className="text-4xl font-bold text-slate-900 dark:text-white">
            Donation Flow Example
          </h1>
          <p className="text-slate-600 dark:text-slate-400">
            Test the complete donation transaction and receipt flow
          </p>
        </div>

        <div className="bg-white dark:bg-slate-800 rounded-2xl p-8 space-y-6 border border-slate-200 dark:border-slate-700">
          <div className="space-y-4">
            <h2 className="text-2xl font-bold text-slate-900 dark:text-white">
              Step 1: Connect Wallet
            </h2>
            <ConnectWallet />
          </div>

          <div className="space-y-4">
            <h2 className="text-2xl font-bold text-slate-900 dark:text-white">
              Step 2: Make a Donation
            </h2>
            <button
              onClick={() => setIsModalOpen(true)}
              className="w-full py-4 text-lg font-semibold rounded-xl bg-gradient-to-r from-blue-600 to-cyan-500 hover:from-blue-700 hover:to-cyan-600 text-white shadow-lg shadow-blue-500/25 transition-all"
            >
              Open Donation Modal
            </button>
          </div>

          <div className="bg-slate-50 dark:bg-slate-900/50 p-4 rounded-xl border border-slate-200 dark:border-slate-700">
            <h3 className="font-semibold text-slate-900 dark:text-white mb-2">
              What happens next:
            </h3>
            <ol className="list-decimal list-inside space-y-2 text-sm text-slate-600 dark:text-slate-400">
              <li>Select asset (XLM or USDC)</li>
              <li>Enter donation amount</li>
              <li>Review fees and total</li>
              <li>Sign transaction with wallet</li>
              <li>View receipt with transaction details</li>
              <li>Share or download receipt</li>
            </ol>
          </div>
        </div>
      </div>

      <DonationModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        poolTitle="Example Crowdfunding Pool"
        poolAddress="GCZYLNGU4CA5NAWBAVTHMZH4JKXRCN3XWWIL3KIJMZ6QDMRKFZYQUD7Z"
      />
    </div>
  );
}
