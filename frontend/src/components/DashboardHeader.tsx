"use client";

import Link from "next/link";
import { LayoutDashboard } from "lucide-react";
import ConnectWallet from "./ConnectWallet";

/**
 * Top bar for the dashboard page.
 *
 * Contains the Nevo brand mark on the left and the wallet status widget
 * (ConnectWallet) on the right. ConnectWallet handles all connect /
 * disconnect / balance display logic internally.
 */
export default function DashboardHeader() {
    return (
        <header className="sticky top-0 z-50 w-full border-b border-slate-700/60 bg-[#0F172A]/80 backdrop-blur-md">
            <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
                {/* Brand + page context */}
                <div className="flex items-center gap-3">
                    <Link href="/" className="flex items-center gap-2">
                        <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-blue-500 to-cyan-500">
                            <span className="text-xs font-bold text-white">N</span>
                        </div>
                        <span className="text-lg font-bold text-white">Nevo</span>
                    </Link>

                    {/* Breadcrumb separator */}
                    <span className="text-slate-600">/</span>

                    <div className="flex items-center gap-1.5 text-sm text-slate-400">
                        <LayoutDashboard size={14} />
                        <span>Dashboard</span>
                    </div>
                </div>

                {/* Wallet widget */}
                <ConnectWallet />
            </div>
        </header>
    );
}
