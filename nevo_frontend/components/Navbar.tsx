import Link from 'next/link';
import ConnectWallet from '@/components/ConnectWallet';
import ThemeToggle from '@/components/ThemeToggle';

export default function Navbar() {
  return (
    <nav aria-label="Main navigation" className="w-full border-b border-zinc-200 dark:border-zinc-800 bg-white dark:bg-black">
      <div className="max-w-5xl mx-auto px-6 h-14 flex items-center justify-between">
        <Link
          href="/"
          className="text-lg font-semibold text-zinc-900 dark:text-zinc-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 rounded"
        >
          Nevo
        </Link>
        <div className="flex items-center gap-4 text-sm text-zinc-600 dark:text-zinc-400">
          <Link
            href="/pools"
            className="hover:text-zinc-900 dark:hover:text-zinc-50 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 rounded"
          >
            Pools
          </Link>
          <Link
            href="/transactions"
            className="hover:text-zinc-900 dark:hover:text-zinc-50 transition-colors"
          >
            Transactions
          </Link>
          <ThemeToggle />
          <ConnectWallet />
        </div>
      </div>
    </nav>
  );
}
