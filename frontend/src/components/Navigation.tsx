"use client";

import { useEffect, useRef, useState } from "react";
import { Menu, Search, X } from "lucide-react";
import Link from "next/link";

import ConnectWallet from "./ConnectWallet";
import GlobalSearch from "./GlobalSearch";

interface NavLink {
  href: string;
  label: string;
  isRoute: boolean;
}

const navLinks: NavLink[] = [
  { href: "#features", label: "Features", isRoute: false },
  { href: "#how-it-works", label: "How It Works", isRoute: false },
  { href: "#security", label: "Security", isRoute: false },
  { href: "/discovery", label: "Discover", isRoute: true },
  { href: "/about-us", label: "About Us", isRoute: true },
];

export default function Navigation() {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const menuButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setIsSearchOpen(true);
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, []);

  const closeMenu = () => {
    setIsMenuOpen(false);
    menuButtonRef.current?.focus();
  };

  const openSearch = () => {
    setIsSearchOpen(true);
    setIsMenuOpen(false);
  };

  return (
    <>
      <nav className="fixed top-0 z-50 w-full border-b border-[#50C878]/40 bg-[#1E293B]">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-4 sm:px-6 lg:px-8">
          <Link href="/" className="flex shrink-0 items-center gap-2">
            <div className="h-8 w-8 rounded-lg bg-gradient-to-br from-blue-500 to-cyan-500" />
            <span className="text-xl font-bold text-slate-900 dark:text-white">
              Nevo
            </span>
          </Link>

          <div className="hidden items-center gap-5 lg:flex">
            {navLinks.map((link) => (
              <NavItem key={link.href} link={link} />
            ))}
            <SearchButton onClick={() => setIsSearchOpen(true)} />
            <button className="rounded-lg bg-blue-600 px-6 py-2 font-medium text-white transition-all duration-300 hover:-translate-y-1 hover:bg-blue-700 active:scale-95 active:shadow-[0_0_20px_rgba(37,99,235,0.6)]">
              Launch App
            </button>
            <ConnectWallet />
          </div>

          <button
            ref={menuButtonRef}
            onClick={() => setIsMenuOpen(true)}
            className="rounded-lg p-2 text-white transition hover:bg-slate-800 focus:outline-none focus:ring-2 focus:ring-emerald-400 lg:hidden"
            aria-label="Open navigation menu"
            aria-expanded={isMenuOpen}
            aria-controls="mobile-navigation-drawer"
          >
            <Menu className="h-6 w-6" />
          </button>
        </div>
      </nav>

      <MobileMenuDrawer
        isOpen={isMenuOpen}
        links={navLinks}
        onClose={closeMenu}
        onOpenSearch={openSearch}
      />

      <GlobalSearch
        isOpen={isSearchOpen}
        onClose={() => setIsSearchOpen(false)}
      />
    </>
  );
}

function NavItem({
  link,
  onClick,
  className,
}: {
  link: NavLink;
  onClick?: () => void;
  className?: string;
}) {
  const sharedClassName =
    className ??
    "whitespace-nowrap text-sm text-slate-600 transition hover:text-slate-900 dark:text-slate-400 dark:hover:text-white";

  if (link.isRoute) {
    return (
      <Link href={link.href} onClick={onClick} className={sharedClassName}>
        {link.label}
      </Link>
    );
  }

  return (
    <a href={link.href} onClick={onClick} className={sharedClassName}>
      {link.label}
    </a>
  );
}

function SearchButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="flex items-center gap-2 rounded-lg border border-slate-300 px-3 py-2 text-sm text-slate-600 transition hover:border-slate-400 hover:text-slate-900 dark:border-slate-600 dark:text-slate-400 dark:hover:border-slate-500 dark:hover:text-white"
    >
      <Search className="h-4 w-4" />
      <span>Search</span>
      <kbd className="rounded bg-slate-100 px-1.5 py-0.5 text-xs dark:bg-slate-800">
        Ctrl K
      </kbd>
    </button>
  );
}

