import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "@/lib/utils"

const statusBadgeVariants = cva(
  "inline-flex items-center justify-center text-xs font-medium rounded-md px-2.5 py-0.5 whitespace-nowrap transition-colors",
  {
    variants: {
      variant: {
        success:
          "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400",
        warning:
          "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400",
        info: "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400",
        danger:
          "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400",
        default:
          "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-400",
      },
    },
    defaultVariants: {
      variant: "info",
    },
  }
)

export interface StatusBadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof statusBadgeVariants> {}

function StatusBadge({ className, variant, ...props }: StatusBadgeProps) {
  return (
    <span
      data-slot="status-badge"
      data-variant={variant}
      className={cn(statusBadgeVariants({ variant }), className)}
      {...props}
    />
  )
}

export { StatusBadge, statusBadgeVariants }
