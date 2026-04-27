import Link from "next/link";

const features = [
  {
    title: "Open Donation Pools",
    description:
      "Create public fundraising pools for any cause with goals, descriptions, and full on-chain metadata.",
    icon: (
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="size-6" aria-hidden="true">
        <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v12m-3-2.818.879.659c1.171.879 3.07.879 4.242 0 1.172-.879 1.172-2.303 0-3.182C13.536 12.219 12.768 12 12 12c-.725 0-1.45-.22-2.003-.659-1.106-.879-1.106-2.303 0-3.182s2.9-.879 4.006 0l.415.33M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
      </svg>
    ),
  },
  {
    title: "Transparent Tracking",
    description:
      "Every contribution is recorded on-chain. Anyone can verify transactions, contributors, and pool status in real time.",
    icon: (
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="size-6" aria-hidden="true">
        <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 3v11.25A2.25 2.25 0 0 0 6 16.5h2.25M3.75 3h-1.5m1.5 0h16.5m0 0h1.5m-1.5 0v11.25A2.25 2.25 0 0 1 18 16.5h-2.25m-7.5 0h7.5m-7.5 0-1 3m8.5-3 1 3m0 0 .5 1.5m-.5-1.5h-9.5m0 0-.5 1.5m.75-9 3-3 2.148 2.148A12.061 12.061 0 0 1 16.5 7.605" />
      </svg>
    ),
  },
  {
    title: "Trustless Withdrawals",
    description:
      "Pool creators withdraw funds securely via smart contracts — no intermediaries, no centralized custody.",
    icon: (
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="size-6" aria-hidden="true">
        <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75 11.25 15 15 9.75m-3-7.036A11.959 11.959 0 0 1 3.598 6 11.99 11.99 0 0 0 3 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285Z" />
      </svg>
    ),
  },
  {
    title: "Low Fees on Stellar",
    description:
      "Built on Stellar's infrastructure for near-zero transaction fees and fast settlement times globally.",
    icon: (
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="size-6" aria-hidden="true">
        <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z" />
      </svg>
    ),
  },
];

const stats = [
  { label: "Pools Created", value: "1,200+" },
  { label: "Total Donated", value: "$480K+" },
  { label: "Contributors", value: "8,500+" },
  { label: "Avg. Fee", value: "< $0.01" },
];

const steps = [
  { step: "01", title: "Create a Pool", description: "Set a title, goal, and description for your campaign." },
  { step: "02", title: "Receive Donations", description: "Share your pool link — anyone can contribute on-chain." },
  { step: "03", title: "Withdraw Funds", description: "Close the pool and withdraw collected funds securely." },
];

