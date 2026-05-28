import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center justify-center font-medium whitespace-nowrap transition-colors",
  {
    variants: {
      variant: {
        default:  "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300",
        primary:  "bg-primary/10 text-primary dark:bg-primary/20",
        success:  "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400",
        warning:  "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400",
        danger:   "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400",
        info:     "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400",
      },
      size: {
        sm: "text-xs px-2 py-0.5",
        md: "text-xs px-2.5 py-0.5",
        lg: "text-sm px-3 py-1",
      },
      shape: {
        pill: "rounded-full",
        rect: "rounded-md",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "md",
      shape: "pill",
    },
  }
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof badgeVariants> {}

const Badge = React.forwardRef<HTMLSpanElement, BadgeProps>(
  ({ className, variant, size, shape, ...props }, ref) => (
    <span
      ref={ref}
      data-slot="badge"
      className={cn(badgeVariants({ variant, size, shape }), className)}
      {...props}
    />
  )
);
Badge.displayName = "Badge";

export { Badge, badgeVariants };
