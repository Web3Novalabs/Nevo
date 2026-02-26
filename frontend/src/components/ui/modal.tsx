"use client";

import React, { useEffect, useRef, useCallback } from "react";
import { X } from "lucide-react";

interface ModalProps {
  /** Controls modal visibility */
  isOpen: boolean;
  /** Called when modal should close (backdrop click, X button, Escape key) */
  onClose: () => void;
  /** Modal heading shown in the header */
  title: string;
  /** Optional subheading shown below the title */
  subtitle?: string;
  /** Main content of the modal */
  children: React.ReactNode;
  /** Optional footer content (e.g. action buttons) */
  footer?: React.ReactNode;
  /** Max width of the modal. Defaults to 'md' */
  size?: "sm" | "md" | "lg" | "xl";
  /** Prevent closing when clicking the backdrop */
  disableBackdropClose?: boolean;
}

const sizeMap: Record<NonNullable<ModalProps["size"]>, string> = {
  sm: "max-w-sm",
  md: "max-w-md",
  lg: "max-w-lg",
  xl: "max-w-xl",
};

export const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  title,
  subtitle,
  children,
  footer,
  size = "md",
  disableBackdropClose = false,
}) => {
  const modalRef = useRef<HTMLDivElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  // Save focus and restore on close
  useEffect(() => {
    if (isOpen) {
      previousFocusRef.current = document.activeElement as HTMLElement;
      // Focus the modal panel after open
      setTimeout(() => modalRef.current?.focus(), 0);
    } else {
      previousFocusRef.current?.focus();
    }
  }, [isOpen]);

  // Trap focus within the modal
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLDivElement>) => {
      if (e.key === "Escape") {
        onClose();
        return;
      }

      if (e.key !== "Tab") return;

      const focusable = modalRef.current?.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), textarea, input, select, [tabindex]:not([tabindex="-1"])'
      );

      if (!focusable || focusable.length === 0) return;

      const first = focusable[0];
      const last = focusable[focusable.length - 1];

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
    },
    [onClose]
  );

  const handleBackdropClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!disableBackdropClose && e.target === e.currentTarget) {
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      style={{ backgroundColor: "rgba(10, 20, 40, 0.75)", backdropFilter: "blur(6px)" }}
      onClick={handleBackdropClick}
      aria-hidden={!isOpen}
    >
      <div
        ref={modalRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="modal-title"
        aria-describedby={subtitle ? "modal-subtitle" : undefined}
        tabIndex={-1}
        onKeyDown={handleKeyDown}
        className={`relative w-full ${sizeMap[size]} rounded-2xl shadow-2xl flex flex-col outline-none overflow-hidden`}
        style={{
          backgroundColor: "#0a1428",
          border: "1px solid rgba(31, 228, 255, 0.2)",
          boxShadow: "0 0 40px rgba(31, 228, 255, 0.08), 0 25px 50px rgba(0,0,0,0.6)",
        }}
      >
        {/* Top accent line */}
        <div
          className="h-0.5 w-full"
          style={{
            background: "linear-gradient(to right, #1fe4ff, #10b981, #1fe4ff)",
          }}
        />

        {/* Header */}
        <div
          className="flex items-start justify-between px-6 py-5"
          style={{ borderBottom: "1px solid rgba(31, 228, 255, 0.1)" }}
        >
          <div>
            <h2
              id="modal-title"
              className="text-xl font-bold tracking-tight"
              style={{ color: "#ffffff" }}
            >
              {title}
            </h2>
            {subtitle && (
              <p
                id="modal-subtitle"
                className="text-sm mt-1"
                style={{ color: "#a0aec0" }}
              >
                {subtitle}
              </p>
            )}
          </div>

          {/* Close button */}
          <button
            onClick={onClose}
            aria-label="Close modal"
            className="ml-4 flex-shrink-0 p-2 rounded-lg transition-all duration-200 hover:scale-110 active:scale-95"
            style={{
              color: "#a0aec0",
              backgroundColor: "rgba(31, 228, 255, 0.05)",
              border: "1px solid rgba(31, 228, 255, 0.1)",
            }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLButtonElement).style.color = "#1fe4ff";
              (e.currentTarget as HTMLButtonElement).style.borderColor =
                "rgba(31, 228, 255, 0.4)";
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLButtonElement).style.color = "#a0aec0";
              (e.currentTarget as HTMLButtonElement).style.borderColor =
                "rgba(31, 228, 255, 0.1)";
            }}
          >
            <X size={18} />
          </button>
        </div>

        {/* Body */}
        <div className="px-6 py-5 flex-1 overflow-y-auto">{children}</div>

        {/* Footer */}
        {footer && (
          <div
            className="px-6 py-4"
            style={{
              borderTop: "1px solid rgba(31, 228, 255, 0.1)",
              backgroundColor: "rgba(31, 228, 255, 0.02)",
            }}
          >
            {footer}
          </div>
        )}
      </div>
    </div>
  );
};

export default Modal;


// ─── USAGE EXAMPLE ───────────────────────────────────────────────────────────
//
// import { useState } from "react";
// import { Modal } from "@/components/ui/Modal";
//
// export function ExamplePage() {
//   const [open, setOpen] = useState(false);
//
//   return (
//     <>
//       <button onClick={() => setOpen(true)}>Open Modal</button>
//
//       <Modal
//         isOpen={open}
//         onClose={() => setOpen(false)}
//         title="Confirm Action"
//         subtitle="This action cannot be undone."
//         size="md"
//         footer={
//           <div className="flex gap-3 justify-end">
//             <button onClick={() => setOpen(false)}>Cancel</button>
//             <button onClick={() => setOpen(false)}>Confirm</button>
//           </div>
//         }
//       >
//         <p>Your modal content goes here.</p>
//       </Modal>
//     </>
//   );
// }