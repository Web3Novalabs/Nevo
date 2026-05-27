type StepStatus = "complete" | "current" | "upcoming";

type Milestone = {
  label: string;
  value: string;
  status: StepStatus;
};

type TimelineStep = {
  title: string;
  description: string;
  status: StepStatus;
};

type ProgressTimelineProps = {
  title: string;
  subtitle?: string;
  currentAmount: number;
  targetAmount: number;
  currencySymbol?: string;
  milestones: Milestone[];
  steps: TimelineStep[];
};

function statusStyles(status: StepStatus) {
  switch (status) {
    case "complete":
      return {
        dot: "bg-brand-500 border-brand-500 text-white",
        label: "text-zinc-900 dark:text-zinc-50",
        description: "text-zinc-600 dark:text-zinc-400",
      };
    case "current":
      return {
        dot: "border-2 border-brand-500 bg-white text-brand-600",
        label: "text-zinc-900 dark:text-zinc-50",
        description: "text-zinc-600 dark:text-zinc-400",
      };
    default:
      return {
        dot: "border border-zinc-300 bg-zinc-100 text-zinc-500 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-400",
        label: "text-zinc-600 dark:text-zinc-400",
        description: "text-zinc-500 dark:text-zinc-500",
      };
  }
}

export default function ProgressTimeline({
  title,
  subtitle,
  currentAmount,
  targetAmount,
  currencySymbol = "$",
  milestones,
  steps,
}: ProgressTimelineProps) {
  const progress = Math.min(100, Math.max(0, Math.round((currentAmount / targetAmount) * 100)));

  return (
    <section className="rounded-3xl border border-zinc-200 bg-white p-6 shadow-sm shadow-zinc-200/50 dark:border-zinc-800 dark:bg-zinc-950 dark:shadow-black/10 sm:p-8">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h2 className="text-xl font-semibold tracking-tight text-zinc-900 dark:text-zinc-50">
            {title}
          </h2>
          {subtitle ? (
            <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-400">{subtitle}</p>
          ) : null}
        </div>
        <div className="inline-flex items-center gap-2 rounded-full bg-zinc-100 px-4 py-2 text-sm text-zinc-700 dark:bg-zinc-900 dark:text-zinc-200">
          <span className="font-semibold">{progress}%</span>
          <span className="text-zinc-500 dark:text-zinc-400">funded</span>
        </div>
      </div>

      <div className="mt-6">
        <div className="mb-3 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="text-sm text-zinc-500 dark:text-zinc-400">Raised</p>
            <p className="text-lg font-semibold text-zinc-900 dark:text-zinc-50">
              {currencySymbol}{currentAmount.toLocaleString()} / {currencySymbol}{targetAmount.toLocaleString()}
            </p>
          </div>
          <p className="text-sm text-zinc-500 dark:text-zinc-400">{progress}% to target</p>
        </div>

        <div
          role="progressbar"
          aria-valuenow={progress}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label="Pool funding progress"
          className="h-3 overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-800"
        >
          <div
            className="h-full rounded-full bg-brand-500 transition-all duration-700 ease-out"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>

      <div className="mt-8 grid gap-6 lg:grid-cols-[1fr_auto] lg:items-start">
        <div>
          <h3 className="text-sm font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
            Milestones
          </h3>
          <div className="mt-4 flex flex-col gap-4 sm:flex-row sm:items-center sm:gap-3">
            {milestones.map((milestone, index) => {
              const styles = statusStyles(milestone.status);
              return (
                <div key={milestone.label} className="flex-1 min-w-0 rounded-3xl border border-zinc-200 bg-zinc-50 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900">
                  <div className="flex items-center gap-3">
                    <div className={`flex h-9 w-9 items-center justify-center rounded-full transition ${styles.dot}`}>
                      {milestone.status === "complete" ? "✓" : index + 1}
                    </div>
                    <div className="min-w-0">
                      <p className={`text-sm font-semibold ${styles.label}`}>{milestone.label}</p>
                      <p className={`text-xs ${styles.description}`}>{milestone.value}</p>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        <div>
          <h3 className="text-sm font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
            Timeline
          </h3>
          <ol className="mt-4 space-y-4">
            {steps.map((step) => {
              const styles = statusStyles(step.status);
              return (
                <li key={step.title} className="flex items-start gap-4 rounded-3xl border border-zinc-200 bg-zinc-50 p-4 dark:border-zinc-800 dark:bg-zinc-900">
                  <div className={`mt-1 flex h-8 w-8 items-center justify-center rounded-full text-sm font-semibold transition ${styles.dot}`}>
                    {step.status === "complete" ? "✓" : step.status === "current" ? "●" : "○"}
                  </div>
                  <div>
                    <p className={`text-sm font-semibold ${styles.label}`}>{step.title}</p>
                    <p className={`mt-1 text-sm ${styles.description}`}>{step.description}</p>
                  </div>
                </li>
              );
            })}
          </ol>
        </div>
      </div>
    </section>
  );
}
