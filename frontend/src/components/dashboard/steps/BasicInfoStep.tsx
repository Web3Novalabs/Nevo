"use client";

import { FormData } from "../CreatePoolStepper";

interface BasicInfoStepProps {
  formData: FormData;
  onChange: (updates: Partial<FormData>) => void;
}

const CATEGORIES = ["Education", "Medical", "Community", "Environment", "Arts", "Other"];

export function BasicInfoStep({ formData, onChange }: BasicInfoStepProps) {
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
          className="w-full rounded-lg border border-slate-700/80 bg-slate-900/70 px-4 py-3 text-sm text-white placeholder:text-slate-500 outline-none transition-all duration-200 focus:border-emerald-500/70 focus:ring-2 focus:ring-emerald-500/20"
        />
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
            className="w-full appearance-none rounded-lg border border-slate-700/80 bg-slate-900/70 px-4 py-3 text-sm text-white outline-none transition-all duration-200 focus:border-emerald-500/70 focus:ring-2 focus:ring-emerald-500/20"
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
          className="w-full resize-none rounded-lg border border-slate-700/80 bg-slate-900/70 px-4 py-3 text-sm text-white placeholder:text-slate-500 outline-none transition-all duration-200 focus:border-emerald-500/70 focus:ring-2 focus:ring-emerald-500/20"
        />
        <p className="text-right text-xs text-slate-500">
          {formData.description.length} / 500
        </p>
      </div>

      {/* End Date */}
      <div className="space-y-2">
        <label htmlFor="endDate" className="block text-sm font-medium text-slate-300">
          Pool End Date
        </label>
        <input
          id="endDate"
          type="date"
          value={formData.endDate}
          onChange={(e) => onChange({ endDate: e.target.value })}
          className="w-full rounded-lg border border-slate-700/80 bg-slate-900/70 px-4 py-3 text-sm text-white outline-none transition-all duration-200 focus:border-emerald-500/70 focus:ring-2 focus:ring-emerald-500/20 [color-scheme:dark]"
        />
      </div>
    </div>
  );
}
