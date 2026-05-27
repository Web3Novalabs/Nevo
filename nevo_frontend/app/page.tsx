import Link from "next/link";
import ProgressTimeline from "@/components/ProgressTimeline";

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col px-6 py-12">
      <section className="mx-auto w-full max-w-5xl rounded-3xl border border-zinc-200 bg-white px-6 py-16 shadow-sm shadow-zinc-200/50 dark:border-zinc-800 dark:bg-zinc-950 dark:shadow-black/10 sm:px-10">
        <div className="text-center">
          <h1 className="text-4xl font-bold tracking-tight text-zinc-900 dark:text-zinc-50 sm:text-5xl">
            Fundraising on Stellar
          </h1>
          <p className="mx-auto mt-4 max-w-2xl text-lg text-zinc-600 dark:text-zinc-400">
            Nevo lets you create transparent, on-chain donation pools. Every
            contribution is verifiable, every withdrawal is trustless.
          </p>
          <div className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row">
            <Link
              href="/pools"
              className="rounded-full bg-zinc-900 px-6 py-3 text-sm font-medium text-white hover:bg-zinc-700 dark:bg-zinc-50 dark:text-zinc-900 dark:hover:bg-zinc-200 transition-colors"
            >
              Browse Pools
            </Link>
            <Link
              href="/pools/new"
              className="rounded-full border border-zinc-300 px-6 py-3 text-sm font-medium text-zinc-900 hover:bg-zinc-50 dark:border-zinc-700 dark:text-zinc-50 dark:hover:bg-zinc-900 transition-colors"
            >
              Create a Pool
            </Link>
          </div>
        </div>
      </section>

      <section className="mx-auto mt-12 w-full max-w-5xl">
        <ProgressTimeline
          title="Community Donation Pool"
          subtitle="Track funding progress and follow the pool launch process."
          currentAmount={28500}
          targetAmount={50000}
          currencySymbol="$"
          milestones={[
            { label: "Seed goal", value: "$15k", status: "complete" },
            { label: "Halfway", value: "$25k", status: "complete" },
            { label: "Launch target", value: "$50k", status: "current" },
          ]}
          steps={[
            {
              title: "Pool created",
              description: "Donation pool is live and ready to accept contributions.",
              status: "complete",
            },
            {
              title: "First milestone",
              description: "Reach the initial funding goal to unlock the next phase.",
              status: "complete",
            },
            {
              title: "Final push",
              description: "Collect remaining funds and prepare for distribution.",
              status: "current",
            },
            {
              title: "Funds distributed",
              description: "Complete the payout once the pool goal is met.",
              status: "upcoming",
            },
          ]}
        />
      </section>
    </main>
  );
}
