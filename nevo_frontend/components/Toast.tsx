//Toast
'use client';

import React, { useEffect, useState, useCallback, useRef } from 'react';
import { CheckIcon, XIcon, InfoIcon } from '@/components/icons';

export type ToastType = 'success' | 'error' | 'info';

export interface ToastMessage {
  id: string;
  message: string;
  type: ToastType;
}

interface ToastItemProps {
  toast: ToastMessage;
  onDismiss: (id: string) => void;
}

let toastIdCounter = 0;
let addToastFn: ((message: string, type?: ToastType) => void) | null = null;

export function toast(message: string, type: ToastType = 'success') {
  addToastFn?.(message, type);
}

function ToastItem({ toast: t, onDismiss }: ToastItemProps) {
  const [isHovered, setIsHovered] = useState(false);
  const [isFocused, setIsFocused] = useState(false);

  const isPaused = isHovered || isFocused;

  useEffect(() => {
    if (isPaused) return;
    const timer = setTimeout(() => onDismiss(t.id), 3000);
    return () => clearTimeout(timer);
  }, [t.id, onDismiss, isPaused]);

  const bgColor = {
    success: 'bg-green-600',
    error: 'bg-red-600',
    info: 'bg-blue-600',
  };

  const icon = {
    success: <CheckIcon className="size-4 shrink-0" />,
    error: <XIcon className="size-4 shrink-0" />,
    info: <InfoIcon className="size-4 shrink-0" />,
  };

  return (
    <div
      role="status"
      aria-live="polite"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      onFocus={() => setIsFocused(true)}
      onBlur={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget as Node)) {
          setIsFocused(false);
        }
      }}
      className={`pointer-events-auto flex items-center gap-2 rounded-lg ${bgColor[t.type]} px-4 py-2.5 text-sm font-medium text-white shadow-lg transition-all duration-300`}
    >
      {icon[t.type]}
      <span>{t.message}</span>
      <button
        type="button"
        onClick={() => onDismiss(t.id)}
        className="ml-2 rounded p-0.5 hover:bg-white/20 transition-colors"
        aria-label="Dismiss notification"
      >
        <XIcon className="size-3" />
      </button>
    </div>
  );
}

export function ToastContainer() {
  const [toasts, setToasts] = useState<ToastMessage[]>([]);
  const initialized = useRef(false);

  const addToast = useCallback(
    (message: string, type: ToastType = 'success') => {
      const id = String(++toastIdCounter);
      setToasts((prev) => [...prev, { id, message, type }]);
    },
    []
  );

  const dismiss = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  useEffect(() => {
    if (!initialized.current) {
      addToastFn = addToast;
      initialized.current = true;
    }
    return () => {
      addToastFn = null;
    };
  }, [addToast]);

  if (toasts.length === 0) return null;

  return (
    <div
      aria-label="Notifications"
      className="pointer-events-none fixed bottom-5 left-1/2 z-50 flex -translate-x-1/2 flex-col items-center gap-2"
    >
      {toasts.map((t) => (
        <ToastItem key={t.id} toast={t} onDismiss={dismiss} />
      ))}
    </div>
  );
}
