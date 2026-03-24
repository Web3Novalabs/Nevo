"use client";

import { useState, useMemo } from "react";
import { Search, Filter, X } from "lucide-react";
import { StatusBadge } from "@/components/ui/status-badge";

type PoolStatus = "Active" | "Completed";

interface Pool {
  id: string;
  title: string;
  description: string;
  category: string;
  status: PoolStatus;
  target: number;
  raised: number;
  imageColor: string;
}

const CATEGORIES = ["Education", "Health", "Environment", "Technology", "Community"];
const STATUSES: PoolStatus[] = ["Active", "Completed"];

const MOCK_POOLS: Pool[] = [
  {
    id: "1",
    title: "Clean Water Initiative",
    description: "Providing clean drinking water to communities in need.",
    category: "Health",
    status: "Active",
    target: 50000,
    raised: 12500,
    imageColor: "from-blue-400 to-blue-600",
  },
  {
    id: "2",
    title: "Tech Education for Kids",
    description: "Coding bootcamps and laptops for underprivileged students.",
    category: "Education",
    status: "Active",
    target: 25000,
    raised: 20000,
    imageColor: "from-purple-400 to-purple-600",
  },
  {
    id: "3",
    title: "Ocean Cleanup Project",
    description: "Removing plastic waste from coastal areas and beaches.",
    category: "Environment",
    status: "Active",
    target: 100000,
    raised: 85000,
    imageColor: "from-teal-400 to-teal-600",
  },
  {
    id: "4",
    title: "Community Solar Power",
    description: "Installing solar panels for low-income neighborhoods.",
    category: "Technology",
    status: "Completed",
    target: 40000,
    raised: 40000,
    imageColor: "from-yellow-400 to-orange-500",
  },
  {
    id: "5",
    title: "Local Food Bank Support",
    description: "Stocking up local food banks for the winter season.",
    category: "Community",
    status: "Active",
    target: 15000,
    raised: 5000,
    imageColor: "from-green-400 to-green-600",
  },
  {
    id: "6",
    title: "Medical Supplies Drive",
    description: "Emergency medical supplies for disaster relief.",
    category: "Health",
    status: "Completed",
    target: 30000,
    raised: 30000,
    imageColor: "from-red-400 to-red-600",
  },
];

