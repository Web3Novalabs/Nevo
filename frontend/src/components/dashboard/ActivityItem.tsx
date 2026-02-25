import React from "react";
import { Heart, Plus, Trophy } from "lucide-react";
import { ActivityData } from "./DashboardContext";

export function ActivityItem({ activity }: { activity: ActivityData }) {
    const getActivityDetails = () => {
        switch (activity.type) {
            case "donation":
                return {
                    icon: <Heart className="h-4 w-4 text-rose-400" />,
                    bg: "bg-rose-500/10 text-rose-500",
                    text: (
                        <>
                            <span className="font-medium text-slate-200">{activity.user}</span> donated{" "}
                            <span className="font-semibold text-emerald-400">{activity.amount}</span> to{" "}
                            <span className="font-medium text-indigo-300">{activity.poolName}</span>
                        </>
                    ),
                };
            case "pool_created":
                return {
                    icon: <Plus className="h-4 w-4 text-indigo-400" />,
                    bg: "bg-indigo-500/10 text-indigo-500",
                    text: (
                        <>
                            <span className="font-medium text-slate-200">{activity.user}</span> created a new pool:{" "}
                            <span className="font-medium text-indigo-300">{activity.poolName}</span>
                        </>
                    ),
                };
            case "reward":
                return {
                    icon: <Trophy className="h-4 w-4 text-amber-400" />,
                    bg: "bg-amber-500/10 text-amber-500",
                    text: (
                        <>
                            <span className="font-medium text-slate-200">{activity.user}</span> earned a reward of{" "}
                            <span className="font-semibold text-amber-400">{activity.amount}</span>
                        </>
                    ),
                };
        }
    };

    const details = getActivityDetails();

    return (
        <div className="flex items-start gap-4 p-4 transition-colors hover:bg-slate-800/30 rounded-lg">
            <div className={`mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-full ${details.bg}`}>
                {details.icon}
            </div>
            <div className="flex-1 space-y-1">
                <p className="text-sm text-slate-300">{details.text}</p>
                <p className="text-xs text-slate-500">{activity.timestamp}</p>
            </div>
        </div>
    );
}

export function ActivityItemSkeleton() {
    return (
        <div className="flex items-start gap-4 p-4 animate-pulse">
            <div className="h-8 w-8 shrink-0 rounded-full bg-slate-700/50"></div>
            <div className="flex-1 space-y-2 py-1">
                <div className="h-4 w-3/4 rounded bg-slate-700/50"></div>
                <div className="h-3 w-20 rounded bg-slate-700/50"></div>
            </div>
        </div>
    );
}
