"use client"

import React, { useEffect, useRef } from "react"
import { cva, type VariantProps } from "class-variance-authority"
import {
  CheckCircle2,
  XCircle,
  AlertTriangle,
  Info,
  X,
} from "lucide-react"
import { cn } from "@/lib/utils"
import type { Toast as ToastType } from "./ToastContext"

const toastVariants = cva(
  "relative flex w-full max-w-sm gap-3 rounded-lg border p-4 shadow-lg transition-all duration-300 animate-in slide-in-from-right-5 fade-in-0",
  {
    variants: {
      variant: {
        success:
          "bg-green-50 border-green-200 text-green-900 dark:bg-green-900/20 dark:border-green-800 dark:text-green-100",
        error:
          "bg-red-50 border-red-200 text-red-900 dark:bg-red-900/20 dark:border-red-800 dark:text-red-100",
        warning:
          "bg-amber-50 border-amber-200 text-amber-900 dark:bg-amber-900/20 dark:border-amber-800 dark:text-amber-100",
        info: "bg-blue-50 border-blue-200 text-blue-900 dark:bg-blue-900/20 dark:border-blue-800 dark:text-blue-100",
      },
    },
    defaultVariants: { variant: "info" },
  }
)

const iconMap = {
  success: <CheckCircle2 className="size-5 shrink-0 text-green-600 dark:text-green-400" aria-hidden="true" />,
  error: <XCircle className="size-5 shrink-0 text-red-600 dark:text-red-400" aria-hidden="true" />,
  warning: <AlertTriangle className="size-5 shrink-0 text-amber-600 dark:text-amber-400" aria-hidden="true" />,
  info: <Info className="size-5 shrink-0 text-blue-600 dark:text-blue-400" aria-hidden="true" />,
}

const DEFAULT_DURATION = 5000

interface ToastProps extends VariantProps<typeof toastVariants> {
  toast: ToastType
  onDismiss: (id: string) => void
}

export function Toast({ toast, onDismiss }: ToastProps) {
  const { id, variant, title, description, duration = DEFAULT_DURATION, action } = toast
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    if (duration === 0) return
    timerRef.current = setTimeout(() => onDismiss(id), duration)
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current)
    }
  }, [id, duration, onDismiss])

  return (
    <div
      role="alert"
      aria-live="assertive"
      aria-atomic="true"
      className={cn(toastVariants({ variant }))}
    >
      {iconMap[variant ?? "info"]}

      <div className="flex-1 min-w-0">
        <p className="text-sm font-semibold leading-snug">{title}</p>
        {description && (
          <p className="mt-0.5 text-sm opacity-80 leading-snug">{description}</p>
        )}
        {action && (
          <button
            onClick={action.onClick}
            className="mt-2 text-xs font-medium underline underline-offset-2 hover:opacity-70 transition-opacity"
          >
            {action.label}
          </button>
        )}
      </div>

      <button
        onClick={() => onDismiss(id)}
        aria-label="Dismiss notification"
        className="shrink-0 self-start rounded p-0.5 opacity-60 hover:opacity-100 transition-opacity focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <X className="size-4" />
      </button>
    </div>
  )
}
