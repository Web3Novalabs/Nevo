import Link from "next/link";
import { ArrowLeft } from "lucide-react";

import Footer from "@/components/Footer";
import Navigation from "@/components/Navigation";

interface PoolDetailPageProps {
  params: Promise<{
    poolId: string;
  }>;
}

export default async function PoolDetailPage({ params }: PoolDetailPageProps) {
  const { poolId } = await params;
  const readableTitle = poolId
    .split("-")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");

  return (
    <div className="min-h-screen bg-[#0F172A] text-white">
      <Navigation />
      <main className="mx-auto flex min-h-[70vh] max-w-4xl flex-col justify-center px-4 py-28 sm:px-6 lg:px-8">
        <Link
          href="/explore"
          className="mb-8 inline-flex w-fit items-center gap-2 text-sm font-semibold text-emerald-300 transition hover:text-white"
        >
          <ArrowLeft className="h-4 w-4" aria-hidden="true" />
          Back to pools
        </Link>

        <section className="rounded-xl border border-slate-800 bg-slate-900/70 p-8 shadow-xl shadow-slate-950/30">
          <p className="mb-3 text-sm font-semibold uppercase tracking-wide text-emerald-300">
            Pool detail
          </p>
          <h1 className="text-3xl font-bold tracking-tight sm:text-4xl">
            {readableTitle || "Donation Pool"}
          </h1>
          <p className="mt-4 max-w-2xl text-slate-400">
            This detail page is ready for live pool data. The PoolCard links here
            with the pool identifier so listings and dashboards can navigate to
            a dedicated view.
          </p>
        </section>
      </main>
      <Footer />
    </div>
  );
}
