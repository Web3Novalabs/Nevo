"use client";

import { useState } from "react";
import { Menu, X } from "lucide-react";
import Link from "next/link";
import { OptimizedImage } from "@/components/OptimizedImage";
import ConnectWallet from "./ConnectWallet";
import { LOGO_SRC, LOGO_WIDTH, LOGO_HEIGHT } from "@/lib/images";

export default function Navigation() {
  const [isOpen, setIsOpen] = useState(false);

  const toggleMenu = () => {
    setIsOpen(!isOpen);
  };

  const navLinks = [
    { href: "#features", label: "Features", isRoute: false },
    { href: "#how-it-works", label: "How It Works", isRoute: false },
    { href: "#security", label: "Security", isRoute: false },
    { href: "/about-us", label: "About Us", isRoute: true },
  ];

  const handleLinkClick = () => {
    setIsOpen(false);
  };

  return (
    <nav className="fixed top-0 w-full bg-[#1E293B]  border-b border-[#50C878]/40 z-50 ">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex justify-between items-center">
        {/* Logo */}
        <Link href="/" className="flex items-center gap-2">
          <OptimizedImage
            src={LOGO_SRC}
            alt="Nevo"
            width={LOGO_WIDTH}
            height={LOGO_HEIGHT}
            className="rounded-lg object-cover"
            sizes="32px"
            priority
          />
          <span className="text-xl font-bold text-slate-900 dark:text-white">
            Nevo
          </span>
        </Link>

        {/* Desktop Menu */}
        <div className="hidden md:flex gap-8 items-center">
          {navLinks.map((link) =>
            link.isRoute ? (
              <Link
                key={link.href}
                href={link.href}
                className="text-sm text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white transition"
              >
                {link.label}
              </Link>
            ) : (
              <a
                key={link.href}
                href={link.href}
                className="text-sm text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white transition"
              >
                {link.label}
              </a>
            )
          )}
          <button className="bg-blue-600 hover:bg-blue-700 text-white px-6 py-2 rounded-lg transition font-medium">
            Launch App
          </button>
          <ConnectWallet />
        </div>

        {/* Mobile Menu Button */}
        <button
          onClick={toggleMenu}
          className="md:hidden p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-800 transition"
          aria-label="Toggle menu"
        >
          {isOpen ? (
            <X size={24} className="text-slate-900 dark:text-white" />
          ) : (
            <Menu size={24} className="text-slate-900 dark:text-white" />
          )}
        </button>
      </div>

      {/* Mobile Menu */}
      {isOpen && (
        <div className="md:hidden bg-white dark:bg-slate-900 border-t border-slate-200 dark:border-slate-800">
          <div className="px-4 py-4 space-y-3">
            {navLinks.map((link) =>
              link.isRoute ? (
                <Link
                  key={link.href}
                  href={link.href}
                  onClick={handleLinkClick}
                  className="block px-4 py-2 text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition"
                >
                  {link.label}
                </Link>
              ) : (
                <a
                  key={link.href}
                  href={link.href}
                  onClick={handleLinkClick}
                  className="block px-4 py-2 text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition"
                >
                  {link.label}
                </a>
              )
            )}
            <button className="w-full bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg transition font-medium">
              Launch App
            </button>
          </div>
        </div>
      )}
    </nav>
  );
}
