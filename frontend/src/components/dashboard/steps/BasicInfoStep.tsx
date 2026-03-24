"use client";

import { FormData, FormErrors } from "../CreatePoolStepper";
import { cn } from "@/lib/utils";

interface BasicInfoStepProps {
  formData: FormData;
  onChange: (updates: Partial<FormData>) => void;
  errors?: FormErrors;
}

const CATEGORIES = ["Education", "Medical", "Community", "Environment", "Arts", "Other"];

export function BasicInfoStep({ formData, onChange, errors = {} }: BasicInfoStepProps) {
  return (
    <div className="space-y-6">
      <div className="space-y-1">
        <h2 className="text-2xl font-bold text-white">Basic Information</h2>
        <p className="text-sm text-slate-400">
          Tell the community what your donation pool is about.
        </p>
      </div>

      {/* Pool Name */}
      <div className="space-y-2">
        <label htmlFor="poolName" className="block text-sm font-medium text-slate-300">
          Pool Name <span className="text-emerald-400">*</span>
        </label>
        <input
          id="poolName"
          type="text"
          value={formData.poolName}
          onChange={(e) => onChange({ poolName: e.target.value })}
          placeholder="e.g. Community School Rebuild Fund"
          className={cn(
            "w-full rounded-lg border bg-slate-900/70 px-4 py-3 text-sm text-white placeholder:text-slate-500 outline-none transition-all duration-200 focus:ring-2",
            errors.poolName
              ? "border-red-500/70 focus:border-red-500/70 focus:ring-red-500/20"
              : "border-slate-700/80 focus:border-emerald-500/70 focus:ring-emerald-500/20"
          )}
        />
        {errors.poolName && (
          <p className="flex items-center gap-1.5 text-xs text-red-400">
            <svg className="h-3.5 w-3.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            {errors.poolName}
          </p>
        )}
      </div>

      {/* Category */}
      <div className="space-y-2">
        <label htmlFor="category" className="block text-sm font-medium text-slate-300">
          Category <span className="text-emerald-400">*</span>
        </label>
        <div className="relative">
          <select
            id="category"
            value={formData.category}
            onChange={(e) => onChange({ category: e.target.value })}
            className={cn(
              "w-full appearance-none rounded-lg border bg-slate-900/70 px-4 py-3 text-sm text-white outline-none transition-all duration-200 focus:ring-2",
              errors.category
                ? "border-red-500/70 focus:border-red-500/70 focus:ring-red-500/20"
                : "border-slate-700/80 focus:border-emerald-500/70 focus:ring-emerald-500/20"
            )}
          >
            <option value="" disabled>
              Select a category…
            </option>
            {CATEGORIES.map((cat) => (
              <option key={cat} value={cat} className="bg-slate-900">
                {cat}
              </option>
            ))}
          </select>
          <div className="pointer-events-none absolute inset-y-0 right-3 flex items-center">
            <svg className="h-4 w-4 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </div>
        </div>
        {errors.category && (
          <p className="flex items-center gap-1.5 text-xs text-red-400">
            <svg className="h-3.5 w-3.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            {errors.category}
          </p>
        )}
      </div>

      {/* Description */}
      <div className="space-y-2">
        <label htmlFor="description" className="block text-sm font-medium text-slate-300">
          Description <span className="text-emerald-400">*</span>
        </label>
        <textarea
          id="description"
          value={formData.description}
          onChange={(e) => onChange({ description: e.target.value })}
          placeholder="Describe the purpose of this pool and how funds will be used…"
          rows={4}
          className={cn(
            "w-full resize-none rounded-lg border bg-slate-900/70 px-4 py-3 text-sm text-white placeholder:text-slate-500 outline-none transition-all duration-200 focus:ring-2",
            errors.description
              ? "border-red-500/70 focus:border-red-500/70 focus:ring-red-500/20"
              : "border-slate-700/80 focus:border-emerald-500/70 focus:ring-emerald-500/20"
          )}
        />
        <div className="flex items-center justify-between">
          {errors.description ? (
            <p className="flex items-center gap-1.5 text-xs text-red-400">
              <svg className="h-3.5 w-3.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              {errors.description}
            </p>
          ) : (
            <span />
          )}
          <p className="text-right text-xs text-slate-500">{formData.description.length} / 500</p>
        </div>
      </div>

      {/* End Date */}
      <div className="space-y-2">
        <label htmlFor="endDate" className="block text-sm font-medium text-slate-300">
          Pool End Date <span className="text-emerald-400">*</span>
        </label>
        <input
          id="endDate"
          type="date"
          value={formData.endDate}
          onChange={(e) => onChange({ endDate: e.target.value })}
          className={cn(
            "w-full rounded-lg border bg-slate-900/70 px-4 py-3 text-sm text-white outline-none transition-all duration-200 focus:ring-2 [color-scheme:dark]",
            errors.endDate
              ? "border-red-500/70 focus:border-red-500/70 focus:ring-red-500/20"
              : "border-slate-700/80 focus:border-emerald-500/70 focus:ring-emerald-500/20"
          )}
        />
        {errors.endDate && (
          <p className="flex items-center gap-1.5 text-xs text-red-400">
            <svg className="h-3.5 w-3.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            {errors.endDate}
          </p>
        )}
      </div>
    </div>
  );
}
