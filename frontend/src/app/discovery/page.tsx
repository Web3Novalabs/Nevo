"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import {
  Search,
  Filter,
  TrendingUp,
  Users,
  Clock,
  ChevronDown,
  Loader2,
  Heart,
  Globe,
  Leaf,
  GraduationCap,
  Droplets,
  PawPrint,
  Sparkles,
  ArrowUpRight,
  X,
} from "lucide-react";
import Navigation from "@/components/Navigation";
import Footer from "@/components/Footer";
import Link from "next/link";
import Image from "next/image";

// ─── Types ───────────────────────────────────────────────────────────
interface Pool {
  id: number;
  title: string;
  description: string;
  category: string;
  raised: number;
  goal: number;
  contributors: number;
  daysLeft: number;
  yieldRate: number;
  image: string;
  creator: string;
  trending: boolean;
}

type Category = "All" | "Healthcare" | "Environment" | "Education" | "Water" | "Animals" | "Community";

// ─── Mock Data Generator ─────────────────────────────────────────────
const categories: Category[] = [
  "All",
  "Healthcare",
  "Environment",
  "Education",
  "Water",
  "Animals",
  "Community",
];

const categoryIcons: Record<string, React.ReactNode> = {
  Healthcare: <Heart size={16} />,
  Environment: <Leaf size={16} />,
  Education: <GraduationCap size={16} />,
  Water: <Droplets size={16} />,
  Animals: <PawPrint size={16} />,
  Community: <Globe size={16} />,
};

const categoryColors: Record<string, string> = {
  Healthcare: "from-rose-500/20 to-pink-500/20 border-rose-500/30",
  Environment: "from-emerald-500/20 to-green-500/20 border-emerald-500/30",
  Education: "from-violet-500/20 to-purple-500/20 border-violet-500/30",
  Water: "from-sky-500/20 to-blue-500/20 border-sky-500/30",
  Animals: "from-amber-500/20 to-orange-500/20 border-amber-500/30",
  Community: "from-cyan-500/20 to-teal-500/20 border-cyan-500/30",
};

const categoryTextColors: Record<string, string> = {
  Healthcare: "text-rose-400",
  Environment: "text-emerald-400",
  Education: "text-violet-400",
  Water: "text-sky-400",
  Animals: "text-amber-400",
  Community: "text-cyan-400",
};

const poolTitles: Record<string, string[]> = {
  Healthcare: [
    "Rural Clinic Expansion Fund",
    "Mental Health Support Hub",
    "Clean Medical Supplies Drive",
    "Pediatric Care Initiative",
    "Community Health Workers Fund",
    "Telehealth Access Program",
    "Vaccination Outreach Campaign",
    "Emergency Medical Aid Reserve",
  ],
  Environment: [
    "Reforestation Initiative",
    "Ocean Cleanup Partnership",
    "Solar Energy for Villages",
    "Carbon Offset Community Pool",
    "Wildlife Corridor Restoration",
    "Sustainable Farming Grants",
    "Urban Green Spaces Project",
    "Climate Resilience Fund",
  ],
  Education: [
    "Girls Education Scholarship",
    "STEM Labs for Schools",
    "Library Expansion Project",
    "Teacher Training Program",
    "Digital Literacy Initiative",
    "University Access Fund",
    "After-School Programs",
    "Early Childhood Education",
  ],
  Water: [
    "Clean Water Wells Project",
    "Water Purification Systems",
    "Rainwater Harvesting Fund",
    "Drought Relief Program",
    "Sanitation Infrastructure",
    "River Restoration Fund",
    "Community Water Testing",
    "Desalination Research Pool",
  ],
  Animals: [
    "Wildlife Rescue Center",
    "Ocean Conservation Fund",
    "Endangered Species Protection",
    "Animal Shelter Expansion",
    "Marine Life Research",
    "Anti-Poaching Initiative",
    "Pet Adoption Support",
    "Habitat Preservation Fund",
  ],
  Community: [
    "Affordable Housing Initiative",
    "Food Bank Network",
    "Youth Mentorship Program",
    "Disaster Relief Reserve",
    "Small Business Microloans",
    "Community Arts Center",
    "Senior Care Support",
    "Refugee Assistance Fund",
  ],
};

