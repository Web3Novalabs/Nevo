"use client";

import { useEffect, useMemo, useState } from "react";

import { PoolCard } from "@/components/PoolCard";

import { PoolDiscoveryFilters } from "./PoolDiscoveryFilters";
import { PoolDiscoveryHeader } from "./PoolDiscoveryHeader";
import {
  EmptyPoolState,
  PoolGridSkeleton,
  PoolPagination,
} from "./PoolDiscoveryResults";
import {
  MOCK_POOLS,
  POOL_CATEGORIES,
  POOL_STATUSES,
  sortPools,
  type DiscoverablePool,
  type PoolSort,
} from "./mock-pools";

const PAGE_SIZE = 9;

export function PoolBrowser() {
  const [pools, setPools] = useState<DiscoverablePool[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const [visibleCount, setVisibleCount] = useState(PAGE_SIZE);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedStatus, setSelectedStatus] =
    useState<"all" | "open" | "closed">("all");
  const [selectedCategory, setSelectedCategory] = useState<
    "all" | (typeof POOL_CATEGORIES)[number]
  >("all");
  const [startDate, setStartDate] = useState("");
  const [endDate, setEndDate] = useState("");
  const [sortBy, setSortBy] = useState<PoolSort>("newest");

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setPools(MOCK_POOLS);
      setIsLoading(false);
    }, 500);

    return () => window.clearTimeout(timer);
  }, []);

  useEffect(() => {
    setVisibleCount(PAGE_SIZE);
  }, [searchQuery, selectedStatus, selectedCategory, startDate, endDate, sortBy]);

  const filteredPools = useMemo(() => {
    const normalizedSearch = searchQuery.trim().toLowerCase();

    return pools
      .filter((pool) => {
        const matchesSearch =
          !normalizedSearch ||
          pool.title.toLowerCase().includes(normalizedSearch) ||
          pool.description.toLowerCase().includes(normalizedSearch);
        const matchesStatus =
          selectedStatus === "all" || pool.status === selectedStatus;
        const matchesCategory =
          selectedCategory === "all" || pool.category === selectedCategory;
        const matchesStart = !startDate || pool.createdAt >= startDate;
        const matchesEnd = !endDate || pool.createdAt <= endDate;

        return (
          matchesSearch &&
          matchesStatus &&
          matchesCategory &&
          matchesStart &&
          matchesEnd
        );
      })
      .sort(sortPools(sortBy));
  }, [pools, searchQuery, selectedStatus, selectedCategory, startDate, endDate, sortBy]);

  const visiblePools = filteredPools.slice(0, visibleCount);
  const hasMore = visibleCount < filteredPools.length;

  const loadMore = () => {
    setIsLoadingMore(true);
    window.setTimeout(() => {
      setVisibleCount((count) => count + PAGE_SIZE);
      setIsLoadingMore(false);
    }, 350);
  };

  const clearFilters = () => {
    setSearchQuery("");
    setSelectedStatus("all");
    setSelectedCategory("all");
    setStartDate("");
    setEndDate("");
    setSortBy("newest");
  };

  return (
    <div className="mx-auto max-w-7xl space-y-8 px-4 pb-20 pt-28 sm:px-6 lg:px-8">
      <PoolDiscoveryHeader />

      <PoolDiscoveryFilters
        categories={POOL_CATEGORIES}
        statuses={POOL_STATUSES}
        searchQuery={searchQuery}
        selectedStatus={selectedStatus}
        selectedCategory={selectedCategory}
        startDate={startDate}
        endDate={endDate}
        sortBy={sortBy}
        resultCount={filteredPools.length}
        onSearchChange={setSearchQuery}
        onStatusChange={setSelectedStatus}
        onCategoryChange={setSelectedCategory}
        onStartDateChange={setStartDate}
        onEndDateChange={setEndDate}
        onSortChange={setSortBy}
        onClear={clearFilters}
      />

      {isLoading ? (
        <PoolGridSkeleton />
      ) : visiblePools.length > 0 ? (
        <>
          <div className="grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3">
            {visiblePools.map((pool) => (
              <PoolCard key={pool.id} {...pool} />
            ))}
          </div>
          <PoolPagination
            visibleCount={visiblePools.length}
            totalCount={filteredPools.length}
            hasMore={hasMore}
            isLoadingMore={isLoadingMore}
            onLoadMore={loadMore}
          />
        </>
      ) : (
        <EmptyPoolState onClear={clearFilters} />
      )}
    </div>
  );
}
