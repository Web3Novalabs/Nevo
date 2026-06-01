import Link from 'next/link';
import LegalPrint from '@/components/LegalPrint';

const LAST_UPDATED = 'May 31, 2026';

export default function TermsPage() {
  return (
    <main className="mx-auto max-w-4xl px-6 py-12">
      <div className="mb-8">
        <h1 className="text-3xl font-bold">Terms of Service</h1>
        <p className="mt-2 text-sm text-[var(--color-text-muted)]">
          Please read these terms carefully before using Nevo.
        </p>
      </div>

      <LegalPrint lastUpdated={LAST_UPDATED} />

      <section className="prose max-w-none">
        <h2>1. Acceptance of Terms</h2>
        <p>
          By accessing or using Nevo you agree to be bound by these Terms of
          Service.
        </p>

        <h2>2. Use of Service</h2>
        <p>
          You may use the service for lawful purposes only. You are responsible
          for your actions and any content you create.
        </p>

        <h2>3. Intellectual Property</h2>
        <p>
          The platform and its content are protected by intellectual property
          laws.
        </p>

        <h2>4. Disclaimers</h2>
        <p>
          The service is provided &quot;as is&quot; without warranties. Nevo is
          not responsible for third-party wallets or on-chain contract behavior.
        </p>

        <h2>5. Contact</h2>
        <p>
          Questions about these terms can be directed to our{' '}
          <Link href="/contact">support</Link>.
        </p>
      </section>
    </main>
  );
}
