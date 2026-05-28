import Link from "next/link";
import { Plus } from "lucide-react";

import { PoolCard, type PoolCardProps } from "@/components/PoolCard";

const MY_POOLS: PoolCardProps[] = [
  {
    id: "rural-clinic-expansion",
    title: "Rural Clinic Expansion Fund",
    description:
      "Expanding a mobile clinic route with transparent Stellar payouts for medicine, fuel, and field staff.",
    category: "Healthcare",
    imageUrl:
      "https://images.unsplash.com/photo-1538108149393-fbbd81895907?auto=format&fit=crop&q=80&w=800",
    goalAmount: 75000,
    raisedAmount: 48250,
    donorCount: 312,
    creator: {
      name: "Nevo Health Ops",
      handle: "GHOP...MED",
    },
    status: "open",
  },
  {
    id: "solar-classroom-kit",
    title: "Solar Classroom Kit",
    description:
      "Funding portable solar kits so community classrooms can keep laptops and learning tools online.",
    category: "Education",
    imageUrl:
      "https://images.unsplash.com/photo-1577896851231-70ef18881754?auto=format&fit=crop&q=80&w=800",
    goalAmount: 18000,
    raisedAmount: 18000,
    donorCount: 147,
    creator: {
      name: "Nevo Education Guild",
      handle: "GEDU...SUN",
    },
    status: "closed",
  },
];

export default function MyPoolsPage() {
  return (
    <div className="space-y-8">
      <header className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-white">
            My Pools
          </h1>
          <p className="mt-2 text-slate-400">
            View and manage your donation pools.
          </p>
        </div>
        <Link
          href="/dashboard/pools/create"
          className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-emerald-500 to-cyan-500 px-5 py-2.5 text-sm font-semibold text-white shadow-lg shadow-emerald-500/30 transition-all duration-200 hover:brightness-110 hover:shadow-emerald-500/50 active:scale-95"
        >
          <Plus className="h-4 w-4" aria-hidden="true" />
          Create Pool
        </Link>
      </header>

      <section className="grid grid-cols-1 gap-6 xl:grid-cols-2">
        {MY_POOLS.map((pool) => (
          <PoolCard key={pool.id} {...pool} />
        ))}
      </section>
    </div>
  );
}
