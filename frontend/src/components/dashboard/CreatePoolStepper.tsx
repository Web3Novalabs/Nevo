"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { BasicInfoStep } from "./steps/BasicInfoStep";
import { FinancialsStep } from "./steps/FinancialsStep";
import { ReviewStep } from "./steps/ReviewStep";
import { cn } from "@/lib/utils";

export interface FormData {
  // Step 1
  poolName: string;
  category: string;
  description: string;
  endDate: string;
  // Step 2
  fundingGoal: string;
  minContribution: string;
  beneficiaryWallet: string;
  visibility: "Public" | "Private";
}

const INITIAL_FORM: FormData = {
  poolName: "",
  category: "",
  description: "",
  endDate: "",
  fundingGoal: "",
  minContribution: "",
  beneficiaryWallet: "",
  visibility: "Public",
};

const STEPS = [
  { id: 1, label: "Basic Info" },
  { id: 2, label: "Financials" },
  { id: 3, label: "Review" },
];

function isStep1Valid(data: FormData) {
  return data.poolName.trim() !== "" && data.category !== "" && data.description.trim() !== "";
}

function isStep2Valid(data: FormData) {
  return data.fundingGoal !== "" && Number(data.fundingGoal) > 0 && data.beneficiaryWallet.trim() !== "";
}

