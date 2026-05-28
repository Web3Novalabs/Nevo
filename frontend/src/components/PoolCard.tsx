import Image from "next/image";
import Link from "next/link";
import { ArrowUpRight, Target, Users } from "lucide-react";

import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { StatusBadge } from "@/components/ui/status-badge";
import { cn } from "@/lib/utils";

export type PoolStatus = "open" | "closed";

export interface PoolCardProps {
  id: string;
  title: string;
  description: string;
  category?: string;
  imageUrl?: string;
  goalAmount: number;
  raisedAmount: number;
  donorCount: number;
  creator: {
    name: string;
    avatarUrl?: string;
    handle?: string;
  };
  status: PoolStatus;
  href?: string;
  assetCode?: string;
  className?: string;
}

const currencyFormatter = new Intl.NumberFormat("en-US", {
  maximumFractionDigits: 0,
});

function getInitials(name: string) {
  return name
    .split(" ")
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

export function PoolCard({
  id,
  title,
  description,
  category = "Donation Pool",
  imageUrl,
  goalAmount,
  raisedAmount,
  donorCount,
  creator,
  status,
  href,
  assetCode = "USDC",
  className,
}: PoolCardProps) {
  const safeGoal = Math.max(goalAmount, 0);
  const progressPercent =
    safeGoal === 0 ? 0 : Math.min((raisedAmount / safeGoal) * 100, 100);
  const detailHref = href ?? `/pools/${id}`;
  const statusVariant = status === "open" ? "success" : "default";
  const statusLabel = status === "open" ? "Open" : "Closed";

  return (
    <Link href={detailHref} className="block h-full focus:outline-none">
      <Card
        className={cn(
          "group flex h-full min-h-[460px] flex-col overflow-hidden rounded-xl border-slate-700/60 bg-slate-900/80 shadow-xl shadow-slate-950/20 transition-all duration-300 hover:-translate-y-1 hover:border-emerald-400/60 hover:shadow-emerald-500/10 focus-within:border-emerald-400/70",
          className
        )}
      >
        {imageUrl ? (
          <div className="relative h-44 w-full overflow-hidden bg-slate-800">
            <Image
              src={imageUrl}
              alt={title}
              fill
              sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw"
              className="object-cover transition-transform duration-500 group-hover:scale-105"
            />
            <div className="absolute inset-x-0 bottom-0 h-20 bg-gradient-to-t from-slate-950/70 to-transparent" />
          </div>
        ) : (
          <div className="h-3 w-full bg-gradient-to-r from-emerald-400 via-cyan-400 to-sky-500" />
        )}

        <CardHeader className="gap-4 p-5 pb-3">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <span className="rounded-md border border-emerald-400/30 bg-emerald-400/10 px-2.5 py-1 text-xs font-semibold text-emerald-300">
              {category}
            </span>
            <StatusBadge variant={statusVariant}>{statusLabel}</StatusBadge>
          </div>

          <div className="space-y-2">
            <CardTitle className="line-clamp-2 text-xl leading-tight text-white transition-colors group-hover:text-emerald-300">
              {title}
            </CardTitle>
            <CardDescription className="line-clamp-3 min-h-[60px] leading-relaxed text-slate-400">
              {description}
            </CardDescription>
          </div>
        </CardHeader>

        <CardContent className="flex flex-1 flex-col gap-5 p-5 pt-2">
          <div className="space-y-3">
            <div className="flex items-end justify-between gap-4">
              <div>
                <p className="text-xs font-medium uppercase text-slate-500">
                  Donated
                </p>
                <p className="text-2xl font-bold text-white">
                  {currencyFormatter.format(raisedAmount)} {assetCode}
                </p>
              </div>
              <div className="text-right">
                <p className="text-xs font-medium uppercase text-slate-500">
                  Goal
                </p>
                <p className="font-semibold text-slate-300">
                  {currencyFormatter.format(goalAmount)} {assetCode}
                </p>
              </div>
            </div>

            <div
              className="h-2.5 overflow-hidden rounded-full bg-slate-800"
              aria-label={`${Math.round(progressPercent)}% funded`}
            >
              <div
                className="h-full rounded-full bg-gradient-to-r from-emerald-400 to-cyan-400 transition-all duration-700"
                style={{ width: `${progressPercent}%` }}
              />
            </div>

            <div className="flex items-center justify-between text-sm">
              <span className="font-semibold text-emerald-300">
                {Math.round(progressPercent)}% funded
              </span>
              <span className="inline-flex items-center gap-1.5 text-slate-400">
                <Users className="h-4 w-4" aria-hidden="true" />
                {donorCount.toLocaleString()} donors
              </span>
            </div>
          </div>

          <div className="mt-auto grid grid-cols-2 gap-3 rounded-lg border border-slate-800 bg-slate-950/35 p-3 text-sm">
            <div className="inline-flex items-center gap-2 text-slate-300">
              <Target className="h-4 w-4 text-cyan-300" aria-hidden="true" />
              <span>{currencyFormatter.format(goalAmount)} goal</span>
            </div>
            <div className="text-right font-medium text-slate-400">
              {status === "open" ? "Accepting donations" : "Pool closed"}
            </div>
          </div>
        </CardContent>

        <CardFooter className="justify-between gap-4 border-t border-slate-800 p-5">
          <div className="flex min-w-0 items-center gap-3">
            <div className="relative flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-full border border-slate-700 bg-slate-800 text-sm font-bold text-emerald-300">
              {creator.avatarUrl ? (
                <Image
                  src={creator.avatarUrl}
                  alt={creator.name}
                  fill
                  sizes="40px"
                  className="object-cover"
                />
              ) : (
                getInitials(creator.name)
              )}
            </div>
            <div className="min-w-0">
              <p className="truncate text-sm font-semibold text-white">
                {creator.name}
              </p>
              {creator.handle && (
                <p className="truncate text-xs text-slate-500">
                  {creator.handle}
                </p>
              )}
            </div>
          </div>

          <span className="inline-flex shrink-0 items-center gap-1.5 text-sm font-semibold text-emerald-300">
            Details
            <ArrowUpRight
              className="h-4 w-4 transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5"
              aria-hidden="true"
            />
          </span>
        </CardFooter>
      </Card>
    </Link>
  );
}
