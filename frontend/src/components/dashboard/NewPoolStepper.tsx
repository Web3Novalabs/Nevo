\"use client\";

import { useState } from \"react\";
import { Button } from \"@/components/ui/button\";
import { Input } from \"@/components/ui/input\";
import { Label } from \"@/components/ui/label\";
import { cn } from \"@/lib/utils\";

type StepIndex = 0 | 1 | 2;

const STEPS: { id: StepIndex; title: string; description: string }[] = [
  {
    id: 0,
    title: \"Basic Info\",
    description: \"Name and describe your donation pool.\",
  },
  {
    id: 1,
    title: \"Financials\",
    description: \"Configure targets and contribution details.\",
  },
  {
    id: 2,
    title: \"Review\",
    description: \"Double-check everything before creating.\",
  },
];

interface FormState {
  name: string;
  description: string;
  category: string;
  targetAmount: string;
  currency: string;
  minContribution: string;
  durationDays: string;
}

type FormErrors = Partial<Record<keyof FormState, string>>;

const INITIAL_STATE: FormState = {
  name: \"\",
  description: \"\",
  category: \"General\",
  targetAmount: \"\",
  currency: \"XLM\",
  minContribution: \"\",
  durationDays: \"30\",
};

function validateStep(step: StepIndex, values: FormState): FormErrors {
  const errors: FormErrors = {};

  if (step === 0) {
    if (!values.name.trim()) {
      errors.name = \"Pool name is required.\";
    }
    if (!values.description.trim()) {
      errors.description = \"A short description is required.\";
    }
  }

  if (step === 1) {
    if (!values.targetAmount.trim()) {
      errors.targetAmount = \"Target amount is required.\";
    }
    if (!values.minContribution.trim()) {
      errors.minContribution = \"Minimum contribution is required.\";
    }
  }

  return errors;
}

