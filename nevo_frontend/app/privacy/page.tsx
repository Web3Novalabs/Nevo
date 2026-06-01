import Link from 'next/link';
import LegalPrint from '@/components/LegalPrint';

const LAST_UPDATED = 'May 31, 2026';

export default function PrivacyPage() {
  return (
    <main className="mx-auto max-w-4xl px-6 py-12">
      <div className="mb-8">
        <h1 className="text-3xl font-bold">Privacy Policy</h1>
        <p className="mt-2 text-sm text-[var(--color-text-muted)]">
          How we collect, use, and protect your information.
        </p>
      </div>

      <LegalPrint lastUpdated={LAST_UPDATED} />

      <section className="prose max-w-none">
        <h2>1. Information We Collect</h2>
        <p>
          We collect only minimal information required to operate the platform.
          Wallet addresses and transaction metadata are stored on-chain; we do
          not store private keys.
        </p>

        <h2>2. Use of Data</h2>
        <p>
          Data is used to operate and improve the service, to process donations,
          and to communicate with users.
        </p>

        <h2>3. Cookies and Tracking</h2>
        <p>
          We use cookies and analytics tools to understand platform usage. You
          can opt out where applicable.
        </p>

        <h2>4. Data Retention</h2>
        <p>
          On-chain data persists according to blockchain rules. Off-chain logs
          are retained for operational purposes.
        </p>

        <h2>5. Contact</h2>
        <p>
          For privacy requests, contact us via the{' '}
          <Link href="/contact">support page</Link>.
        </p>
      </section>
    </main>
  );
}
