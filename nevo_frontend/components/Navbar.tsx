'use client';

import Link from 'next/link';
import { useState } from 'react';
import ConnectWallet from '@/components/ConnectWallet';
import {
  MobileMenu,
  MobileMenuButton,
  NAV_LINKS,
} from '@/components/MobileMenu';
import ThemeToggle from '@/components/ThemeToggle';
import LanguageSwitcher from '@/components/LanguageSwitcher';
import { useTranslation } from 'react-i18next';
import '@/src/lib/i18n';

export default function Navbar() {
  const [menuOpen, setMenuOpen] = useState(false);
  const { t } = useTranslation();

  function openMenu() {
    setMenuOpen(true);
  }

  function closeMenu() {
    setMenuOpen(false);
  }

  return (
    <>
      <nav
        aria-label="Main navigation"
        className="w-full border-b border-[var(--color-border)] bg-[var(--color-surface)]"
      >
        <div className="mx-auto flex h-14 max-w-5xl items-center justify-between px-4 sm:px-6">
          <Link
            href="/"
            className="text-lg font-semibold text-[var(--color-text)] transition-colors hover:text-brand-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600 rounded"
          >
            Nevo
          </Link>

          <div className="hidden items-center gap-6 text-sm lg:flex">
            {NAV_LINKS.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className="font-medium text-[var(--color-text-muted)] transition-colors hover:text-[var(--color-text)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600 rounded"
              >
                {t(link.key.replace(/^nav\./, 'nav.'))}
              </Link>
            ))}
            <ThemeToggle />
            <LanguageSwitcher />
            <ConnectWallet />
          </div>

          <MobileMenuButton open={menuOpen} onOpen={openMenu} />
        </div>
      </nav>

      <div id="mobile-menu-drawer">
        <MobileMenu open={menuOpen} onClose={closeMenu} />
      </div>
    </>
  );
}
