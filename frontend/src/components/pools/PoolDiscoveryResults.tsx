"use client";

import { Loader2, Search } from "lucide-react";

import { Button } from "@/components/ui/button";

const SKELETON_COUNT = 9;

export function PoolGridSkeleton() {
  return (
    <div className="grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3">
      {Array.from({ length: SKELETON_COUNT }).map((_, index) => (
        <div
          key={index}
          className="min-h-[460px] animate-pulse rounded-xl border border-slate-800 bg-slate-900/70"
        >
          <div className="h-44 rounded-t-xl bg-slate-800" />
          <div className="space-y-4 p-5">
            <div className="h-5 w-28 rounded bg-slate-800" />
            <div className="h-7 w-3/4 rounded bg-slate-800" />
            <div className="h-16 rounded bg-slate-800/70" />
            <div className="h-2 rounded-full bg-slate-800" />
          </div>
        </div>
      ))}
    </div>
  );
}

export function PoolPagination({
  visibleCount,
  totalCount,
  hasMore,
  isLoadingMore,
  onLoadMore,
}: {
  visibleCount: number;
  totalCount: number;
  hasMore: boolean;
  isLoadingMore: boolean;
  onLoadMore: () => void;
}) {
  return (
    <div className="flex flex-col items-center gap-4">
      <p className="text-sm text-slate-500">
        Showing {visibleCount} of {totalCount} pools
      </p>
      {hasMore && (
        <Button onClick={onLoadMore} disabled={isLoadingMore}>
          {isLoadingMore && <Loader2 className="h-4 w-4 animate-spin" />}
          Load more pools
        </Button>
      )}
    </div>
  );
}

export function EmptyPoolState({ onClear }: { onClear: () => void }) {
  return (
    <div className="rounded-xl border border-slate-800 bg-slate-900/70 p-10 text-center">
      <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-slate-800 text-slate-500">
        <Search className="h-6 w-6" />
      </div>
      <h2 className="mt-4 text-xl font-bold text-white">No pools found</h2>
      <p className="mx-auto mt-2 max-w-md text-sm text-slate-400">
        Try a broader search, choose a different status, or clear the date
        range to see more donation pools.
      </p>
      <Button onClick={onClear} className="mt-6">
        Clear filters
      </Button>
    </div>
  );
}
