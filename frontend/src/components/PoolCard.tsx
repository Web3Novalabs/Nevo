import Link from "next/link"
import Image from "next/image"
import { Users } from "lucide-react"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { StatusBadge } from "@/components/ui/status-badge"

type PoolStatus = "Open" | "Closed"

export interface PoolCardProps {
  id: string
  title: string
  description: string
  imageUrl: string
  goalAmount: number
  raisedAmount: number
  donorCount: number
  creatorName: string
  creatorAvatarUrl: string
  status: PoolStatus
}

export const PoolCard = ({
  id,
  title,
  description,
  imageUrl,
  goalAmount,
  raisedAmount,
  donorCount,
  creatorName,
  creatorAvatarUrl,
  status,
}: PoolCardProps) => {
  const progress = goalAmount > 0 ? Math.min((raisedAmount / goalAmount) * 100, 100) : 0

  return (
    <Link href={`/pools/${id}`} className="block h-full">
      <Card className="group flex h-full flex-col overflow-hidden border-slate-200/70 transition-all duration-300 hover:-translate-y-1 hover:shadow-xl dark:border-slate-700/70">
        <div className="relative h-44 w-full overflow-hidden">
          <Image
            src={imageUrl}
            alt={title}
            fill
            sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw"
            className="object-cover transition-transform duration-500 group-hover:scale-105"
          />
          <div className="absolute left-3 top-3">
            <StatusBadge variant={status === "Open" ? "success" : "default"}>
              {status}
            </StatusBadge>
          </div>
        </div>

        <CardHeader className="space-y-3 pb-4">
          <div className="flex items-center gap-3">
            <Image
              src={creatorAvatarUrl}
              alt={creatorName}
              width={34}
              height={34}
              className="h-8 w-8 rounded-full object-cover ring-2 ring-white dark:ring-slate-800"
            />
            <p className="text-sm text-slate-600 dark:text-slate-400">
              by <span className="font-semibold text-slate-800 dark:text-slate-200">{creatorName}</span>
            </p>
          </div>
          <CardTitle className="line-clamp-1 text-xl">{title}</CardTitle>
          <CardDescription className="line-clamp-2">{description}</CardDescription>
        </CardHeader>

        <CardContent className="mt-auto space-y-4">
          <div className="flex items-end justify-between gap-2 text-sm">
            <div>
              <p className="text-slate-500 dark:text-slate-400">Donated</p>
              <p className="text-lg font-bold text-slate-900 dark:text-white">${raisedAmount.toLocaleString()}</p>
            </div>
            <p className="text-right font-medium text-slate-700 dark:text-slate-300">
              Goal: ${goalAmount.toLocaleString()}
            </p>
          </div>

          <div className="space-y-2">
            <div className="h-2.5 w-full overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700">
              <div
                className="h-full rounded-full bg-linear-to-r from-blue-500 to-cyan-400 transition-all duration-700"
                style={{ width: `${progress}%` }}
              />
            </div>
            <div className="flex items-center justify-between text-xs">
              <span className="font-semibold text-blue-600 dark:text-cyan-400">{Math.round(progress)}% funded</span>
              <span className="flex items-center gap-1 text-slate-500 dark:text-slate-400">
                <Users size={14} />
                {donorCount} donors
              </span>
            </div>
          </div>
        </CardContent>
      </Card>
    </Link>
  )
}