function MobileMenuDrawer({
  isOpen,
  links,
  onClose,
  onOpenSearch,
}: {
  isOpen: boolean;
  links: NavLink[];
  onClose: () => void;
  onOpenSearch: () => void;
}) {
  const drawerRef = useRef<HTMLDivElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!isOpen) return;

    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    closeButtonRef.current?.focus();

    return () => {
      document.body.style.overflow = previousOverflow;
    };
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
        return;
      }

      if (event.key !== "Tab" || !drawerRef.current) return;

      const focusableElements = drawerRef.current.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), [tabindex]:not([tabindex="-1"])'
      );
      const firstElement = focusableElements[0];
      const lastElement = focusableElements[focusableElements.length - 1];

      if (!firstElement || !lastElement) return;

      if (event.shiftKey && document.activeElement === firstElement) {
        event.preventDefault();
        lastElement.focus();
      } else if (!event.shiftKey && document.activeElement === lastElement) {
        event.preventDefault();
        firstElement.focus();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  return (
    <div
      className={`fixed inset-0 z-[60] lg:hidden ${
        isOpen ? "pointer-events-auto" : "pointer-events-none"
      }`}
      aria-hidden={!isOpen}
      inert={!isOpen}
    >
      <button
        type="button"
        onClick={onClose}
        className={`absolute inset-0 bg-slate-950/70 transition-opacity duration-300 ${
          isOpen ? "opacity-100" : "opacity-0"
        }`}
        aria-label="Close navigation menu"
        tabIndex={isOpen ? 0 : -1}
      />

      <aside
        id="mobile-navigation-drawer"
        ref={drawerRef}
        role="dialog"
        aria-modal="true"
        aria-label="Mobile navigation"
        className={`absolute right-0 top-0 flex h-full w-[min(88vw,24rem)] flex-col border-l border-slate-700 bg-slate-950 text-white shadow-2xl transition-transform duration-300 ease-out ${
          isOpen ? "translate-x-0" : "translate-x-full"
        }`}
      >
        <div className="flex items-center justify-between border-b border-slate-800 px-5 py-4">
          <Link href="/" onClick={onClose} className="flex items-center gap-2">
            <div className="h-8 w-8 rounded-lg bg-gradient-to-br from-blue-500 to-cyan-500" />
            <span className="text-lg font-bold">Nevo</span>
          </Link>
          <button
            ref={closeButtonRef}
            onClick={onClose}
            className="rounded-lg p-2 text-slate-300 transition hover:bg-slate-800 hover:text-white focus:outline-none focus:ring-2 focus:ring-emerald-400"
            aria-label="Close navigation menu"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        <div className="flex flex-1 flex-col gap-2 overflow-y-auto px-5 py-5">
          {links.map((link) => (
            <NavItem
              key={link.href}
              link={link}
              onClick={onClose}
              className="rounded-lg px-3 py-3 text-base font-medium text-slate-200 transition hover:bg-slate-800 hover:text-white focus:outline-none focus:ring-2 focus:ring-emerald-400"
            />
          ))}

          <button
            onClick={onOpenSearch}
            className="mt-2 flex items-center justify-between rounded-lg border border-slate-800 px-3 py-3 text-left text-base font-medium text-slate-200 transition hover:border-emerald-400/50 hover:bg-slate-900 hover:text-white focus:outline-none focus:ring-2 focus:ring-emerald-400"
          >
            <span className="flex items-center gap-2">
              <Search className="h-4 w-4" />
              Search
            </span>
            <kbd className="rounded bg-slate-800 px-1.5 py-0.5 text-xs text-slate-400">
              Ctrl K
            </kbd>
          </button>

          <Link
            href="/dashboard"
            onClick={onClose}
            className="mt-4 rounded-lg bg-blue-600 px-4 py-3 text-center font-semibold text-white transition hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-emerald-400"
          >
            Launch App
          </Link>
        </div>
      </aside>
    </div>
  );
}
