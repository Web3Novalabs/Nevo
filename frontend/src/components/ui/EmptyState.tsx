"use client";

import * as React from "react";
import { motion } from "framer-motion";
import { LucideIcon, ArrowRight, Info } from "lucide-react";
import Link from "next/link";
import { Button } from "./button";
import { cn } from "@/lib/utils";

export interface EmptyStateProps {
  /**
   * Title of the empty state (e.g., "No donation pools active")
   */
  title: string;
  /**
   * Explanatory text giving context on what to do next
   */
  description: string;
  /**
   * Optional Lucide icon to display at the top
   */
  icon?: LucideIcon;
  /**
   * Optional custom React node (e.g., a custom SVG illustration or animation)
   */
  illustration?: React.ReactNode;
  /**
   * Primary call-to-action button configuration
   */
  action?: {
    label: string;
    onClick?: (e: React.MouseEvent<HTMLButtonElement>) => void;
    href?: string;
    icon?: LucideIcon;
  };
  /**
   * Optional secondary action button configuration
   */
  secondaryAction?: {
    label: string;
    onClick?: (e: React.MouseEvent<HTMLButtonElement>) => void;
    href?: string;
    icon?: LucideIcon;
  };
  /**
   * Optional list of suggested next steps/tips
   */
  suggestions?: string[];
  /**
   * Custom CSS classes for the outer wrapper
   */
  className?: string;
}

export function EmptyState({
  title,
  description,
  icon: Icon,
  illustration,
  action,
  secondaryAction,
  suggestions,
  className,
}: EmptyStateProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 15 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, ease: "easeOut" }}
      className={cn(
        "relative overflow-hidden w-full max-w-xl mx-auto rounded-3xl bg-slate-900/60 border border-slate-800 backdrop-blur-md px-6 py-10 sm:p-12 text-center shadow-2xl",
        "before:absolute before:-top-24 before:-left-24 before:w-48 before:h-48 before:bg-[#50C878]/10 before:blur-3xl before:rounded-full before:pointer-events-none",
        "after:absolute after:-bottom-24 after:-right-24 after:w-48 after:h-48 after:bg-blue-500/10 after:blur-3xl after:rounded-full after:pointer-events-none",
        className
      )}
    >
      {/* Icon or Illustration Section */}
      <div className="relative mb-6 flex justify-center">
        {illustration ? (
          <div className="relative z-10 flex items-center justify-center">
            {illustration}
          </div>
        ) : Icon ? (
          <motion.div
            whileHover={{ scale: 1.05, y: -2 }}
            transition={{ type: "spring", stiffness: 300, damping: 15 }}
            className="relative z-10 flex size-16 items-center justify-center rounded-2xl bg-gradient-to-br from-slate-800 to-slate-950 border border-slate-700/60 text-[#50C878] shadow-[0_8px_30px_rgb(0,0,0,0.2)]"
          >
            <Icon className="size-8" strokeWidth={1.5} />
          </motion.div>
        ) : null}
      </div>

      {/* Title & Description */}
      <div className="relative z-10 space-y-2 mb-8">
        <h3 className="text-xl sm:text-2xl font-bold tracking-tight text-white">
          {title}
        </h3>
        <p className="text-sm sm:text-base text-slate-400 max-w-md mx-auto leading-relaxed">
          {description}
        </p>
      </div>

      {/* Suggested next steps */}
      {suggestions && suggestions.length > 0 && (
        <div className="relative z-10 text-left max-w-md mx-auto bg-slate-950/40 rounded-2xl p-5 mb-8 border border-slate-800/80">
          <h4 className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-[#50C878] mb-3">
            <Info className="size-3.5" />
            Suggested next steps
          </h4>
          <ul className="space-y-2.5">
            {suggestions.map((suggestion, index) => (
              <li key={index} className="flex items-start gap-2.5 text-sm text-slate-300 leading-tight">
                <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-slate-800/80 text-xs font-medium text-slate-400">
                  {index + 1}
                </span>
                <span className="pt-0.5">{suggestion}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Action CTA Buttons */}
      {(action || secondaryAction) && (
        <div className="relative z-10 flex flex-col sm:flex-row items-center justify-center gap-3">
          {secondaryAction && (
            secondaryAction.href ? (
              <Button
                asChild
                variant="outline"
                className="w-full sm:w-auto min-w-[140px] border-slate-800 hover:border-slate-700 hover:bg-slate-800 text-slate-300"
              >
                <Link href={secondaryAction.href} className="flex items-center justify-center gap-2">
                  {secondaryAction.icon && <secondaryAction.icon className="size-4" />}
                  <span>{secondaryAction.label}</span>
                </Link>
              </Button>
            ) : (
              <Button
                variant="outline"
                onClick={secondaryAction.onClick}
                className="w-full sm:w-auto min-w-[140px] border-slate-800 hover:border-slate-700 hover:bg-slate-800 text-slate-300"
              >
                <span className="flex items-center justify-center gap-2">
                  {secondaryAction.icon && <secondaryAction.icon className="size-4" />}
                  <span>{secondaryAction.label}</span>
                </span>
              </Button>
            )
          )}

          {action && (
            action.href ? (
              <Button
                asChild
                className="w-full sm:w-auto min-w-[140px] bg-[#50C878] hover:bg-[#45b76b] text-slate-950 font-semibold shadow-[0_0_15px_rgba(80,200,120,0.2)]"
              >
                <Link href={action.href} className="flex items-center justify-center gap-2">
                  <span>{action.label}</span>
                  {action.icon ? (
                    <action.icon className="size-4" />
                  ) : (
                    <ArrowRight className="size-4" />
                  )}
                </Link>
              </Button>
            ) : (
              <Button
                onClick={action.onClick}
                className="w-full sm:w-auto min-w-[140px] bg-[#50C878] hover:bg-[#45b76b] text-slate-950 font-semibold shadow-[0_0_15px_rgba(80,200,120,0.2)]"
              >
                <span className="flex items-center justify-center gap-2">
                  <span>{action.label}</span>
                  {action.icon ? (
                    <action.icon className="size-4" />
                  ) : (
                    <ArrowRight className="size-4" />
                  )}
                </span>
              </Button>
            )
          )}
        </div>
      )}
    </motion.div>
  );
}
