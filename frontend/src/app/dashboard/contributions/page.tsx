"use client";

import { HeartHandshake, Compass } from "lucide-react";
import { EmptyState } from "@/components/ui/EmptyState";

export default function ContributionsPage() {
  const hasContributions = false;

  return (
    <div className="space-y-8">
      <header>
        <h1 className="text-3xl font-bold tracking-tight text-white">
          Contributions
        </h1>
        <p className="mt-2 text-slate-400">
          Track your contribution history across pools.
        </p>
      </header>

      {!hasContributions && (
        <EmptyState
          title="No contributions yet"
          description="You haven't made any donations yet. Browse active pools and make your first contribution to support transparent fundraising."
          icon={HeartHandshake}
          suggestions={[
            "Explore active pools using the 'Discover Pools' button",
            "Choose a cause you want to support (Health, Environment, etc.)",
            "Connect your Stellar wallet and donate XLM or USDC directly"
          ]}
          action={{
            label: "Discover Pools",
            href: "/explore",
            icon: Compass,
          }}
        />
      )}
    </div>
  );
}
