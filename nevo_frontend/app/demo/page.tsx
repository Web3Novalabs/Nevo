import { ProgressBar } from '@/components/ProgressBar';
import { Timeline, TimelineStep } from '@/components/Timeline';

const poolSteps: TimelineStep[] = [
  {
    id: 1,
    label: 'Pool Created',
    description: 'May 20, 2026',
    status: 'completed',
  },
  {
    id: 2,
    label: 'Funding Open',
    description: 'Accepting donations',
    status: 'active',
  },
  { id: 3, label: 'Goal Reached', status: 'pending' },
  { id: 4, label: 'Funds Released', status: 'pending' },
];

const txSteps: TimelineStep[] = [
  { id: 1, label: 'Submitted', status: 'completed' },
  { id: 2, label: 'Confirmed', status: 'completed' },
  {
    id: 3,
    label: 'Failed',
    description: 'Insufficient balance',
    status: 'error',
  },
];

export default function DemoPage() {
  return (
    <main className="mx-auto max-w-2xl px-6 py-16 flex flex-col gap-12">
      <h1 className="text-2xl font-bold">Component Demo</h1>

      {/* ── ProgressBar ─────────────────────────────────────────────────── */}
      <section aria-labelledby="pb-heading" className="flex flex-col gap-6">
        <h2 id="pb-heading" className="text-lg font-semibold">
          ProgressBar
        </h2>

        <ProgressBar
          label="Pool Funding"
          value={650}
          max={1000}
          valueLabel="650 XLM raised"
          maxLabel="1,000 XLM goal"
          showPercent
        />

        <ProgressBar
          label="Donation Drive"
          value={320}
          max={500}
          valueLabel="$320"
          maxLabel="$500"
          variant="success"
          size="lg"
          showPercent
        />

        <ProgressBar
          label="Urgent Appeal"
          value={80}
          max={500}
          valueLabel="$80"
          maxLabel="$500"
          variant="warning"
          size="sm"
          showPercent
        />
      </section>

      {/* ── Timeline vertical ───────────────────────────────────────────── */}
      <section aria-labelledby="tl-v-heading" className="flex flex-col gap-4">
        <h2 id="tl-v-heading" className="text-lg font-semibold">
          Timeline — Vertical (Pool lifecycle)
        </h2>
        <Timeline steps={poolSteps} />
      </section>

      {/* ── Timeline horizontal ─────────────────────────────────────────── */}
      <section aria-labelledby="tl-h-heading" className="flex flex-col gap-4">
        <h2 id="tl-h-heading" className="text-lg font-semibold">
          Timeline — Horizontal (Transaction status)
        </h2>
        <Timeline steps={txSteps} orientation="horizontal" />
      </section>
    </main>
  );
}
