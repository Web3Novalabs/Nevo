"use client"

import React, { createContext, useCallback, useContext, useReducer } from "react"

export type ToastVariant = "success" | "error" | "warning" | "info"
export type ToastPosition =
  | "top-right"
  | "top-left"
  | "top-center"
  | "bottom-right"
  | "bottom-left"
  | "bottom-center"

export interface ToastAction {
  label: string
  onClick: () => void
}

export interface Toast {
  id: string
  variant: ToastVariant
  title: string
  description?: string
  duration?: number // ms; 0 = persistent
  action?: ToastAction
  position?: ToastPosition
}

export type ToastInput = Omit<Toast, "id">

type Action =
  | { type: "ADD"; toast: Toast }
  | { type: "REMOVE"; id: string }

function reducer(state: Toast[], action: Action): Toast[] {
  switch (action.type) {
    case "ADD":
      return [...state, action.toast]
    case "REMOVE":
      return state.filter((t) => t.id !== action.id)
    default:
      return state
  }
}

interface ToastContextValue {
  toasts: Toast[]
  toast: (input: ToastInput) => string
  dismiss: (id: string) => void
}

const ToastContext = createContext<ToastContextValue | null>(null)

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, dispatch] = useReducer(reducer, [])

  const toast = useCallback((input: ToastInput): string => {
    const id = Math.random().toString(36).slice(2)
    dispatch({ type: "ADD", toast: { ...input, id } })
    return id
  }, [])

  const dismiss = useCallback((id: string) => {
    dispatch({ type: "REMOVE", id })
  }, [])

  return (
    <ToastContext.Provider value={{ toasts, toast, dismiss }}>
      {children}
    </ToastContext.Provider>
  )
}

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext)
  if (!ctx) throw new Error("useToast must be used within a ToastProvider")
  return ctx
}
