import React from "react";
import { Banknote, Droplets, Activity, type LucideIcon } from "lucide-react";

const iconMap: Record<string, LucideIcon> = {
    Banknotes: Banknote,
    Droplets: Droplets,
    Activity: Activity,
};

interface StatCardProps {
    title: string;
    value: string;
    iconName: string;
    trend?: string;
}

export function StatCard({ title, value, iconName, trend }: StatCardProps) {
    const Icon = iconMap[iconName] || Activity;

    return (
        <div className="rounded-xl border border-slate-800/80 bg-slate-900/50 p-6 backdrop-blur-sm transition-all hover:bg-slate-800/50">
            <div className="flex items-center justify-between">
                <h3 className="text-sm font-medium text-slate-400">{title}</h3>
                <div className="rounded-md bg-indigo-500/10 p-2">
                    <Icon className="h-5 w-5 text-indigo-400" />
                </div>
            </div>
            <div className="mt-4 flex items-baseline gap-2">
                <span className="text-2xl font-bold tracking-tight text-white">{value}</span>
                {trend && (
                    <span className={`text-xs font-medium ${trend.startsWith("+") ? "text-emerald-400" : "text-rose-400"}`}>
                        {trend}
                    </span>
                )}
            </div>
        </div>
    );
}

export function StatCardSkeleton() {
    return (
        <div className="rounded-xl border border-slate-800/50 bg-slate-900/20 p-6 backdrop-blur-sm animate-pulse">
            <div className="flex items-center justify-between">
                <div className="h-4 w-24 rounded bg-slate-700/50"></div>
                <div className="h-9 w-9 rounded-md bg-slate-700/50"></div>
            </div>
            <div className="mt-4 flex items-baseline gap-2">
                <div className="h-8 w-32 rounded bg-slate-700/50"></div>
                <div className="h-4 w-12 rounded bg-slate-700/50"></div>
            </div>
        </div>
    );
}
