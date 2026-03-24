import { DashboardStats } from "@/components/dashboard/DashboardStats";
import { ActivityFeed } from "@/components/dashboard/ActivityFeed";

export default function DashboardOverviewPage() {
  return (
    <div className="space-y-8">
      <header>
        <h1 className="text-3xl font-bold tracking-tight text-white">
          Overview
        </h1>
        <p className="mt-2 text-slate-400">
          Welcome to your Nevo dashboard. Track your pools and contributions here.
        </p>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <div className="lg:col-span-2 space-y-8">
          <section>
            <DashboardStats />
          </section>
        </div>

        <div className="lg:col-span-1 h-[400px]">
          <ActivityFeed />
        </div>
      </div>
    </div>
  );
}