export default function Home() {
  return (
    <main>
      {/* ── Hero ─────────────────────────────────────────────────────────── */}
      <section
        className="relative overflow-hidden bg-gradient-to-br from-brand-700 via-brand-600 to-brand-500 text-white"
        aria-labelledby="hero-heading"
      >
        {/* decorative blobs */}
        <div className="pointer-events-none absolute -top-32 -right-32 size-[480px] rounded-full bg-brand-400/30 blur-3xl" aria-hidden="true" />
        <div className="pointer-events-none absolute -bottom-24 -left-24 size-[360px] rounded-full bg-brand-800/40 blur-3xl" aria-hidden="true" />

        <div className="relative mx-auto max-w-5xl px-6 py-24 sm:py-36 text-center">
          <p className="mb-4 inline-block rounded-full border border-white/20 bg-white/10 px-4 py-1 text-sm font-medium tracking-wide">
            Built on Stellar · Open Source
          </p>
          <h1
            id="hero-heading"
            className="text-4xl font-bold tracking-tight sm:text-5xl lg:text-6xl"
          >
            Fundraising that&apos;s{" "}
            <span className="text-brand-200">transparent by design</span>
          </h1>
          <p className="mt-6 mx-auto max-w-2xl text-lg text-white/80 sm:text-xl">
            Nevo lets anyone create on-chain donation pools on Stellar. Every
            contribution is verifiable, every withdrawal is trustless — no
            middlemen, no hidden fees.
          </p>
          <div className="mt-10 flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link
              href="/pools/new"
              className="w-full sm:w-auto rounded-full bg-white px-8 py-3 text-sm font-semibold text-brand-700 shadow hover:bg-brand-50 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
              aria-label="Create a new donation pool"
            >
              Create a Pool
            </Link>
            <Link
              href="/pools"
              className="w-full sm:w-auto rounded-full border border-white/40 bg-white/10 px-8 py-3 text-sm font-semibold text-white hover:bg-white/20 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
              aria-label="Browse existing donation pools"
            >
              Browse Pools
            </Link>
          </div>
        </div>
      </section>

      {/* ── Stats ────────────────────────────────────────────────────────── */}
      <section aria-label="Platform statistics" className="border-b border-[var(--color-border)]">
        <div className="mx-auto max-w-5xl px-6 py-12 grid grid-cols-2 gap-8 sm:grid-cols-4">
          {stats.map(({ label, value }) => (
            <div key={label} className="text-center">
              <p className="text-3xl font-bold text-brand-600">{value}</p>
              <p className="mt-1 text-sm text-[var(--color-text-muted)]">{label}</p>
            </div>
          ))}
        </div>
      </section>

      {/* ── Features ─────────────────────────────────────────────────────── */}
      <section aria-labelledby="features-heading" className="mx-auto max-w-5xl px-6 py-20">
        <div className="text-center mb-12">
          <h2 id="features-heading" className="text-3xl font-bold tracking-tight sm:text-4xl">
            Everything you need to fundraise on-chain
          </h2>
          <p className="mt-4 text-[var(--color-text-muted)] max-w-xl mx-auto">
            Nevo combines the security of smart contracts with the simplicity of a modern web app.
          </p>
        </div>
        <div className="grid gap-8 sm:grid-cols-2 lg:grid-cols-4">
          {features.map(({ title, description, icon }) => (
            <article
              key={title}
              className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-6 flex flex-col gap-4"
            >
              <div className="flex size-11 items-center justify-center rounded-xl bg-brand-100 text-brand-600">
                {icon}
              </div>
              <h3 className="font-semibold">{title}</h3>
              <p className="text-sm text-[var(--color-text-muted)] leading-relaxed">{description}</p>
            </article>
          ))}
        </div>
      </section>

      {/* ── How it works ─────────────────────────────────────────────────── */}
      <section
        aria-labelledby="how-it-works-heading"
        className="bg-[var(--color-surface-raised)] border-y border-[var(--color-border)]"
      >
        <div className="mx-auto max-w-5xl px-6 py-20">
          <h2 id="how-it-works-heading" className="text-3xl font-bold tracking-tight text-center sm:text-4xl mb-12">
            How it works
          </h2>
          <ol className="grid gap-8 sm:grid-cols-3" role="list">
            {steps.map(({ step, title, description }) => (
              <li key={step} className="flex flex-col gap-3">
                <span className="text-4xl font-bold text-brand-500/30">{step}</span>
                <h3 className="font-semibold text-lg">{title}</h3>
                <p className="text-sm text-[var(--color-text-muted)]">{description}</p>
              </li>
            ))}
          </ol>
        </div>
      </section>

      {/* ── Bottom CTA ───────────────────────────────────────────────────── */}
      <section aria-labelledby="cta-heading" className="mx-auto max-w-5xl px-6 py-24 text-center">
        <h2 id="cta-heading" className="text-3xl font-bold tracking-tight sm:text-4xl">
          Ready to start fundraising?
        </h2>
        <p className="mt-4 text-[var(--color-text-muted)] max-w-lg mx-auto">
          Launch your campaign in minutes. No sign-up required — just connect your Stellar wallet.
        </p>
        <div className="mt-8 flex flex-col sm:flex-row items-center justify-center gap-4">
          <Link
            href="/pools/new"
            className="w-full sm:w-auto rounded-full bg-brand-600 px-8 py-3 text-sm font-semibold text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
            aria-label="Create a new donation pool"
          >
            Create a Pool
          </Link>
          <Link
            href="/pools"
            className="w-full sm:w-auto rounded-full border border-[var(--color-border)] px-8 py-3 text-sm font-semibold hover:bg-[var(--color-surface-raised)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
            aria-label="Browse existing donation pools"
          >
            Browse Pools
          </Link>
        </div>
      </section>
    </main>
  );
}
