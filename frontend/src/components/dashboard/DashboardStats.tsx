"use client";

import React from "react";
import { useDashboard } from "./DashboardContext";
import { StatCard, StatCardSkeleton } from "./StatCard";

export function DashboardStats() {
    const { stats, isLoading } = useDashboard();

    if (isLoading) {
        return (
            <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
                {[1, 2, 3].map((i) => (
                    <StatCardSkeleton key={i} />
                ))}
            </div>
        );
    }

    return (
        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {stats.map((stat, i) => (
                <StatCard key={i} {...stat} />
            ))}
        </div>
    );
}
