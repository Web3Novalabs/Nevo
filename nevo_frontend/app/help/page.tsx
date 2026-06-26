'use client';

import React, { useState, useId } from 'react';
import Link from 'next/link';

interface FaqItem {
  id: string;
  question: string;
  answer: string;
  category: string;
}

const FAQ_ITEMS: FaqItem[] = [
  {
    id: 'what-is-nevo',
    category: 'General',
    question: 'What is Nevo?',
    answer:
      'Nevo is an open-source on-chain donation platform built on the Stellar blockchain. It lets anyone create transparent fundraising pools where every contribution is recorded on-chain and withdrawals are handled by a smart contract — no intermediaries.',
  },
  {
    id: 'how-transparent',
    category: 'General',
    question: 'How is Nevo transparent?',
    answer:
      'Every donation and withdrawal is recorded directly on the Stellar blockchain. Anyone can verify transactions using a blockchain explorer like Stellar Expert. No funds can be moved without a smart contract interaction, ensuring full auditability.',
  },
  {
    id: 'supported-wallets',
    category: 'General',
    question: 'Which wallets are supported?',
    answer:
      'Nevo supports Freighter wallet and other Stellar-compatible wallets via the Stellar Wallets Kit. We recommend Freighter for the best experience. Make sure your wallet is set to the correct Stellar network.',
  },
  {
    id: 'create-pool',
    category: 'Creating Pools',
    question: 'How do I create a donation pool?',
    answer:
      'Connect your Stellar wallet, then navigate to Pools → Create Pool. Fill in your pool title, description, goal amount, and category. Once submitted, a smart contract transaction will be created and you will need to sign it in your wallet. After confirmation the pool goes live on-chain.',
  },
  {
    id: 'pool-fees',
    category: 'Creating Pools',
    question: 'Are there fees for creating a pool?',
    answer:
      'Nevo itself charges no platform fees. You only pay the standard Stellar network fee (typically under $0.01 USD) to submit the smart contract transaction. The pool funds belong entirely to the creator and donors.',
  },
  {
    id: 'edit-pool',
    category: 'Creating Pools',
    question: 'Can I edit my pool after creating it?',
    answer:
      'Off-chain metadata such as description and images can be updated by the pool creator from the pool detail page. The goal amount and creator wallet are recorded on-chain at creation and cannot be changed afterwards.',
  },
  {
    id: 'how-to-donate',
    category: 'Donating',
    question: 'How do I donate to a pool?',
    answer:
      'Browse or search for a pool on the Pools page, then open its detail page. Click "Donate Now", enter the amount you want to contribute, and confirm the transaction in your Stellar wallet. Your donation is recorded on-chain instantly.',
  },
  {
    id: 'minimum-donation',
    category: 'Donating',
    question: 'Is there a minimum donation amount?',
    answer:
      'The minimum donation is determined by the smart contract. In most cases any amount above the Stellar network dust threshold (0.0000001 XLM) is accepted. There is no platform-imposed minimum.',
  },
  {
    id: 'refund-donation',
    category: 'Donating',
    question: 'Can I get a refund on my donation?',
    answer:
      'Blockchain transactions are irreversible by design. Once your donation is confirmed on the Stellar network it cannot be reversed. Please verify the pool details carefully before donating.',
  },
  {
    id: 'withdraw-funds',
    category: 'Withdrawals',
    question: 'How does a pool creator withdraw funds?',
    answer:
      'Pool creators can withdraw raised funds from the pool detail page once the pool is active and has reached its goal or is ready to close. The smart contract will generate an unsigned transaction that the creator must sign in their wallet.',
  },
  {
    id: 'who-can-withdraw',
    category: 'Withdrawals',
    question: 'Who is allowed to withdraw from a pool?',
    answer:
      'Only the wallet address that created the pool (the pool creator) is authorised to withdraw funds. This is enforced at the smart contract level — no other address can move the funds.',
  },
  {
    id: 'view-transaction',
    category: 'Blockchain',
    question: 'How do I view my transaction on the blockchain?',
    answer:
      'Go to the Transaction History page (Transactions in the nav). Each transaction row shows a truncated hash and an "Explorer" link that opens the full transaction on Stellar Expert in a new tab, where you can see all on-chain details including fees and status.',
  },
  {
    id: 'network-fees',
    category: 'Blockchain',
    question: 'What network fees apply?',
    answer:
      'Stellar network fees are very low — typically around 100 stroops (0.00001 XLM) per operation, which is a fraction of a cent. Smart contract invocations may incur slightly higher fees depending on resource usage, but they remain well under $0.01 USD.',
  },
  {
    id: 'testnet-mainnet',
    category: 'Blockchain',
    question: 'Is Nevo on testnet or mainnet?',
    answer:
      'Nevo is currently running on the Stellar testnet. Testnet XLM has no real value and is used for testing purposes. We will announce the mainnet launch separately. Make sure your Freighter wallet is pointed at the correct network.',
  },
];

const CATEGORIES = Array.from(new Set(FAQ_ITEMS.map((item) => item.category)));

function ChevronDownIcon({ open }: { open: boolean }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={2}
      stroke="currentColor"
      className={`size-4 flex-shrink-0 transition-transform duration-200 ${open ? 'rotate-180' : ''}`}
      aria-hidden="true"
    >
      <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
    </svg>
  );
}

function SearchIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-4"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
      />
    </svg>
  );
}

interface AccordionItemProps {
  item: FaqItem;
  open: boolean;
  onToggle: () => void;
}

