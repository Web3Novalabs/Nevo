"use client";

import Link from "next/link";
import { FolderOpen, Plus } from "lucide-react";
import { EmptyState } from "@/components/ui/EmptyState";


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
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
          </svg>
          Create Pool
        </Link>
      </header>

      <EmptyState
        title="No pools yet"
        description="Get started by creating your first donation pool to receive transparent contributions."
        icon={FolderOpen}
        suggestions={[
          "Click 'Create Pool' to initialize the creation wizard",
          "Fill in your pool's title, description, category, and target goal",
          "Connect your Stellar wallet and authorize the smart contract creation"
        ]}
        action={{
          label: "Create Pool",
          href: "/dashboard/pools/create",
          icon: Plus,
        }}
      />
    </div>
  );
}
