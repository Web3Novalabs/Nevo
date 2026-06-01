'use client';

import React, {
  FC,
  HTMLAttributes,
  KeyboardEvent,
  ReactNode,
  useCallback,
  useEffect,
  useId,
  useRef,
} from 'react';
import { createPortal } from 'react-dom';

export interface ModalProps {
  /** Whether the modal is open */
  open: boolean;
  /** Callback fired when the modal requests to be closed */
  onClose: () => void;
  /** Modal heading */
  title?: ReactNode;
  /** Content rendered in the body section */
  children?: ReactNode;
  /** Content rendered in the footer section */
  footer?: ReactNode;
  /**
   * If true, clicking the backdrop does NOT close the modal.
   * Useful for destructive-action confirmations. Defaults to false.
   */
  disableBackdropClose?: boolean;
  /** Extra class names applied to the dialog panel */
  className?: string;
}

const FOCUSABLE =
  'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])';

export const Modal: FC<ModalProps> = ({
  open,
  onClose,
  title,
  children,
  footer,
  disableBackdropClose = false,
  className = '',
}) => {
  const dialogRef = useRef<HTMLDivElement>(null);
  const titleId = useId();
  const descId = useId();

  // ── Focus trap ────────────────────────────────────────────────────────────
  const trapFocus = useCallback((e: globalThis.KeyboardEvent) => {
    if (!dialogRef.current) return;
    const focusable = Array.from(
      dialogRef.current.querySelectorAll<HTMLElement>(FOCUSABLE)
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];

    if (e.key === 'Tab') {
      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault();
          last.focus();
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    }
  }, []);

  // ── Open / close effects ──────────────────────────────────────────────────
  useEffect(() => {
    if (!open) return;

    // Lock body scroll
    const prev = document.body.style.overflow;
    document.body.style.overflow = 'hidden';

    // Focus first focusable element inside dialog
    const id = requestAnimationFrame(() => {
      const el = dialogRef.current?.querySelector<HTMLElement>(FOCUSABLE);
      el?.focus();
    });

    document.addEventListener('keydown', trapFocus);

    return () => {
      document.body.style.overflow = prev;
      cancelAnimationFrame(id);
      document.removeEventListener('keydown', trapFocus);
    };
  }, [open, trapFocus]);

  // ── Esc key ───────────────────────────────────────────────────────────────
  function handleKeyDown(e: KeyboardEvent<HTMLDivElement>) {
    if (e.key === 'Escape') {
      e.stopPropagation();
      onClose();
    }
  }

  // ── Backdrop click ────────────────────────────────────────────────────────
  function handleBackdropClick() {
    if (!disableBackdropClose) {
      onClose();
    }
  }

  if (!open) return null;

  const panel = (
    <div
      role="presentation"
      className={[
        'fixed inset-0 z-50 flex items-end justify-center sm:items-center p-0 sm:p-4',
        // Backdrop fade-in
        'animate-[fadeIn_150ms_ease-out]',
      ].join(' ')}
      onKeyDown={handleKeyDown}
    >
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        aria-hidden="true"
        onClick={handleBackdropClick}
      />

      {/* Dialog panel */}
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={title ? titleId : undefined}
        aria-describedby={children ? descId : undefined}
        tabIndex={-1}
        className={[
          // Layout
          'relative z-10 flex flex-col w-full',
          // Mobile: full-screen sheet from bottom
          'max-h-[100dvh] rounded-t-2xl sm:rounded-2xl',
          // Desktop: constrained width
          'sm:max-w-lg sm:max-h-[90vh]',
          // Surface
          'bg-[var(--color-surface)] border border-[var(--color-border)] shadow-2xl',
          // Slide-up on mobile, scale on desktop
          'animate-[slideUp_200ms_ease-out] sm:animate-[scaleIn_150ms_ease-out]',
          className,
        ].join(' ')}
      >
        {/* ── Header ────────────────────────────────────────────────────── */}
        {title !== undefined && (
          <div className="flex items-center justify-between gap-4 px-6 py-4 border-b border-[var(--color-border)] shrink-0">
            <h2 id={titleId} className="text-base font-semibold leading-tight">
              {title}
            </h2>
            <button
              type="button"
              onClick={onClose}
              className="flex min-h-11 min-w-11 shrink-0 items-center justify-center rounded-lg text-[var(--color-text-muted)] hover:bg-[var(--color-surface-raised)] hover:text-[var(--color-text)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
              aria-label="Close dialog"
            >
              <XIcon />
            </button>
          </div>
        )}

        {/* If there is no title, still render a standalone close button */}
        {title === undefined && (
          <div className="absolute right-3 top-3 z-10">
            <button
              type="button"
              onClick={onClose}
              className="flex min-h-11 min-w-11 items-center justify-center rounded-lg text-[var(--color-text-muted)] hover:bg-[var(--color-surface-raised)] hover:text-[var(--color-text)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
              aria-label="Close dialog"
            >
              <XIcon />
            </button>
          </div>
        )}

        {/* ── Body ──────────────────────────────────────────────────────── */}
        {children !== undefined && (
          <div id={descId} className="flex-1 overflow-y-auto px-6 py-5">
            {children}
          </div>
        )}

        {/* ── Footer ────────────────────────────────────────────────────── */}
        {footer !== undefined && (
          <div className="shrink-0 border-t border-[var(--color-border)] px-6 py-4">
            {footer}
          </div>
        )}
      </div>
    </div>
  );

  return createPortal(panel, document.body);
};

/* ── X Icon ───────────────────────────────────────────────────────────────── */

function XIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={2}
      stroke="currentColor"
      className="size-4"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M6 18 18 6M6 6l12 12"
      />
    </svg>
  );
}

/* ── Keyframe styles injected once ────────────────────────────────────────── */
// These are Tailwind v4 arbitrary keyframe animations defined via @keyframes in globals.css.
// For portability without modifying globals.css, we inject them here as a style tag
// only when the component is first mounted.

if (typeof document !== 'undefined') {
  const STYLE_ID = '__nevo_modal_keyframes__';
  if (!document.getElementById(STYLE_ID)) {
    const style = document.createElement('style');
    style.id = STYLE_ID;
    style.textContent = `
@keyframes fadeIn {
  from { opacity: 0; }
  to   { opacity: 1; }
}
@keyframes slideUp {
  from { opacity: 0; transform: translateY(100%); }
  to   { opacity: 1; transform: translateY(0); }
}
@keyframes scaleIn {
  from { opacity: 0; transform: scale(0.95) translateY(8px); }
  to   { opacity: 1; transform: scale(1)    translateY(0); }
}
    `;
    document.head.appendChild(style);
  }
}