function generatePools(startId: number, count: number): Pool[] {
  const pools: Pool[] = [];
  const catKeys = Object.keys(poolTitles);

  for (let i = 0; i < count; i++) {
    const catIndex = (startId + i) % catKeys.length;
    const category = catKeys[catIndex];
    const titles = poolTitles[category];
    const titleIndex = ((startId + i) >> 1) % titles.length;
    const goal = Math.floor(Math.random() * 90000) + 10000;
    const raised = Math.floor(Math.random() * goal * 0.95);

    const unsplashIds = [
      "1541249591-6284fcdbf769", // Water
      "1577896851231-70ef18881754", // Education
      "1542601906990-b4d3fb778b09", // Environment
      "1538108149393-fbbd81895907", // Health
      "1460661419201-fd4cecdf8a8b", // Community
      "1588680145224-811c751270ae", // Emergency
    ];

    pools.push({
      id: startId + i,
      title: titles[titleIndex],
      description: `Support this ${category.toLowerCase()} initiative and make a real impact. Every contribution is tracked transparently on the Stellar blockchain.`,
      category,
      raised,
      goal,
      contributors: Math.floor(Math.random() * 450) + 10,
      daysLeft: Math.floor(Math.random() * 60) + 1,
      yieldRate: parseFloat((Math.random() * 5 + 1).toFixed(1)),
      image: `https://images.unsplash.com/photo-${unsplashIds[i % unsplashIds.length]}?auto=format&fit=crop&q=80&w=800`,
      creator: `G${Array.from({ length: 6 }, () => "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"[Math.floor(Math.random() * 32)]).join("")}...`,
      trending: Math.random() > 0.7,
    });
  }
  return pools;
}

