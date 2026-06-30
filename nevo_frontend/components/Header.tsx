'use client';

import Link from 'next/link';
import { useState } from 'react';
import { Button } from './Button';
import { MobileMenu, MobileMenuButton, NAV_LINKS } from './MobileMenu';

export const Header = () => {
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <>
      <header className="sticky top-0 z-50 w-full border-b border-gray-200 bg-white shadow-sm dark:bg-gray-900 dark:border-gray-800">
        <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
          <Link
            href="/"
            className="flex items-center gap-2 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500 rounded"
          >
            <span className="text-2xl font-bold text-blue-600 dark:text-blue-400">
              Nevo
            </span>
          </Link>

          {/* Desktop Navigation */}
          <nav aria-label="Global" className="hidden md:flex md:gap-x-8">
            {NAV_LINKS.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className="text-sm font-semibold leading-6 text-gray-900 hover:text-blue-600 dark:text-gray-100 dark:hover:text-blue-400 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500 rounded"
              >
                {link.label}
              </Link>
            ))}
          </nav>

          <div className="hidden md:flex md:items-center">
            <Button variant="primary" size="small">
              Connect Wallet
            </Button>
          </div>

          <MobileMenuButton open={menuOpen} onOpen={() => setMenuOpen(true)} />
        </div>
      </header>

      <MobileMenu open={menuOpen} onClose={() => setMenuOpen(false)} />
    </>
  );
};
