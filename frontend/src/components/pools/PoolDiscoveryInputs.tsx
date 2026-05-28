"use client";

import type { ReactNode } from "react";

interface FilterSelectProps<T extends string> {
  label: string;
  value: T;
  children: ReactNode;
  onChange: (value: T) => void;
}

export function FilterSelect<T extends string>({
  label,
  value,
  children,
  onChange,
}: FilterSelectProps<T>) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-semibold text-slate-500">
        {label}
      </span>
      <select
        value={value}
        onChange={(event) => onChange(event.target.value as T)}
        className="h-11 w-full rounded-lg border border-slate-700 bg-slate-950/60 px-3 text-sm text-white outline-none transition focus:border-emerald-400 focus:ring-2 focus:ring-emerald-400/20"
      >
        {children}
      </select>
    </label>
  );
}

export function DateInput({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-semibold text-slate-500">
        {label}
      </span>
      <input
        type="date"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="h-11 w-full rounded-lg border border-slate-700 bg-slate-950/60 px-3 text-sm text-white outline-none transition focus:border-emerald-400 focus:ring-2 focus:ring-emerald-400/20"
      />
    </label>
  );
}
