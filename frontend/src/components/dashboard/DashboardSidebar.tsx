"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  Droplets,
  Wallet,
  Settings,
  Menu,
  X,
} from "lucide-react";
import { cn } from "@/lib/utils";

const navItems = [
  { href: "/dashboard", label: "Overview", icon: LayoutDashboard },
  { href: "/dashboard/my-pools", label: "My Pools", icon: Droplets },
  { href: "/dashboard/contributions", label: "Contributions", icon: Wallet },
  { href: "/dashboard/settings", label: "Settings", icon: Settings },
];

export function DashboardSidebar() {
  const [isMobileOpen, setIsMobileOpen] = useState(false);
  const pathname = usePathname();

  const toggleMobile = () => setIsMobileOpen((prev) => !prev);

  const navLinks = (
    <nav className="flex flex-col gap-1 p-4">
      {navItems.map(({ href, label, icon: Icon }) => {
        const isActive =
          href === "/dashboard"
            ? pathname === "/dashboard"
            : pathname.startsWith(href);
        return (
          <Link
            key={href}
            href={href}
            onClick={() => setIsMobileOpen(false)}
            className={cn(
              "flex items-center gap-3 rounded-lg px-4 py-3 text-sm font-medium transition-all duration-200",
              isActive
                ? "bg-emerald-500/20 text-emerald-400 shadow-[inset_0_0_0_1px_rgba(16,185,129,0.3)]"
                : "text-slate-400 hover:bg-slate-800/80 hover:text-slate-200"
            )}
          >
            <Icon
              className={cn(
                "h-5 w-5 shrink-0",
                isActive ? "text-emerald-400" : "text-slate-500"
              )}
            />
            {label}
          </Link>
        );
      })}
    </nav>
  );

  return (
    <>
      {/* Mobile header with hamburger */}
      <header className="sticky top-0 z-50 flex items-center justify-between border-b border-slate-800/80 bg-slate-950/95 px-4 py-3 backdrop-blur-sm lg:hidden">
        <Link
          href="/dashboard"
          className="flex items-center gap-2"
          onClick={() => setIsMobileOpen(false)}
        >
          <div className="h-8 w-8 rounded-lg bg-gradient-to-br from-emerald-500 to-cyan-500" />
          <span className="text-lg font-bold text-white">Nevo</span>
        </Link>
        <button
          onClick={toggleMobile}
          className="rounded-lg p-2 text-slate-400 transition hover:bg-slate-800 hover:text-white"
          aria-label={isMobileOpen ? "Close menu" : "Open menu"}
        >
          {isMobileOpen ? <X className="h-6 w-6" /> : <Menu className="h-6 w-6" />}
        </button>
      </header>

      {/* Mobile overlay */}
      {isMobileOpen && (
        <div
          className="fixed inset-0 z-40 bg-slate-950/80 backdrop-blur-sm lg:hidden"
          onClick={() => setIsMobileOpen(false)}
          aria-hidden
        />
      )}

      {/* Sidebar - drawer on mobile, fixed on desktop */}
      <aside
        className={cn(
          "fixed top-0 z-40 h-full w-72 shrink-0 border-r border-slate-800/80 bg-slate-950/98 shadow-xl backdrop-blur-sm transition-transform duration-300 ease-out lg:sticky lg:top-0 lg:translate-x-0 lg:shadow-none",
          isMobileOpen ? "translate-x-0" : "-translate-x-full lg:translate-x-0"
        )}
      >
        {/* Logo - desktop only */}
        <div className="hidden border-b border-slate-800/80 px-6 py-5 lg:block">
          <Link href="/dashboard" className="flex items-center gap-2">
            <div className="h-9 w-9 rounded-lg bg-gradient-to-br from-emerald-500 to-cyan-500 shadow-lg shadow-emerald-500/20" />
            <span className="text-xl font-bold tracking-tight text-white">
              Nevo
            </span>
          </Link>
        </div>

        {navLinks}
      </aside>
    </>
  );
}
