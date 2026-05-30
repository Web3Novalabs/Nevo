'use client';

import React, { FC, useEffect, useRef } from 'react';
import { Button } from './Button';

export interface ConfirmDialogProps {
  open: boolean;
  /** Dialog heading */
  title: string;
  /** Body message or node */
  message: React.ReactNode;
  /** Confirm button label (default: "Confirm") */
  confirmLabel?: string;
  /** Cancel button label (default: "Cancel") */
  cancelLabel?: string;
  /** Variant controls confirm button colour */
  variant?: 'danger' | 'primary';
  /** Whether the confirm action is in-flight */
  loading?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

const variantIcon: Record<
  NonNullable<ConfirmDialogProps['variant']>,
  React.ReactNode
> = {
  danger: (
    <svg
      className="w-6 h-6 text-[var(--color-error)]"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
      />
    </svg>
  ),
  primary: (
    <svg
      className="w-6 h-6 text-brand-500"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M9.879 7.519c1.171-1.025 3.071-1.025 4.242 0 1.172 1.025 1.172 2.687 0 3.712-.203.179-.43.326-.67.442-.745.361-1.45.999-1.45 1.827v.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 5.25h.008v.008H12v-.008Z"
      />
    </svg>
  ),
};

export const ConfirmDialog: FC<ConfirmDialogProps> = ({
  open,
  title,
  message,
  confirmLabel = 'Confirm',
  cancelLabel = 'Cancel',
  variant = 'danger',
  loading = false,
  onConfirm,
  onCancel,
}) => {
  const ref = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (open) {
      if (!el.open) el.showModal();
    } else {
      if (el.open) el.close();
    }
  }, [open]);

  // Close on backdrop click
  const handleClick = (e: React.MouseEvent<HTMLDialogElement>) => {
    if (e.target === ref.current) onCancel();
  };

  // Close on Escape (native dialog fires 'cancel')
  const handleCancel = (e: React.SyntheticEvent) => {
    e.preventDefault();
    onCancel();
  };

  return (
    <dialog
      ref={ref}
      onClick={handleClick}
      onCancel={handleCancel}
      className="fixed bottom-0 left-0 right-0 m-0 w-full max-w-md max-h-[100dvh] rounded-t-2xl sm:m-auto sm:bottom-auto sm:left-auto sm:right-auto sm:top-1/2 sm:-translate-y-1/2 sm:rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-0 shadow-xl backdrop:bg-black/50 backdrop:backdrop-blur-sm open:flex open:flex-col"
      aria-labelledby="confirm-dialog-title"
      aria-describedby="confirm-dialog-message"
    >
      <div className="flex flex-col gap-4 p-6">
        {/* Icon + title */}
        <div className="flex items-start gap-3">
          <span className="mt-0.5 shrink-0">{variantIcon[variant]}</span>
          <h2
            id="confirm-dialog-title"
            className="text-base font-semibold text-[var(--color-text)]"
          >
            {title}
          </h2>
        </div>

        {/* Message */}
        <p
          id="confirm-dialog-message"
          className="text-sm text-[var(--color-text-muted)] leading-relaxed pl-9"
        >
          {message}
        </p>
      </div>

      {/* Actions */}
      <div className="flex justify-end gap-3 border-t border-[var(--color-border)] px-6 py-4">
        <Button
          variant="outlined"
          size="small"
          onClick={onCancel}
          disabled={loading}
        >
          {cancelLabel}
        </Button>
        <Button
          variant={variant === 'danger' ? 'danger' : 'primary'}
          size="small"
          loading={loading}
          onClick={onConfirm}
        >
          {confirmLabel}
        </Button>
      </div>
    </dialog>
  );
};
