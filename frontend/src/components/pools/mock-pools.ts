import type { PoolCardProps, PoolStatus } from "@/components/PoolCard";

export type PoolSort = "newest" | "most-funded" | "close-to-goal" | "trending";

export interface DiscoverablePool extends PoolCardProps {
  createdAt: string;
  trendingScore: number;
}

export const POOL_CATEGORIES = [
  "Healthcare",
  "Environment",
  "Education",
  "Water",
  "Community",
  "Emergency",
] as const;

export const POOL_STATUSES: PoolStatus[] = ["open", "closed"];

const images = [
  "1541249591-6284fcdbf769",
  "1577896851231-70ef18881754",
  "1542601906990-b4d3fb778b09",
  "1538108149393-fbbd81895907",
  "1460661419201-fd4cecdf8a8b",
  "1588680145224-811c751270ae",
];

const poolSeeds = [
  ["clean-water-wells", "Clean Water Wells Project", "Water", 50000, 39200, 286, "open", 96],
  ["rural-clinic-expansion", "Rural Clinic Expansion Fund", "Healthcare", 75000, 48250, 312, "open", 88],
  ["girls-stem-labs", "Girls STEM Labs", "Education", 30000, 25800, 174, "open", 91],
  ["urban-reforestation", "Urban Reforestation", "Environment", 22000, 13400, 121, "open", 73],
  ["food-bank-network", "Food Bank Network", "Community", 18000, 16750, 244, "open", 84],
  ["emergency-relief-reserve", "Emergency Relief Reserve", "Emergency", 100000, 100000, 803, "closed", 99],
  ["mental-health-hub", "Mental Health Support Hub", "Healthcare", 42000, 19100, 138, "open", 67],
  ["solar-classroom-kit", "Solar Classroom Kit", "Education", 18000, 18000, 147, "closed", 78],
  ["river-restoration", "River Restoration Fund", "Water", 64000, 40100, 229, "open", 82],
  ["climate-resilience", "Climate Resilience Fund", "Environment", 90000, 54800, 341, "open", 95],
  ["community-arts-center", "Community Arts Center", "Community", 12000, 9900, 198, "open", 76],
  ["medical-supplies-drive", "Medical Supplies Drive", "Emergency", 36000, 33100, 408, "open", 93],
] as const;

export const MOCK_POOLS: DiscoverablePool[] = poolSeeds.map((seed, index) => {
  const [id, title, category, goal, raised, donors, status, trendingScore] = seed;

  return {
    id,
    title,
    category,
    description: `Support this ${category.toLowerCase()} pool with transparent contributions tracked on Stellar for donors and organizers.`,
    goalAmount: goal,
    raisedAmount: raised,
    donorCount: donors,
    status,
    trendingScore,
    createdAt: `2026-05-${String(26 - index).padStart(2, "0")}`,
    imageUrl: `https://images.unsplash.com/photo-${images[index % images.length]}?auto=format&fit=crop&q=80&w=800`,
    creator: {
      name: [
        "Nevo Impact Guild",
        "Field Partners DAO",
        "Stellar Giving Circle",
        "Community Trust",
      ][index % 4],
      handle: `G${id.slice(0, 4).toUpperCase()}...${String(index + 11).padStart(2, "0")}`,
    },
  };
});

export function sortPools(sortBy: PoolSort) {
  return (a: DiscoverablePool, b: DiscoverablePool) => {
    if (sortBy === "most-funded") return b.raisedAmount - a.raisedAmount;
    if (sortBy === "close-to-goal") {
      return b.raisedAmount / b.goalAmount - a.raisedAmount / a.goalAmount;
    }
    if (sortBy === "trending") return b.trendingScore - a.trendingScore;
    return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
  };
}