export function CreatePoolStepper() {
  const router = useRouter();
  const [currentStep, setCurrentStep] = useState(1);
  const [formData, setFormData] = useState<FormData>(INITIAL_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [direction, setDirection] = useState<"forward" | "backward">("forward");
  const [animating, setAnimating] = useState(false);

  const updateForm = (updates: Partial<FormData>) => {
    setFormData((prev) => ({ ...prev, ...updates }));
  };

  const canProceed =
    currentStep === 1
      ? isStep1Valid(formData)
      : currentStep === 2
      ? isStep2Valid(formData)
      : true;

  const transitionToStep = (next: number, dir: "forward" | "backward") => {
    setDirection(dir);
    setAnimating(true);
    setTimeout(() => {
      setCurrentStep(next);
      setAnimating(false);
    }, 220);
  };

  const handleNext = () => {
    if (currentStep < 3 && canProceed) transitionToStep(currentStep + 1, "forward");
  };

  const handleBack = () => {
    if (currentStep > 1) transitionToStep(currentStep - 1, "backward");
  };

  const handleSubmit = async () => {
    setSubmitting(true);
    // Simulate async submission
    await new Promise((r) => setTimeout(r, 1500));
    setSubmitting(false);
    router.push("/dashboard/my-pools");
  };

  const progressPercent = ((currentStep - 1) / (STEPS.length - 1)) * 100;

  return (
    <div className="mx-auto max-w-2xl space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-white">Create a New Pool</h1>
        <p className="mt-2 text-slate-400">
          Complete each step to launch your donation pool on the Stellar blockchain.
        </p>
      </div>

      {/* Progress bar + Step indicators */}
      <div className="space-y-4">
        {/* Step circles */}
        <div className="flex items-center">
          {STEPS.map((step, idx) => {
            const isDone = currentStep > step.id;
            const isActive = currentStep === step.id;
            return (
              <div key={step.id} className={cn("flex items-center", idx < STEPS.length - 1 && "flex-1")}>
                <div className="flex flex-col items-center gap-1.5">
                  {/* Circle */}
                  <div
                    className={cn(
                      "flex h-9 w-9 items-center justify-center rounded-full border-2 text-sm font-bold transition-all duration-300",
                      isDone
                        ? "border-emerald-500 bg-emerald-500 text-white shadow-lg shadow-emerald-500/30"
                        : isActive
                        ? "border-emerald-400 bg-emerald-400/10 text-emerald-400 shadow-lg shadow-emerald-400/20"
                        : "border-slate-700 bg-slate-900 text-slate-500"
                    )}
                  >
                    {isDone ? (
                      <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                      </svg>
                    ) : (
                      step.id
                    )}
                  </div>
                  {/* Label */}
                  <span
                    className={cn(
                      "text-xs font-medium transition-colors duration-200",
                      isActive ? "text-emerald-400" : isDone ? "text-emerald-500/70" : "text-slate-500"
                    )}
                  >
                    {step.label}
                  </span>
                </div>

                {/* Connector line */}
                {idx < STEPS.length - 1 && (
                  <div className="relative mx-3 mt-[-20px] h-0.5 flex-1 overflow-hidden rounded-full bg-slate-800">
                    <div
                      className="absolute inset-y-0 left-0 rounded-full bg-gradient-to-r from-emerald-500 to-cyan-500 transition-all duration-500 ease-out"
                      style={{ width: isDone ? "100%" : "0%" }}
                    />
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {/* Thin progress bar */}
        <div className="h-1 w-full overflow-hidden rounded-full bg-slate-800">
          <div
            className="h-full rounded-full bg-gradient-to-r from-emerald-500 to-cyan-400 transition-all duration-500 ease-out shadow-[0_0_8px_rgba(16,185,129,0.5)]"
            style={{ width: `${progressPercent === 0 ? 6 : progressPercent}%` }}
          />
        </div>
        <p className="text-right text-xs text-slate-500">
          Step {currentStep} of {STEPS.length}
        </p>
      </div>

      {/* Step content card */}
      <div
        className={cn(
          "rounded-2xl border border-slate-800/80 bg-slate-900/50 p-6 backdrop-blur-sm transition-all duration-220 lg:p-8",
          animating
            ? direction === "forward"
              ? "translate-y-2 opacity-0"
              : "-translate-y-2 opacity-0"
            : "translate-y-0 opacity-100"
        )}
      >
        {currentStep === 1 && <BasicInfoStep formData={formData} onChange={updateForm} />}
        {currentStep === 2 && <FinancialsStep formData={formData} onChange={updateForm} />}
        {currentStep === 3 && <ReviewStep formData={formData} />}
      </div>

      {/* Navigation footer */}
      <div className="flex items-center justify-between gap-4 pt-2">
        <button
          onClick={handleBack}
          disabled={currentStep === 1}
          className={cn(
            "flex items-center gap-2 rounded-xl border border-slate-700/80 bg-slate-900 px-5 py-3 text-sm font-medium transition-all duration-200",
            currentStep === 1
              ? "cursor-not-allowed opacity-30 text-slate-500"
              : "text-slate-300 hover:border-slate-600 hover:bg-slate-800 hover:text-white"
          )}
        >
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M15 19l-7-7 7-7" />
          </svg>
          Back
        </button>

        {currentStep < 3 ? (
          <button
            onClick={handleNext}
            disabled={!canProceed}
            className={cn(
              "flex items-center gap-2 rounded-xl px-6 py-3 text-sm font-semibold transition-all duration-200",
              canProceed
                ? "bg-gradient-to-r from-emerald-500 to-cyan-500 text-white shadow-lg shadow-emerald-500/30 hover:shadow-emerald-500/50 hover:brightness-110 active:scale-95"
                : "cursor-not-allowed bg-slate-800 text-slate-500"
            )}
          >
            Next
            <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
            </svg>
          </button>
        ) : (
          <button
            onClick={handleSubmit}
            disabled={submitting}
            className="flex items-center gap-2 rounded-xl bg-gradient-to-r from-emerald-500 to-cyan-500 px-6 py-3 text-sm font-semibold text-white shadow-lg shadow-emerald-500/30 transition-all duration-200 hover:shadow-emerald-500/50 hover:brightness-110 active:scale-95 disabled:opacity-60"
          >
            {submitting ? (
              <>
                <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                </svg>
                Creating Pool…
              </>
            ) : (
              <>
                <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                </svg>
                Create Pool
              </>
            )}
          </button>
        )}
      </div>
    </div>
  );
}
