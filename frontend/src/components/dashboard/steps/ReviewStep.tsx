"use client";

import { FormData } from "../CreatePoolStepper";

interface ReviewStepProps {
  formData: FormData;
}

function ReviewField({ label, value }: { label: string; value: string }) {
  return (
    <div className="space-y-1">
      <p className="text-xs font-semibold uppercase tracking-widest text-slate-500">{label}</p>
      <p className="text-sm text-white break-words">
        {value || <span className="italic text-slate-500">Not provided</span>}
      </p>
    </div>
  );
}

export function ReviewStep({ formData }: ReviewStepProps) {
  const walletPreview = formData.beneficiaryWallet
    ? `${formData.beneficiaryWallet.slice(0, 8)}…${formData.beneficiaryWallet.slice(-6)}`
    : "";

  return (
    <div className="space-y-6">
      <div className="space-y-1">
        <h2 className="text-2xl font-bold text-white">Review & Submit</h2>
        <p className="text-sm text-slate-400">
          Double-check everything before creating your pool on the Stellar network.
        </p>
      </div>

      {/* Basic Info Card */}
      <div className="rounded-xl border border-slate-700/60 bg-slate-900/60 p-5 space-y-5">
        <div className="flex items-center gap-2">
          <div className="flex h-7 w-7 items-center justify-center rounded-full bg-emerald-500/20 text-xs font-bold text-emerald-400">
            1
          </div>
          <h3 className="text-sm font-semibold text-slate-200">Basic Information</h3>
        </div>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <ReviewField label="Pool Name" value={formData.poolName} />
          <ReviewField label="Category" value={formData.category} />
          <div className="sm:col-span-2">
            <ReviewField label="Description" value={formData.description} />
          </div>
          <ReviewField
            label="End Date"
            value={
              formData.endDate
                ? new Date(formData.endDate).toLocaleDateString("en-US", {
                    year: "numeric",
                    month: "long",
                    day: "numeric",
                  })
                : ""
            }
          />
        </div>
      </div>

      {/* Financials Card */}
      <div className="rounded-xl border border-slate-700/60 bg-slate-900/60 p-5 space-y-5">
        <div className="flex items-center gap-2">
          <div className="flex h-7 w-7 items-center justify-center rounded-full bg-emerald-500/20 text-xs font-bold text-emerald-400">
            2
          </div>
          <h3 className="text-sm font-semibold text-slate-200">Financial Details</h3>
        </div>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <ReviewField
            label="Funding Goal"
            value={formData.fundingGoal ? `${Number(formData.fundingGoal).toLocaleString()} XLM` : ""}
          />
          <ReviewField
            label="Min. Contribution"
            value={formData.minContribution ? `${Number(formData.minContribution).toLocaleString()} XLM` : "None"}
          />
          <ReviewField
            label="Visibility"
            value={formData.visibility}
          />
          <ReviewField
            label="Beneficiary Wallet"
            value={walletPreview}
          />
        </div>
      </div>

      {/* Confirmation note */}
      <div className="flex items-start gap-3 rounded-lg border border-emerald-500/20 bg-emerald-500/5 px-4 py-3">
        <svg className="mt-0.5 h-4 w-4 shrink-0 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p className="text-xs leading-relaxed text-emerald-300/80">
          By submitting, you confirm that all information is accurate. The pool will be
          registered on the Stellar blockchain and cannot be altered after creation.
        </p>
      </div>
    </div>
  );
}