function AccordionItem({ item, open, onToggle }: AccordionItemProps) {
  const panelId = useId();
  const headingId = useId();

  return (
    <div className="border-b border-[var(--color-border)] last:border-b-0">
      <h3>
        <button
          type="button"
          id={headingId}
          aria-expanded={open}
          aria-controls={panelId}
          onClick={onToggle}
          className="flex w-full items-center justify-between gap-4 py-4 text-left text-sm font-medium text-[var(--color-text)] hover:text-brand-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600 transition-colors"
        >
          <span>{item.question}</span>
          <ChevronDownIcon open={open} />
        </button>
      </h3>
      <div
        id={panelId}
        role="region"
        aria-labelledby={headingId}
        hidden={!open}
      >
        <p className="pb-4 text-sm leading-relaxed text-[var(--color-text-muted)]">
          {item.answer}
        </p>
      </div>
    </div>
  );
}

export default function HelpPage() {
  const searchId = useId();
  const [search, setSearch] = useState('');
  const [activeCategory, setActiveCategory] = useState<string>('All');
  const [openItems, setOpenItems] = useState<Set<string>>(new Set());

  const normalizedSearch = search.trim().toLowerCase();

  const filteredItems = FAQ_ITEMS.filter((item) => {
    const matchesCategory =
      activeCategory === 'All' || item.category === activeCategory;
    const matchesSearch =
      !normalizedSearch ||
      item.question.toLowerCase().includes(normalizedSearch) ||
      item.answer.toLowerCase().includes(normalizedSearch);
    return matchesCategory && matchesSearch;
  });

  const groupedItems = CATEGORIES.reduce<Record<string, FaqItem[]>>(
    (acc, category) => {
      const items = filteredItems.filter((item) => item.category === category);
      if (items.length > 0) acc[category] = items;
      return acc;
    },
    {}
  );

  function toggleItem(id: string) {
    setOpenItems((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }

  const hasResults = filteredItems.length > 0;

  return (
    <main className="mx-auto max-w-3xl px-4 py-10 sm:px-6 sm:py-16">
      <div className="mb-10 text-center">
        <h1 className="text-3xl font-bold tracking-tight">Help &amp; FAQ</h1>
        <p className="mt-3 text-[var(--color-text-muted)]">
          Answers to common questions about creating pools, donating, and using
          the platform.
        </p>
      </div>

      {/* Search */}
      <div className="relative mb-6">
        <span className="pointer-events-none absolute inset-y-0 left-3 flex items-center text-[var(--color-text-muted)]">
          <SearchIcon />
        </span>
        <label htmlFor={searchId} className="sr-only">
          Search FAQ
        </label>
        <input
          id={searchId}
          type="search"
          placeholder="Search questions…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] py-3 pl-10 pr-4 text-sm placeholder:text-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-brand-500"
        />
      </div>

      {/* Category tabs */}
      <div
        role="tablist"
        aria-label="FAQ categories"
        className="mb-8 flex flex-wrap gap-2"
      >
        {['All', ...CATEGORIES].map((category) => (
          <button
            key={category}
            role="tab"
            type="button"
            aria-selected={activeCategory === category}
            onClick={() => setActiveCategory(category)}
            className={`rounded-full px-4 py-1.5 text-sm font-medium transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600 ${
              activeCategory === category
                ? 'bg-brand-600 text-white'
                : 'border border-[var(--color-border)] text-[var(--color-text-muted)] hover:border-brand-400 hover:text-brand-600'
            }`}
          >
            {category}
          </button>
        ))}
      </div>

      {/* FAQ accordion */}
      {hasResults ? (
        <div className="flex flex-col gap-8">
          {Object.entries(groupedItems).map(([category, items]) => (
            <section key={category} aria-labelledby={`cat-${category}`}>
              <h2
                id={`cat-${category}`}
                className="mb-3 text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)]"
              >
                {category}
              </h2>
              <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] px-5">
                {items.map((item) => (
                  <AccordionItem
                    key={item.id}
                    item={item}
                    open={openItems.has(item.id)}
                    onToggle={() => toggleItem(item.id)}
                  />
                ))}
              </div>
            </section>
          ))}
        </div>
      ) : (
        <div className="rounded-2xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface-raised)] px-6 py-12 text-center">
          <p className="font-semibold text-[var(--color-text)]">
            No results found
          </p>
          <p className="mt-1 text-sm text-[var(--color-text-muted)]">
            Try a different search term or browse all categories.
          </p>
          <button
            type="button"
            onClick={() => {
              setSearch('');
              setActiveCategory('All');
            }}
            className="mt-4 rounded-full bg-brand-600 px-5 py-2 text-sm font-semibold text-white hover:bg-brand-700 transition-colors"
          >
            Clear search
          </button>
        </div>
      )}

      {/* Contact support */}
      <div className="mt-12 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-6 text-center">
        <p className="font-semibold">Still have questions?</p>
        <p className="mt-1 text-sm text-[var(--color-text-muted)]">
          Can&apos;t find what you&apos;re looking for? Reach out and we&apos;ll
          help.
        </p>
        <div className="mt-4 flex flex-col gap-3 sm:flex-row sm:justify-center">
          <a
            href="mailto:hello@nevo.app"
            className="inline-flex items-center justify-center rounded-xl bg-brand-600 px-5 py-2.5 text-sm font-semibold text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
          >
            Email Support
          </a>
          <Link
            href="/about"
            className="inline-flex items-center justify-center rounded-xl border border-[var(--color-border)] px-5 py-2.5 text-sm font-semibold hover:bg-[var(--color-surface)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
          >
            About Nevo
          </Link>
        </div>
      </div>
    </main>
  );
}
