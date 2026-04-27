"use client";

import React, { useState } from 'react';
import Link from 'next/link';
import { Button } from './Button';

export const Header = () => {
    const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

    const navLinks = [
        { name: 'Home', href: '/' },
        { name: 'Browse Pools', href: '/pools' },
        { name: 'Create Pool', href: '/create-pool' },
        { name: 'Dashboard', href: '/dashboard' },
    ];

    return (
        <header className="sticky top-0 z-50 w-full border-b border-gray-200 bg-white shadow-sm dark:bg-gray-900 dark:border-gray-800">
            <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
                <div className="flex items-center">
                    <Link href="/" className="flex items-center gap-2">
                        <span className="text-2xl font-bold text-blue-600 dark:text-blue-400">Nevo</span>
                    </Link>
                </div>

                {/* Desktop Navigation */}
                <nav aria-label="Global" className="hidden md:flex md:gap-x-8">
                    {navLinks.map((link) => (
                        <Link
                            key={link.name}
                            href={link.href}
                            className="text-sm font-semibold leading-6 text-gray-900 hover:text-blue-600 dark:text-gray-100 dark:hover:text-blue-400 transition-colors"
                        >
                            {link.name}
                        </Link>
                    ))}
                </nav>

                <div className="hidden md:flex md:items-center">
                    <Button variant="primary" size="small">Connect Wallet</Button>
                </div>

                {/* Mobile menu button */}
                <div className="flex md:hidden">
                    <button
                        type="button"
                        className="-m-2.5 inline-flex items-center justify-center rounded-md p-2.5 text-gray-700 dark:text-gray-200"
                        onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
                        aria-expanded={isMobileMenuOpen}
                    >
                        <span className="sr-only">Open main menu</span>
                        <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" strokeWidth="1.5" stroke="currentColor" aria-hidden="true">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                        </svg>
                    </button>
                </div>
            </div>

            {/* Mobile menu */}
            {isMobileMenuOpen && (
                <div className="md:hidden" role="dialog" aria-modal="true">
                    <div className="space-y-1 px-4 pb-3 pt-2 sm:px-6">
                        {navLinks.map((link) => (
                            <Link
                                key={link.name}
                                href={link.href}
                                className="block rounded-md px-3 py-2 text-base font-medium text-gray-900 hover:bg-gray-50 dark:text-gray-100 dark:hover:bg-gray-800"
                                onClick={() => setIsMobileMenuOpen(false)}
                            >
                                {link.name}
                            </Link>
                        ))}
                        <div className="mt-4 px-3">
                            <Button variant="primary" size="medium" className="w-full">
                                Connect Wallet
                            </Button>
                        </div>
                    </div>
                </div>
            )}
        </header>
    );
};