export const ExplorePools = () => {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);
  const [selectedStatuses, setSelectedStatuses] = useState<PoolStatus[]>([]);

  const toggleCategory = (category: string) => {
    setSelectedCategories((prev) =>
      prev.includes(category)
        ? prev.filter((c) => c !== category)
        : [...prev, category]
    );
  };

  const toggleStatus = (status: PoolStatus) => {
    setSelectedStatuses((prev) =>
      prev.includes(status)
        ? prev.filter((s) => s !== status)
        : [...prev, status]
    );
  };

  const clearFilters = () => {
    setSearchQuery("");
    setSelectedCategories([]);
    setSelectedStatuses([]);
  };

  const filteredPools = useMemo(() => {
    return MOCK_POOLS.filter((pool) => {
      const matchSearch =
        pool.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        pool.description.toLowerCase().includes(searchQuery.toLowerCase());
      const matchCategory =
        selectedCategories.length === 0 ||
        selectedCategories.includes(pool.category);
      const matchStatus =
        selectedStatuses.length === 0 || selectedStatuses.includes(pool.status);

      return matchSearch && matchCategory && matchStatus;
    });
  }, [searchQuery, selectedCategories, selectedStatuses]);

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <div className="mb-8">
        <h1 className="text-4xl font-bold text-slate-900 dark:text-white mb-4">
          Explore Donation Pools
        </h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Discover and support impactful initiatives across various categories.
        </p>
      </div>

      <div className="flex flex-col lg:flex-row gap-8">
        {/* Filters Sidebar */}
        <div className="w-full lg:w-64 flex-shrink-0 space-y-6">
          <div className="bg-white dark:bg-slate-900 p-6 rounded-xl shadow-sm border border-slate-200 dark:border-slate-800">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold flex items-center gap-2 text-slate-900 dark:text-white">
                <Filter size={20} /> Filters
              </h2>
              {(selectedCategories.length > 0 || selectedStatuses.length > 0) && (
                <button
                  onClick={clearFilters}
                  className="text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300"
                >
                  Clear all
                </button>
              )}
            </div>

            {/* Status Filter */}
            <div className="mb-6">
              <h3 className="text-sm font-medium text-slate-700 dark:text-slate-300 mb-3">
                Status
              </h3>
              <div className="space-y-2">
                {STATUSES.map((status) => (
                  <label
                    key={status}
                    className="flex items-center gap-2 cursor-pointer"
                  >
                    <input
                      type="checkbox"
                      className="w-4 h-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500 dark:border-slate-600 dark:bg-slate-800"
                      checked={selectedStatuses.includes(status)}
                      onChange={() => toggleStatus(status)}
                    />
                    <span className="text-slate-600 dark:text-slate-400 text-sm">
                      {status}
                    </span>
                  </label>
                ))}
              </div>
            </div>

            {/* Category Filter */}
            <div>
              <h3 className="text-sm font-medium text-slate-700 dark:text-slate-300 mb-3">
                Categories
              </h3>
              <div className="space-y-2">
                {CATEGORIES.map((category) => (
                  <label
                    key={category}
                    className="flex items-center gap-2 cursor-pointer"
                  >
                    <input
                      type="checkbox"
                      className="w-4 h-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500 dark:border-slate-600 dark:bg-slate-800"
                      checked={selectedCategories.includes(category)}
                      onChange={() => toggleCategory(category)}
                    />
                    <span className="text-slate-600 dark:text-slate-400 text-sm">
                      {category}
                    </span>
                  </label>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Main Content: Search and Grid */}
        <div className="flex-1">
          {/* Search Bar */}
          <div className="relative mb-8">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <Search className="h-5 w-5 text-slate-400" />
            </div>
            <input
              type="text"
              placeholder="Search by pool name or description..."
              className="w-full pl-10 pr-4 py-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:text-white transition"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery("")}
                className="absolute inset-y-0 right-0 pr-3 flex items-center text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
              >
                <X className="h-5 w-5" />
              </button>
            )}
          </div>

          {/* Grid or Empty State */}
          {filteredPools.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6">
              {filteredPools.map((pool) => (
                <div
                  key={pool.id}
                  className="bg-white dark:bg-slate-900 rounded-xl shadow-sm border border-slate-200 dark:border-slate-800 overflow-hidden hover:shadow-xl transition-all duration-300 hover:-translate-y-1.5 group cursor-pointer"
                >
                  <div
                    className={"h-32 bg-gradient-to-tr " + pool.imageColor}
                  />
                  <div className="p-6">
                    <div className="flex items-center justify-between mb-3">
                      <span className="text-xs font-semibold uppercase tracking-wider text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/30 px-2 py-1 rounded-md">
                        {pool.category}
                      </span>
                      <StatusBadge
                        variant={pool.status === "Active" ? "success" : "default"}
                      >
                        {pool.status}
                      </StatusBadge>
                    </div>
                    <h3 className="text-lg font-bold text-slate-900 dark:text-white mb-2 group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors">
                      {pool.title}
                    </h3>
                    <p className="text-slate-600 dark:text-slate-400 text-sm mb-4 line-clamp-2">
                      {pool.description}
                    </p>

                    <div className="space-y-2">
                      <div className="flex justify-between text-sm">
                        <span className="text-slate-600 dark:text-slate-400">Raised</span>
                        <span className="font-semibold text-slate-900 dark:text-white">
                          ${pool.raised.toLocaleString()} / ${pool.target.toLocaleString()}
                        </span>
                      </div>
                      <div className="w-full bg-slate-100 dark:bg-slate-800 rounded-full h-2 overflow-hidden">
                        <div
                          className="bg-blue-600 h-2 rounded-full"
                          style={{
                            width: `${Math.min(
                              100,
                              (pool.raised / pool.target) * 100
                            )}%`,
                          }}
                        />
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="bg-white dark:bg-slate-900 rounded-xl p-12 text-center border border-slate-200 dark:border-slate-800 shadow-sm">
              <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-slate-100 dark:bg-slate-800 mb-6">
                <Search className="h-8 w-8 text-slate-400" />
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2">
                No pools found
              </h3>
              <p className="text-slate-600 dark:text-slate-400 mb-6 max-w-md mx-auto">
                We couldn&apos;t find any donation pools matching your search and filter criteria. Try adjusting your filters or search term.
              </p>
              <button
                onClick={clearFilters}
                className="bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 text-slate-900 dark:text-white px-6 py-2 rounded-lg font-medium transition"
              >
                Clear all filters
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
