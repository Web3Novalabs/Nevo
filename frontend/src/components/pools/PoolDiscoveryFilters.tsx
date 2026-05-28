"use client";

import { Search, X } from "lucide-react";

import type { PoolSort, POOL_CATEGORIES, POOL_STATUSES } from "./mock-pools";
import { DateInput, FilterSelect } from "./PoolDiscoveryInputs";

type Category = (typeof POOL_CATEGORIES)[number];
type Status = (typeof POOL_STATUSES)[number];

interface PoolDiscoveryFiltersProps {
  categories: readonly Category[];
  statuses: readonly Status[];
  searchQuery: string;
  selectedStatus: "all" | Status;
  selectedCategory: "all" | Category;
  startDate: string;
  endDate: string;
  sortBy: PoolSort;
  resultCount: number;
  onSearchChange: (value: string) => void;
  onStatusChange: (value: "all" | Status) => void;
  onCategoryChange: (value: "all" | Category) => void;
  onStartDateChange: (value: string) => void;
  onEndDateChange: (value: string) => void;
  onSortChange: (value: PoolSort) => void;
  onClear: () => void;
}

export function PoolDiscoveryFilters({
  categories,
  statuses,
  searchQuery,
  selectedStatus,
  selectedCategory,
  startDate,
  endDate,
  sortBy,
  resultCount,
  onSearchChange,
  onStatusChange,
  onCategoryChange,
  onStartDateChange,
  onEndDateChange,
  onSortChange,
  onClear,
}: PoolDiscoveryFiltersProps) {
  const hasFilters =
    searchQuery ||
    selectedStatus !== "all" ||
    selectedCategory !== "all" ||
    startDate ||
    endDate;

  return (
    <section className="rounded-xl border border-slate-800 bg-slate-900/70 p-4 shadow-xl shadow-slate-950/20 sm:p-5">
      <div className="grid gap-4 lg:grid-cols-[1.4fr_repeat(4,minmax(0,1fr))_auto]">
        <label className="relative block">
          <span className="sr-only">Search pools</span>
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
          <input
            value={searchQuery}
            onChange={(event) => onSearchChange(event.target.value)}
            placeholder="Search by title or description"
            className="h-11 w-full rounded-lg border border-slate-700 bg-slate-950/60 pl-10 pr-10 text-sm text-white outline-none transition placeholder:text-slate-500 focus:border-emerald-400 focus:ring-2 focus:ring-emerald-400/20"
          />
          {searchQuery && (
            <button
              type="button"
              onClick={() => onSearchChange("")}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 transition hover:text-white"
              aria-label="Clear search"
            >
              <X className="h-4 w-4" />
            </button>
          )}
        </label>

        <FilterSelect label="Status" value={selectedStatus} onChange={onStatusChange}>
          <option value="all">All statuses</option>
          {statuses.map((status) => (
            <option key={status} value={status}>
              {status === "open" ? "Open" : "Closed"}
            </option>
          ))}
        </FilterSelect>

        <FilterSelect
          label="Category"
          value={selectedCategory}
          onChange={onCategoryChange}
        >
          <option value="all">All categories</option>
          {categories.map((category) => (
            <option key={category} value={category}>
              {category}
            </option>
          ))}
        </FilterSelect>

        <DateInput label="From" value={startDate} onChange={onStartDateChange} />
        <DateInput label="To" value={endDate} onChange={onEndDateChange} />

        <FilterSelect label="Sort" value={sortBy} onChange={onSortChange}>
          <option value="newest">Newest</option>
          <option value="most-funded">Most funded</option>
          <option value="close-to-goal">Close to goal</option>
          <option value="trending">Trending</option>
        </FilterSelect>
      </div>

      <div className="mt-4 flex flex-wrap items-center justify-between gap-3 text-sm text-slate-400">
        <span>{resultCount} pools found</span>
        {hasFilters && (
          <button
            type="button"
            onClick={onClear}
            className="font-semibold text-emerald-300 transition hover:text-white"
          >
            Clear filters
          </button>
        )}
      </div>
    </section>
  );
}
