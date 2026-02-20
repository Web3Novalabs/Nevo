import DashboardHeader from "@/components/DashboardHeader";
import { PoolGrid } from "@/components/PoolGrid";

export default function DashboardPage() {
    return (
        <div className="min-h-screen bg-[#0F172A]">
            <DashboardHeader />

            <main className="mx-auto max-w-7xl px-4 pb-20 pt-10 sm:px-6 lg:px-8">
                {/* Page heading */}
                <section className="mb-10">
                    <h1 className="text-3xl font-extrabold text-white">Dashboard</h1>
                    <p className="mt-1 text-sm text-slate-400">
                        Manage your donation pools and track contributions.
                    </p>
                </section>

                {/* Pools */}
                <section>
                    <h2 className="mb-6 text-lg font-semibold text-slate-200">
                        Active Pools
                    </h2>
                    <PoolGrid />
                </section>
            </main>
        </div>
    );
}
