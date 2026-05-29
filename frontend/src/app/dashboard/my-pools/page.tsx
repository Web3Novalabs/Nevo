import Link from "next/link"
import { PoolCard } from "@/components/PoolCard"

const DASHBOARD_POOLS = [
  {
    id: "my-1",
    title: "Community School Supplies",
    description: "Providing notebooks, backpacks, and uniforms for students ahead of the new term.",
    imageUrl:
      "https://images.unsplash.com/photo-1503676260728-1c00da094a0b?auto=format&fit=crop&q=80&w=800",
    goalAmount: 12000,
    raisedAmount: 6400,
    donorCount: 76,
    creatorName: "You",
    creatorAvatarUrl: "https://i.pravatar.cc/120?img=15",
    status: "Open" as const,
  },
]

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
          className="inline-flex items-center gap-2 rounded-xl bg-linear-to-r from-emerald-500 to-cyan-500 px-5 py-2.5 text-sm font-semibold text-white shadow-lg shadow-emerald-500/30 transition-all duration-200 hover:brightness-110 hover:shadow-emerald-500/50 active:scale-95"
        >
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
          </svg>
          Create Pool
        </Link>
      </header>

      <section className="grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3">
        {DASHBOARD_POOLS.map((pool) => (
          <PoolCard key={pool.id} {...pool} />
        ))}
      </section>
    </div>
  )
}