// ─── Pool Card Component ─────────────────────────────────────────────
function PoolCard({ pool, index }: { pool: Pool; index: number }) {
  const progress = (pool.raised / pool.goal) * 100;

  return (
    <div
      className="group relative bg-gradient-to-br from-slate-800/80 to-slate-900/80 backdrop-blur-sm rounded-2xl border border-slate-700/50 overflow-hidden hover:border-[#50C878]/40 transition-all duration-500 hover:shadow-[0_0_40px_rgba(80,200,120,0.1)] hover:-translate-y-1"
      style={{
        animationDelay: `${(index % 9) * 80}ms`,
        animation: "fadeSlideUp 0.6s ease-out backwards",
      }}
    >
      {/* Pool Image */}
      <div className="relative h-48 w-full overflow-hidden">
        <Image
          src={pool.image}
          alt={pool.title}
          fill
          sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw"
          className="object-cover transition-transform duration-500 group-hover:scale-105"
        />
        {/* Category Gradient Overlay (bottom) */}
        <div
          className={`absolute bottom-0 left-0 right-0 h-1 bg-gradient-to-r ${categoryColors[pool.category]?.replace("border-", "")?.split(" ").slice(0, 2).join(" ") || "from-[#50C878] to-[#14B8A6]"}`}
        />
      </div>

      <div className="p-6">
        {/* Top row: Category + Trending */}
        <div className="flex items-center justify-between mb-4">
          <span
            className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-gradient-to-r ${categoryColors[pool.category]} border backdrop-blur-sm`}
          >
            <span className={categoryTextColors[pool.category]}>
              {categoryIcons[pool.category]}
            </span>
            <span className={categoryTextColors[pool.category]}>
              {pool.category}
            </span>
          </span>
          {pool.trending && (
            <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium bg-[#50C878]/10 text-[#50C878] border border-[#50C878]/20">
              <TrendingUp size={12} />
              Trending
            </span>
          )}
        </div>

        {/* Title */}
        <h3 className="text-lg font-bold text-white mb-2 group-hover:text-[#50C878] transition-colors duration-300 line-clamp-1">
          {pool.title}
        </h3>

        {/* Description */}
        <p className="text-sm text-slate-400 mb-5 line-clamp-2 leading-relaxed">
          {pool.description}
        </p>

        {/* Progress Bar */}
        <div className="mb-4">
          <div className="flex justify-between text-sm mb-2">
            <span className="text-white font-semibold">
              ${pool.raised.toLocaleString()}
            </span>
            <span className="text-slate-400">
              ${pool.goal.toLocaleString()} goal
            </span>
          </div>
          <div className="h-2 bg-slate-700/50 rounded-full overflow-hidden">
            <div
              className="h-full bg-gradient-to-r from-[#50C878] to-[#14B8A6] rounded-full transition-all duration-1000 ease-out relative"
              style={{ width: `${Math.min(progress, 100)}%` }}
            >
              <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" />
            </div>
          </div>
          <div className="text-right mt-1">
            <span className="text-xs text-[#50C878] font-medium">
              {progress.toFixed(1)}% funded
            </span>
          </div>
        </div>

        {/* Stats Row */}
        <div className="flex items-center gap-4 text-sm text-slate-400 mb-5">
          <div className="flex items-center gap-1.5">
            <Users size={14} className="text-slate-500" />
            <span>{pool.contributors}</span>
          </div>
          <div className="flex items-center gap-1.5">
            <Clock size={14} className="text-slate-500" />
            <span>{pool.daysLeft}d left</span>
          </div>
          <div className="flex items-center gap-1.5">
            <TrendingUp size={14} className="text-[#50C878]" />
            <span className="text-[#50C878]">{pool.yieldRate}% APY</span>
          </div>
        </div>

        {/* Creator + CTA */}
        <div className="flex items-center justify-between pt-4 border-t border-slate-700/50">
          <span className="text-xs text-slate-500 font-mono">
            {pool.creator}
          </span>
          <button className="inline-flex items-center gap-1 text-sm font-semibold text-[#50C878] hover:text-white transition-colors duration-300 group/btn">
            Contribute
            <ArrowUpRight
              size={14}
              className="transition-transform duration-300 group-hover/btn:translate-x-0.5 group-hover/btn:-translate-y-0.5"
            />
          </button>
        </div>
      </div>
    </div>
  );
}

// ─── Loading Skeleton ────────────────────────────────────────────────
function PoolSkeleton() {
  return (
    <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 rounded-2xl border border-slate-700/30 overflow-hidden animate-pulse">
      <div className="h-2 bg-slate-700/50" />
      <div className="p-6">
        <div className="flex items-center justify-between mb-4">
          <div className="h-6 w-24 bg-slate-700/50 rounded-full" />
          <div className="h-6 w-20 bg-slate-700/50 rounded-full" />
        </div>
        <div className="h-6 w-3/4 bg-slate-700/50 rounded mb-2" />
        <div className="h-4 w-full bg-slate-700/30 rounded mb-1" />
        <div className="h-4 w-2/3 bg-slate-700/30 rounded mb-5" />
        <div className="h-2 w-full bg-slate-700/50 rounded-full mb-4" />
        <div className="flex gap-4 mb-5">
          <div className="h-4 w-16 bg-slate-700/30 rounded" />
          <div className="h-4 w-16 bg-slate-700/30 rounded" />
          <div className="h-4 w-20 bg-slate-700/30 rounded" />
        </div>
        <div className="flex items-center justify-between pt-4 border-t border-slate-700/30">
          <div className="h-4 w-28 bg-slate-700/30 rounded" />
          <div className="h-4 w-20 bg-slate-700/30 rounded" />
        </div>
      </div>
    </div>
  );
}

// ─── Sort Options ────────────────────────────────────────────────────
type SortOption = "trending" | "newest" | "most-funded" | "ending-soon";

const sortOptions: { value: SortOption; label: string }[] = [
  { value: "trending", label: "Trending" },
  { value: "newest", label: "Newest" },
  { value: "most-funded", label: "Most Funded" },
  { value: "ending-soon", label: "Ending Soon" },
];

// ─── Discovery Page ──────────────────────────────────────────────────
const ITEMS_PER_PAGE = 9;

export default function DiscoveryPage() {
  const [pools, setPools] = useState<Pool[]>([]);
  const [filteredPools, setFilteredPools] = useState<Pool[]>([]);
  const [displayedPools, setDisplayedPools] = useState<Pool[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<Category>("All");
  const [searchQuery, setSearchQuery] = useState("");
  const [sortBy, setSortBy] = useState<SortOption>("trending");
  const [showSortDropdown, setShowSortDropdown] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const [page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(true);
  const [totalLoaded, setTotalLoaded] = useState(0);

  const observerRef = useRef<HTMLDivElement>(null);
  const sortDropdownRef = useRef<HTMLDivElement>(null);

  // Initial load
  useEffect(() => {
    setIsLoading(true);
    const timer = setTimeout(() => {
      const initialPools = generatePools(1, 100);
      setPools(initialPools);
      setIsLoading(false);
    }, 800);
    return () => clearTimeout(timer);
  }, []);

  // Filter + Sort
  useEffect(() => {
    let result = [...pools];

    // Category filter
    if (selectedCategory !== "All") {
      result = result.filter((p) => p.category === selectedCategory);
    }

    // Search filter
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (p) =>
          p.title.toLowerCase().includes(q) ||
          p.description.toLowerCase().includes(q) ||
          p.category.toLowerCase().includes(q)
      );
    }

    // Sort
    switch (sortBy) {
      case "trending":
        result.sort((a, b) => (b.trending ? 1 : 0) - (a.trending ? 1 : 0) || b.contributors - a.contributors);
        break;
      case "newest":
        result.sort((a, b) => b.id - a.id);
        break;
      case "most-funded":
        result.sort((a, b) => b.raised / b.goal - a.raised / a.goal);
        break;
      case "ending-soon":
        result.sort((a, b) => a.daysLeft - b.daysLeft);
        break;
    }

    setFilteredPools(result);
    setPage(1);
    setDisplayedPools(result.slice(0, ITEMS_PER_PAGE));
    setHasMore(result.length > ITEMS_PER_PAGE);
    setTotalLoaded(Math.min(ITEMS_PER_PAGE, result.length));
  }, [pools, selectedCategory, searchQuery, sortBy]);

  // Load more
  const loadMore = useCallback(() => {
    if (isLoadingMore || !hasMore) return;
    setIsLoadingMore(true);

    setTimeout(() => {
      const nextPage = page + 1;
      const startIdx = (nextPage - 1) * ITEMS_PER_PAGE;
      const endIdx = startIdx + ITEMS_PER_PAGE;
      const newItems = filteredPools.slice(startIdx, endIdx);

      if (newItems.length > 0) {
        setDisplayedPools((prev) => [...prev, ...newItems]);
        setPage(nextPage);
        setTotalLoaded((prev) => prev + newItems.length);
        setHasMore(endIdx < filteredPools.length);
      } else {
        setHasMore(false);
      }
      setIsLoadingMore(false);
    }, 600);
  }, [filteredPools, page, isLoadingMore, hasMore]);

  // Intersection Observer for infinite scroll
  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !isLoadingMore) {
          loadMore();
        }
      },
      { threshold: 0.1, rootMargin: "200px" }
    );

    const current = observerRef.current;
    if (current) observer.observe(current);

    return () => {
      if (current) observer.unobserve(current);
    };
  }, [loadMore, hasMore, isLoadingMore]);

  // Close sort dropdown on outside click
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (sortDropdownRef.current && !sortDropdownRef.current.contains(e.target as Node)) {
        setShowSortDropdown(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <div className="bg-[#0F172A] min-h-screen">
      <Navigation />

      {/* Hero Header */}
      <section className="relative pt-32 pb-16 px-4 sm:px-6 lg:px-8 overflow-hidden">
        {/* Background Effects */}
        <div className="absolute inset-0 overflow-hidden pointer-events-none">
          <div className="absolute top-0 left-1/4 w-96 h-96 bg-[#50C878]/5 rounded-full blur-[120px]" />
          <div className="absolute top-20 right-1/4 w-80 h-80 bg-blue-500/5 rounded-full blur-[100px]" />
          <div className="absolute bottom-0 left-1/2 -translate-x-1/2 w-full h-px bg-gradient-to-r from-transparent via-[#50C878]/20 to-transparent" />
        </div>

        <div className="relative max-w-7xl mx-auto text-center">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-[#50C878]/10 border border-[#50C878]/20 text-[#50C878] text-sm font-medium mb-6">
            <Sparkles size={16} />
            Discover Donation Pools
          </div>
          <h1 className="text-4xl sm:text-5xl lg:text-6xl font-bold text-white mb-4 leading-tight">
            Explore & Support{" "}
            <span className="bg-gradient-to-r from-[#50C878] to-[#14B8A6] bg-clip-text text-transparent">
              Causes
            </span>
          </h1>
          <p className="text-lg text-slate-400 max-w-2xl mx-auto mb-10">
            Browse hundreds of transparent donation pools on Stellar. Find causes
            that matter to you and make an impact with blockchain-verified
            contributions.
          </p>

          {/* Search Bar */}
          <div className="max-w-2xl mx-auto relative">
            <div className="relative group">
              <Search
                size={20}
                className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 group-focus-within:text-[#50C878] transition-colors"
              />
              <input
                id="discovery-search"
                type="text"
                placeholder="Search pools by name, category, or keyword..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-12 pr-12 py-4 bg-slate-800/60 backdrop-blur-sm border border-slate-700/50 rounded-2xl text-white placeholder:text-slate-500 focus:outline-none focus:border-[#50C878]/50 focus:ring-2 focus:ring-[#50C878]/20 transition-all duration-300"
              />
              {searchQuery && (
                <button
                  onClick={() => setSearchQuery("")}
                  className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-400 hover:text-white transition-colors"
                >
                  <X size={18} />
                </button>
              )}
            </div>
          </div>
        </div>
      </section>

      {/* Filters & Sort */}
      <section className="px-4 sm:px-6 lg:px-8 pb-8">
        <div className="max-w-7xl mx-auto">
          <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
            {/* Category Pills */}
            <div className="flex flex-wrap gap-2">
              {categories.map((cat) => (
                <button
                  key={cat}
                  id={`filter-${cat.toLowerCase()}`}
                  onClick={() => setSelectedCategory(cat)}
                  className={`inline-flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium transition-all duration-300 ${selectedCategory === cat
                      ? "bg-[#50C878] text-black shadow-lg shadow-[#50C878]/20"
                      : "bg-slate-800/60 text-slate-400 hover:bg-slate-700/60 hover:text-white border border-slate-700/50"
                    }`}
                >
                  {cat !== "All" && (
                    <span className={selectedCategory === cat ? "text-black" : ""}>
                      {categoryIcons[cat]}
                    </span>
                  )}
                  {cat === "All" && <Filter size={14} />}
                  {cat}
                </button>
              ))}
            </div>

            {/* Sort Dropdown & Count */}
            <div className="flex items-center gap-4">
              <span className="text-sm text-slate-500">
                {filteredPools.length} pools found
              </span>
              <div className="relative" ref={sortDropdownRef}>
                <button
                  id="sort-dropdown-trigger"
                  onClick={() => setShowSortDropdown(!showSortDropdown)}
                  className="inline-flex items-center gap-2 px-4 py-2 bg-slate-800/60 border border-slate-700/50 rounded-xl text-sm text-slate-300 hover:text-white hover:border-slate-600 transition-all duration-300"
                >
                  {sortOptions.find((s) => s.value === sortBy)?.label}
                  <ChevronDown
                    size={14}
                    className={`transition-transform duration-300 ${showSortDropdown ? "rotate-180" : ""}`}
                  />
                </button>
                {showSortDropdown && (
                  <div className="absolute right-0 mt-2 w-44 bg-slate-800 border border-slate-700/50 rounded-xl shadow-xl shadow-black/20 overflow-hidden z-10">
                    {sortOptions.map((opt) => (
                      <button
                        key={opt.value}
                        id={`sort-${opt.value}`}
                        onClick={() => {
                          setSortBy(opt.value);
                          setShowSortDropdown(false);
                        }}
                        className={`w-full text-left px-4 py-2.5 text-sm transition-colors ${sortBy === opt.value
                            ? "bg-[#50C878]/10 text-[#50C878]"
                            : "text-slate-300 hover:bg-slate-700/50 hover:text-white"
                          }`}
                      >
                        {opt.label}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Pool Grid */}
      <section className="px-4 sm:px-6 lg:px-8 pb-16">
        <div className="max-w-7xl mx-auto">
          {isLoading ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {Array.from({ length: 9 }).map((_, i) => (
                <PoolSkeleton key={i} />
              ))}
            </div>
          ) : displayedPools.length === 0 ? (
            /* Empty State */
            <div className="text-center py-20">
              <div className="inline-flex items-center justify-center w-20 h-20 rounded-full bg-slate-800/60 border border-slate-700/50 mb-6">
                <Search size={32} className="text-slate-500" />
              </div>
              <h3 className="text-xl font-bold text-white mb-2">
                No pools found
              </h3>
              <p className="text-slate-400 mb-6">
                Try adjusting your search or filters to find what you&apos;re
                looking for.
              </p>
              <button
                onClick={() => {
                  setSearchQuery("");
                  setSelectedCategory("All");
                }}
                className="px-6 py-2.5 bg-[#50C878] text-black rounded-xl font-semibold hover:brightness-110 transition"
              >
                Clear Filters
              </button>
            </div>
          ) : (
            <>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {displayedPools.map((pool, index) => (
                  <PoolCard key={pool.id} pool={pool} index={index} />
                ))}
                {/* Loading skeletons while fetching more */}
                {isLoadingMore &&
                  Array.from({ length: 3 }).map((_, i) => (
                    <PoolSkeleton key={`skeleton-${i}`} />
                  ))}
              </div>

              {/* Progress indicator */}
              <div className="mt-10 text-center">
                <div className="inline-flex items-center gap-3 px-5 py-2.5 bg-slate-800/40 border border-slate-700/30 rounded-full">
                  <div className="h-1.5 w-32 bg-slate-700/50 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-gradient-to-r from-[#50C878] to-[#14B8A6] rounded-full transition-all duration-700"
                      style={{
                        width: `${(totalLoaded / filteredPools.length) * 100}%`,
                      }}
                    />
                  </div>
                  <span className="text-xs text-slate-400">
                    Showing {totalLoaded} of {filteredPools.length}
                  </span>
                </div>
              </div>

              {/* Load More Button */}
              {hasMore && (
                <div className="mt-8 text-center">
                  <button
                    id="load-more-btn"
                    onClick={loadMore}
                    disabled={isLoadingMore}
                    className="group inline-flex items-center gap-2 px-8 py-3.5 bg-gradient-to-r from-[#50C878] to-[#14B8A6] text-black rounded-xl font-semibold hover:shadow-lg hover:shadow-[#50C878]/20 transition-all duration-300 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100"
                  >
                    {isLoadingMore ? (
                      <>
                        <Loader2 size={18} className="animate-spin" />
                        Loading...
                      </>
                    ) : (
                      <>
                        Load More Pools
                        <ChevronDown
                          size={18}
                          className="transition-transform duration-300 group-hover:translate-y-0.5"
                        />
                      </>
                    )}
                  </button>
                </div>
              )}

              {/* All loaded message */}
              {!hasMore && displayedPools.length > 0 && (
                <div className="mt-10 text-center">
                  <p className="text-slate-500 text-sm">
                    🎉 You&apos;ve seen all {filteredPools.length} pools in this
                    category
                  </p>
                </div>
              )}

              {/* Intersection observer trigger */}
              <div ref={observerRef} className="h-4" />
            </>
          )}
        </div>
      </section>

      {/* Back to Home CTA */}
      <section className="px-4 sm:px-6 lg:px-8 pb-20">
        <div className="max-w-4xl mx-auto text-center bg-gradient-to-br from-slate-800/60 to-slate-900/60 backdrop-blur-sm border border-slate-700/30 rounded-3xl p-12">
          <h2 className="text-3xl font-bold text-white mb-4">
            Want to Create Your Own Pool?
          </h2>
          <p className="text-slate-400 mb-8 max-w-lg mx-auto">
            Launch a transparent donation pool on Stellar blockchain. Start
            collecting contributions with full transparency and DeFi yield.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <button className="bg-[#50C878] text-black px-8 py-3 rounded-lg font-semibold transition transform hover:scale-105 hover:shadow-lg hover:shadow-[#50C878]/20">
              Create a Pool
            </button>
            <Link
              href="/"
              className="border-2 border-slate-600 text-slate-300 hover:bg-slate-700 hover:text-white px-8 py-3 rounded-lg font-semibold transition"
            >
              Back to Home
            </Link>
          </div>
        </div>
      </section>

      <Footer />

      <style jsx>{`
        @keyframes fadeSlideUp {
          from {
            opacity: 0;
            transform: translateY(24px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
        @keyframes shimmer {
          0% {
            transform: translateX(-100%);
          }
          100% {
            transform: translateX(100%);
          }
        }
        .animate-shimmer {
          animation: shimmer 2s infinite;
        }
        .line-clamp-1 {
          display: -webkit-box;
          -webkit-line-clamp: 1;
          -webkit-box-orient: vertical;
          overflow: hidden;
        }
        .line-clamp-2 {
          display: -webkit-box;
          -webkit-line-clamp: 2;
          -webkit-box-orient: vertical;
          overflow: hidden;
        }
      `}</style>
    </div>
  );
}
