import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

const tagVariants = cva(
  "inline-flex items-center gap-1.5 font-medium whitespace-nowrap transition-colors",
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
        sm: "text-xs px-2 py-0.5 rounded-md",
        md: "text-xs px-2.5 py-1 rounded-md",
        lg: "text-sm px-3 py-1.5 rounded-lg",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "md",
    },
  }
);

export interface TagProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof tagVariants> {
  /** Icon rendered before the label */
  icon?: React.ReactNode;
  /** Shows an X button; fires this callback when clicked */
  onRemove?: () => void;
  /** Accessible label for the remove button */
  removeLabel?: string;
}

const Tag = React.forwardRef<HTMLSpanElement, TagProps>(
  (
    {
      className,
      variant,
      size,
      icon,
      onRemove,
      removeLabel = "Remove",
      children,
      ...props
    },
    ref
  ) => (
    <span
      ref={ref}
      data-slot="tag"
      className={cn(tagVariants({ variant, size }), className)}
      {...props}
    >
      {icon && <span className="shrink-0 [&>svg]:size-3.5" aria-hidden="true">{icon}</span>}
      {children}
      {onRemove && (
        <button
          type="button"
          onClick={onRemove}
          aria-label={removeLabel}
          className="shrink-0 rounded-sm opacity-60 hover:opacity-100 transition-opacity focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-current"
        >
          <X className="size-3" />
        </button>
      )}
    </span>
  )
);
Tag.displayName = "Tag";

export { Tag, tagVariants };
