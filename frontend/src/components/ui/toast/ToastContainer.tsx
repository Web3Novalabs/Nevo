"use client"

import React from "react"
import { createPortal } from "react-dom"
import { cn } from "@/lib/utils"
import { useToast, type ToastPosition } from "./ToastContext"
import { Toast } from "./Toast"

const positionClasses: Record<ToastPosition, string> = {
  "top-right": "top-4 right-4 items-end",
  "top-left": "top-4 left-4 items-start",
  "top-center": "top-4 left-1/2 -translate-x-1/2 items-center",
  "bottom-right": "bottom-4 right-4 items-end",
  "bottom-left": "bottom-4 left-4 items-start",
  "bottom-center": "bottom-4 left-1/2 -translate-x-1/2 items-center",
}

const DEFAULT_POSITION: ToastPosition = "top-right"

export function ToastContainer() {
  const { toasts, dismiss } = useToast()

  if (typeof window === "undefined") return null

  // Group toasts by position
  const groups = toasts.reduce<Record<ToastPosition, typeof toasts>>(
    (acc, t) => {
      const pos = t.position ?? DEFAULT_POSITION
      acc[pos] = [...(acc[pos] ?? []), t]
      return acc
    },
    {} as Record<ToastPosition, typeof toasts>
  )

  return createPortal(
    <>
      {(Object.entries(groups) as [ToastPosition, typeof toasts][]).map(
        ([position, group]) => (
          <div
            key={position}
            aria-label="Notifications"
            className={cn(
              "fixed z-50 flex flex-col gap-2 w-full max-w-sm pointer-events-none",
              // On mobile, always bottom-center full-width
              "max-sm:bottom-4 max-sm:left-0 max-sm:right-0 max-sm:top-auto max-sm:translate-x-0 max-sm:items-stretch max-sm:px-4",
              `sm:${positionClasses[position]}`
            )}
          >
            {group.map((t) => (
              <div key={t.id} className="pointer-events-auto">
                <Toast toast={t} onDismiss={dismiss} />
              </div>
            ))}
          </div>
        )
      )}
    </>,
    document.body
  )
}
