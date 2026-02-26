"use client";

import React from "react";
import { useDashboard } from "./DashboardContext";
import { ActivityItem, ActivityItemSkeleton } from "./ActivityItem";
import { History } from "lucide-react";

export function ActivityFeed() {
    const { activities, isLoading } = useDashboard();

    return (
        <div className="rounded-xl border border-slate-800/80 bg-slate-900/50 p-6 backdrop-blur-sm h-full flex flex-col">
            <div className="flex items-center gap-2 mb-6">
                <History className="h-5 w-5 text-indigo-400" />
                <h2 className="text-lg font-semibold text-white">Recent Activity</h2>
            </div>

            <div className="space-y-2 pr-2 overflow-y-auto flex-1 [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-slate-700/50 hover:[&::-webkit-scrollbar-thumb]:bg-slate-600 [&::-webkit-scrollbar-thumb]:rounded-full">
                {isLoading ? (
                    <>
                        {[1, 2, 3, 4].map((i) => (
                            <ActivityItemSkeleton key={i} />
                        ))}
                    </>
                ) : activities.length > 0 ? (
                    activities.map((activity) => (
                        <ActivityItem key={activity.id} activity={activity} />
                    ))
                ) : (
                    <p className="text-sm text-slate-500 py-4 text-center">No recent activity found.</p>
                )}
            </div>
        </div>
    );
}