export function NewPoolStepper() {
  const [currentStep, setCurrentStep] = useState<StepIndex>(0);
  const [values, setValues] = useState<FormState>(INITIAL_STATE);
  const [errors, setErrors] = useState<FormErrors>({});

  const totalSteps = STEPS.length;
  const progress = ((currentStep + 1) / totalSteps) * 100;

  const handleChange =
    (field: keyof FormState) =>
    (event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
      const value = event.target.value;
      setValues((prev) => ({ ...prev, [field]: value }));
      setErrors((prev) => ({ ...prev, [field]: undefined }));
    };

  const goToStep = (next: StepIndex) => {
    if (next === currentStep) return;
    if (next > currentStep) {
      const stepErrors = validateStep(currentStep, values);
      if (Object.keys(stepErrors).length > 0) {
        setErrors(stepErrors);
        return;
      }
    }
    setErrors({});
    setCurrentStep(next);
  };

  const handleNext = () => {
    const stepErrors = validateStep(currentStep, values);
    if (Object.keys(stepErrors).length > 0) {
      setErrors(stepErrors);
      return;
    }
    if (currentStep < (totalSteps - 1) as StepIndex) {
      setCurrentStep((prev) => ((prev + 1) as StepIndex));
      setErrors({});
    }
  };

  const handleBack = () => {
    if (currentStep === 0) return;
    setErrors({});
    setCurrentStep((prev) => ((prev - 1) as StepIndex));
  };

  return (
    <div className=\"space-y-8\">
      <header className=\"space-y-2\">
        <p className=\"inline-flex items-center rounded-full bg-emerald-500/10 px-3 py-1 text-xs font-medium text-emerald-300 ring-1 ring-emerald-500/30\">
          New Pool
        </p>
        <h1 className=\"text-3xl font-semibold tracking-tight text-white\">
          Create a donation pool
        </h1>
        <p className=\"max-w-2xl text-sm text-slate-400\">
          Set up a transparent pool for contributors. You can configure all
          details now and adjust the fine print later.
        </p>
      </header>

      <section className=\"rounded-2xl border border-slate-800/80 bg-slate-950/80 p-6 shadow-xl shadow-emerald-500/10 backdrop-blur-md md:p-8\">
        {/* Progress header */}
        <div className=\"space-y-4\">
          <div className=\"flex items-center justify-between gap-4\">
            <div className=\"flex items-center gap-3 text-xs font-medium uppercase tracking-[0.16em] text-slate-400\">
              <span className=\"inline-flex h-6 w-6 items-center justify-center rounded-full bg-emerald-500/15 text-emerald-300 ring-1 ring-emerald-500/40\">
                {currentStep + 1}
              </span>
              <span>{STEPS[currentStep].title}</span>
            </div>
            <span className=\"text-xs text-slate-500\">
              Step {currentStep + 1} of {totalSteps}
            </span>
          </div>

          <div className=\"relative mt-1 h-1.5 overflow-hidden rounded-full bg-slate-900\">
            <div
              className=\"h-full rounded-full bg-gradient-to-r from-emerald-400 via-emerald-300 to-cyan-400 transition-all duration-300 ease-out\"
              style={{ width: `${progress}%` }}
            />
          </div>

          <ol className=\"mt-4 flex flex-wrap items-center gap-3 text-xs\">
            {STEPS.map((step, index) => {
              const isCompleted = index < currentStep;
              const isActive = index === currentStep;

              return (
                <li key={step.id} className=\"flex items-center gap-2\">
                  <button
                    type=\"button\"
                    onClick={() => goToStep(step.id)}
                    className={cn(
                      \"flex items-center gap-2 rounded-full px-3 py-1 transition-colors\",
                      isActive &&
                        \"bg-emerald-500/15 text-emerald-200 ring-1 ring-emerald-500/40\",
                      isCompleted &&
                        !isActive &&
                        \"bg-slate-900 text-slate-200 ring-1 ring-slate-700 hover:bg-slate-800/80\",
                      !isActive &&
                        !isCompleted &&
                        \"text-slate-500 hover:text-slate-300/80\"
                    )}
                  >
                    <span
                      className={cn(
                        \"flex h-5 w-5 items-center justify-center rounded-full border text-[10px]\",
                        isActive &&
                          \"border-emerald-500/70 bg-emerald-500/15 text-emerald-200\",
                        isCompleted &&
                          !isActive &&
                          \"border-emerald-500/40 bg-emerald-500/20 text-emerald-50\",
                        !isActive &&
                          !isCompleted &&
                          \"border-slate-700 bg-slate-900 text-slate-400\"
                      )}
                    >
                      {index + 1}
                    </span>
                    <span className=\"hidden text-xs font-medium text-left sm:block\">
                      {step.title}
                    </span>
                  </button>
                  {index < totalSteps - 1 && (
                    <span className=\"hidden h-px w-6 bg-slate-800 sm:block\" />
                  )}
                </li>
              );
            })}
          </ol>
        </div>

        {/* Steps */}
        <div className=\"relative mt-8 min-h-[260px]\">
          {/* Basic Info */}
          <div
            className={cn(
              \"absolute inset-0 space-y-6 transition-all duration-300 ease-out\",
              currentStep === 0
                ? \"opacity-100 translate-x-0\"
                : currentStep > 0
                ? \"pointer-events-none -translate-x-4 opacity-0\"
                : \"pointer-events-none translate-x-4 opacity-0\"
            )}
          >
            <p className=\"text-sm text-slate-400\">
              Start by giving your pool a clear identity. This helps
              contributors understand the mission at a glance.
            </p>

            <div className=\"grid gap-5 md:grid-cols-2\">
              <div className=\"space-y-2 md:col-span-2\">
                <Label htmlFor=\"pool-name\" className=\"text-slate-200\">
                  Pool name <span className=\"text-emerald-400\">*</span>
                </Label>
                <Input
                  id=\"pool-name\"
                  value={values.name}
                  onChange={handleChange(\"name\")}
                  placeholder=\"E.g. Emergency Relief Fund for Lagos\"
                  aria-invalid={!!errors.name}
                />
                {errors.name && (
                  <p className=\"text-xs text-rose-400\">{errors.name}</p>
                )}
              </div>

              <div className=\"space-y-2 md:col-span-2\">
                <Label htmlFor=\"pool-description\" className=\"text-slate-200\">
                  Short description <span className=\"text-emerald-400\">*</span>
                </Label>
                <textarea
                  id=\"pool-description\"
                  rows={4}
                  value={values.description}
                  onChange={handleChange(\"description\")}
                  placeholder=\"Describe who this pool supports and how funds will be used.\"
                  aria-invalid={!!errors.description}
                  className={cn(
                    \"file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input w-full min-w-0 rounded-md border bg-transparent px-3 py-2 text-sm shadow-xs transition-[color,box-shadow] outline-none disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50\",
                    \"focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]\",
                    \"aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive\"
                  )}
                />
                {errors.description && (
                  <p className=\"text-xs text-rose-400\">
                    {errors.description}
                  </p>
                )}
              </div>

              <div className=\"space-y-2 md:col-span-1\">
                <Label htmlFor=\"pool-category\" className=\"text-slate-200\">
                  Category
                </Label>
                <select
                  id=\"pool-category\"
                  value={values.category}
                  onChange={handleChange(\"category\")}
                  className=\"h-9 w-full rounded-md border border-input bg-slate-900/60 px-3 text-sm text-slate-100 shadow-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]\"
                >
                  <option value=\"General\">General</option>
                  <option value=\"Emergency Relief\">Emergency Relief</option>
                  <option value=\"Education\">Education</option>
                  <option value=\"Healthcare\">Healthcare</option>
                  <option value=\"Climate & Environment\">Climate &amp; Environment</option>
                  <option value=\"Community\">Community</option>
                </select>
              </div>
            </div>
          </div>

          {/* Financials */}
          <div
            className={cn(
              \"absolute inset-0 space-y-6 transition-all duration-300 ease-out\",
              currentStep === 1
                ? \"opacity-100 translate-x-0\"
                : currentStep > 1
                ? \"pointer-events-none -translate-x-4 opacity-0\"
                : \"pointer-events-none translate-x-4 opacity-0\"
            )}
          >
            <p className=\"text-sm text-slate-400\">
              Define how the pool behaves financially. These values can guide
              expectations for contributors.
            </p>

            <div className=\"grid gap-5 md:grid-cols-2\">
              <div className=\"space-y-2\">
                <Label htmlFor=\"target-amount\" className=\"text-slate-200\">
                  Target amount <span className=\"text-emerald-400\">*</span>
                </Label>
                <div className=\"flex items-center gap-2\">
                  <Input
                    id=\"target-amount\"
                    type=\"number\"
                    min=\"0\"
                    value={values.targetAmount}
                    onChange={handleChange(\"targetAmount\")}
                    aria-invalid={!!errors.targetAmount}
                    placeholder=\"5000\"
                  />
                  <select
                    value={values.currency}
                    onChange={handleChange(\"currency\")}
                    className=\"h-9 rounded-md border border-input bg-slate-900/60 px-3 text-xs font-medium uppercase tracking-wide text-slate-100 shadow-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]\"
                  >
                    <option value=\"XLM\">XLM</option>
                    <option value=\"USDC\">USDC</option>
                    <option value=\"EURC\">EURC</option>
                  </select>
                </div>
                {errors.targetAmount && (
                  <p className=\"text-xs text-rose-400\">
                    {errors.targetAmount}
                  </p>
                )}
              </div>

              <div className=\"space-y-2\">
                <Label
                  htmlFor=\"min-contribution\"
                  className=\"text-slate-200\"
                >
                  Minimum contribution <span className=\"text-emerald-400\">*</span>
                </Label>
                <Input
                  id=\"min-contribution\"
                  type=\"number\"
                  min=\"0\"
                  value={values.minContribution}
                  onChange={handleChange(\"minContribution\")}
                  aria-invalid={!!errors.minContribution}
                  placeholder=\"10\"
                />
                {errors.minContribution && (
                  <p className=\"text-xs text-rose-400\">
                    {errors.minContribution}
                  </p>
                )}
              </div>

              <div className=\"space-y-2\">
                <Label htmlFor=\"duration-days\" className=\"text-slate-200\">
                  Duration in days
                </Label>
                <Input
                  id=\"duration-days\"
                  type=\"number\"
                  min=\"1\"
                  value={values.durationDays}
                  onChange={handleChange(\"durationDays\")}
                  placeholder=\"30\"
                />
                <p className=\"text-xs text-slate-500\">
                  After this date, new contributions can be paused or rerouted.
                </p>
              </div>
            </div>
          </div>

          {/* Review */}
          <div
            className={cn(
              \"absolute inset-0 space-y-6 transition-all duration-300 ease-out\",
              currentStep === 2
                ? \"opacity-100 translate-x-0\"
                : \"pointer-events-none translate-x-4 opacity-0\"
            )}
          >
            <p className=\"text-sm text-slate-400\">
              Confirm that everything looks right. You&apos;ll be able to adjust
              advanced settings after creation.
            </p>

            <div className=\"grid gap-5 md:grid-cols-2\">
              <div className=\"space-y-3 rounded-xl border border-slate-800 bg-slate-900/60 p-4\">
                <h3 className=\"text-sm font-semibold text-slate-100\">
                  Identity
                </h3>
                <dl className=\"space-y-2 text-sm\">
                  <div>
                    <dt className=\"text-slate-500\">Pool name</dt>
                    <dd className=\"text-slate-100\">
                      {values.name || <span className=\"text-slate-600\">Not set</span>}
                    </dd>
                  </div>
                  <div>
                    <dt className=\"text-slate-500\">Category</dt>
                    <dd className=\"text-slate-100\">{values.category}</dd>
                  </div>
                </dl>
              </div>

              <div className=\"space-y-3 rounded-xl border border-slate-800 bg-slate-900/60 p-4\">
                <h3 className=\"text-sm font-semibold text-slate-100\">
                  Financials
                </h3>
                <dl className=\"space-y-2 text-sm\">
                  <div>
                    <dt className=\"text-slate-500\">Target</dt>
                    <dd className=\"text-slate-100\">
                      {values.targetAmount
                        ? `${values.targetAmount} ${values.currency}`
                        : <span className=\"text-slate-600\">Not set</span>}
                    </dd>
                  </div>
                  <div>
                    <dt className=\"text-slate-500\">Minimum contribution</dt>
                    <dd className=\"text-slate-100\">
                      {values.minContribution
                        ? `${values.minContribution} ${values.currency}`
                        : <span className=\"text-slate-600\">Not set</span>}
                    </dd>
                  </div>
                  <div>
                    <dt className=\"text-slate-500\">Duration</dt>
                    <dd className=\"text-slate-100\">
                      {values.durationDays
                        ? `${values.durationDays} days`
                        : <span className=\"text-slate-600\">Not set</span>}
                    </dd>
                  </div>
                </dl>
              </div>

              <div className=\"md:col-span-2 space-y-2 rounded-xl border border-slate-800 bg-slate-900/60 p-4\">
                <h3 className=\"text-sm font-semibold text-slate-100\">
                  Description
                </h3>
                <p className=\"text-sm text-slate-300 whitespace-pre-line\">
                  {values.description || (
                    <span className=\"text-slate-600\">
                      No description provided yet.
                    </span>
                  )}
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Footer actions */}
        <div className=\"mt-10 flex flex-col gap-3 border-t border-slate-800 pt-4 sm:flex-row sm:items-center sm:justify-between\">
          <p className=\"text-xs text-slate-500\">
            You can safely close this page before submitting &mdash; your pool
            is only created once you confirm.
          </p>
          <div className=\"flex items-center justify-end gap-3\">
            <Button
              type=\"button\"
              variant=\"outline\"
              size=\"sm\"
              onClick={handleBack}
              disabled={currentStep === 0}
            >
              Back
            </Button>
            <Button
              type=\"button\"
              size=\"sm\"
              onClick={handleNext}
              aria-label={currentStep === totalSteps - 1 ? \"Validate details\" : \"Go to next step\"}
            >
              {currentStep === totalSteps - 1 ? \"Looks good\" : \"Next\"}
            </Button>
          </div>
        </div>
      </section>
    </div>
  );
}

