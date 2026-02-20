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

      <section className="rounded-xl border border-slate-800/80 bg-slate-900/50 p-6 backdrop-blur-sm">
        <h2 className="text-lg font-semibold text-white">Quick stats</h2>
        <p className="mt-2 text-sm text-slate-500">
          Stats and charts will appear in Part 2.
        </p>
      </section>
    </div>
  );
}
