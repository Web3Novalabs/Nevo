"use client";

import { FormData } from "../CreatePoolStepper";

interface FinancialsStepProps {
  formData: FormData;
  onChange: (updates: Partial<FormData>) => void;
}

export function FinancialsStep({ formData, onChange }: FinancialsStepProps) {
  return (
    <div className="space-y-6">
      <div className="space-y-1">
        <h2 className="text-2xl font-bold text-white">Financial Details</h2>
        <p className="text-sm text-slate-400">
          Set your funding targets and specify where funds will be sent.
        </p>
      </div>

      {/* Funding Goal */}
      <div className="space-y-2">
        <label htmlFor="fundingGoal" className="block text-sm font-medium text-slate-300">
          Funding Goal (XLM) <span className="text-emerald-400">*</span>
        </label>
        <div className="relative">
          <input
            id="fundingGoal"
            type="number"
            min={1}
            value={formData.fundingGoal}
            onChange={(e) => onChange({ fundingGoal: e.target.value })}
            placeholder="5000"
            className="w-full rounded-lg border border-slate-700/80 bg-slate-900/70 py-3 pl-4 pr-16 text-sm text-white placeholder:text-slate-500 outline-none transition-all duration-200 focus:border-emerald-500/70 focus:ring-2 focus:ring-emerald-500/20"
          />
          <span className="pointer-events-none absolute inset-y-0 right-4 flex items-center text-sm font-semibold text-slate-400">
            XLM
          </span>
        </div>
      </div>

      {/* Minimum Contribution */}
      <div className="space-y-2">
        <label htmlFor="minContribution" className="block text-sm font-medium text-slate-300">
          Minimum Contribution (XLM)
          <span className="ml-2 text-xs text-slate-500">Optional</span>
        </label>
        <div className="relative">
          <input
            id="minContribution"
            type="number"
            min={0}
            value={formData.minContribution}
            onChange={(e) => onChange({ minContribution: e.target.value })}
            placeholder="10"
            className="w-full rounded-lg border border-slate-700/80 bg-slate-900/70 py-3 pl-4 pr-16 text-sm text-white placeholder:text-slate-500 outline-none transition-all duration-200 focus:border-emerald-500/70 focus:ring-2 focus:ring-emerald-500/20"
          />
          <span className="pointer-events-none absolute inset-y-0 right-4 flex items-center text-sm font-semibold text-slate-400">
            XLM
          </span>
        </div>
      </div>

      {/* Beneficiary Wallet */}
      <div className="space-y-2">
        <label htmlFor="beneficiaryWallet" className="block text-sm font-medium text-slate-300">
          Beneficiary Wallet Address <span className="text-emerald-400">*</span>
        </label>
        <input
          id="beneficiaryWallet"
          type="text"
          value={formData.beneficiaryWallet}
          onChange={(e) => onChange({ beneficiaryWallet: e.target.value })}
          placeholder="G... (Stellar public key)"
          className="w-full rounded-lg border border-slate-700/80 bg-slate-900/70 px-4 py-3 font-mono text-sm text-white placeholder:text-slate-500 placeholder:font-sans outline-none transition-all duration-200 focus:border-emerald-500/70 focus:ring-2 focus:ring-emerald-500/20"
        />
        <p className="text-xs text-slate-500">
          Funds will be disbursed to this Stellar address once the pool ends.
        </p>
      </div>

      {/* Visibility Toggle */}
      <div className="space-y-3">
        <span className="block text-sm font-medium text-slate-300">Visibility</span>
        <div className="flex gap-3">
          {(["Public", "Private"] as const).map((option) => (
            <button
              key={option}
              type="button"
              onClick={() => onChange({ visibility: option })}
              className={`flex flex-1 items-center justify-center gap-2 rounded-lg border px-4 py-3 text-sm font-medium transition-all duration-200 ${
                formData.visibility === option
                  ? "border-emerald-500/60 bg-emerald-500/15 text-emerald-400 shadow-[inset_0_0_0_1px_rgba(16,185,129,0.4)]"
                  : "border-slate-700/80 bg-slate-900/70 text-slate-400 hover:border-slate-600 hover:text-slate-200"
              }`}
            >
              {option === "Public" ? (
                <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h.5A2.5 2.5 0 0021 5.5V3.935M15 3.935V5.5A2.5 2.5 0 0112.5 8h-.5a2 2 0 00-2 2 2 2 0 10-4 0 2 2 0 00-2 2h-.5A2.5 2.5 0 013 5.5V3.935" />
                </svg>
              ) : (
                <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              )}
              {option}
            </button>
          ))}
        </div>
        <p className="text-xs text-slate-500">
          {formData.visibility === "Public"
            ? "Anyone can discover and contribute to this pool."
            : "Only people with the direct link can contribute."}
        </p>
      </div>
    </div>
  );
}
