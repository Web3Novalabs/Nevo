'use client';

import React, { useState } from 'react';
import { useRouter } from 'next/navigation';
import ProtectedRoute from '@/components/ProtectedRoute';

// TODO: Replace with real pool creation API call once backend is implemented
const CATEGORIES = [
  'Humanitarian',
  'Technology',
  'Environment',
  'Education',
  'Health',
  'Community',
  'Arts',
  'Other',
];

const DURATION_OPTIONS = [
  { label: '7 days', value: 7 },
  { label: '14 days', value: 14 },
  { label: '30 days', value: 30 },
  { label: '60 days', value: 60 },
  { label: '90 days', value: 90 },
];

interface FormData {
  title: string;
  description: string;
  category: string;
  goalAmount: string;
  duration: number;
  imageUrl: string;
  tags: string;
}

interface FormErrors {
  title?: string;
  description?: string;
  category?: string;
  goalAmount?: string;
  duration?: string;
}

const INITIAL_FORM: FormData = {
  title: '',
  description: '',
  category: '',
  goalAmount: '',
  duration: 30,
  imageUrl: '',
  tags: '',
};

type Step = 1 | 2 | 3;

function CreatePoolPageContent() {
  const router = useRouter();
  const [step, setStep] = useState<Step>(1);
  const [form, setForm] = useState<FormData>(INITIAL_FORM);
  const [errors, setErrors] = useState<FormErrors>({});
  const [submitting, setSubmitting] = useState(false);
  const [submitted, setSubmitted] = useState(false);

  function update(field: keyof FormData, value: string | number) {
    setForm((prev) => ({ ...prev, [field]: value }));
    if (errors[field as keyof FormErrors]) {
      setErrors((prev) => ({ ...prev, [field]: undefined }));
    }
  }

  function validateStep1(): boolean {
    const errs: FormErrors = {};
    if (!form.title.trim()) errs.title = 'Title is required.';
    else if (form.title.trim().length < 5)
      errs.title = 'Title must be at least 5 characters.';
    if (!form.description.trim()) errs.description = 'Description is required.';
    else if (form.description.trim().length < 20)
      errs.description = 'Description must be at least 20 characters.';
    if (!form.category) errs.category = 'Please select a category.';
    setErrors(errs);
    return Object.keys(errs).length === 0;
  }

  function validateStep2(): boolean {
    const errs: FormErrors = {};
    const goal = parseFloat(form.goalAmount);
    if (!form.goalAmount) errs.goalAmount = 'Goal amount is required.';
    else if (isNaN(goal) || goal <= 0)
      errs.goalAmount = 'Enter a valid amount greater than 0.';
    if (!form.duration) errs.duration = 'Please select a duration.';
    setErrors(errs);
    return Object.keys(errs).length === 0;
  }

  function handleNext() {
    if (step === 1 && validateStep1()) setStep(2);
    else if (step === 2 && validateStep2()) setStep(3);
  }

  function handleBack() {
    setErrors({});
    if (step === 2) setStep(1);
    else if (step === 3) setStep(2);
  }

  async function handleSubmit() {
    setSubmitting(true);
    // TODO: Replace with real pool creation call once backend is implemented
    await new Promise((r) => setTimeout(r, 1000));
    setSubmitting(false);
    setSubmitted(true);
  }

  if (submitted) {
    return <SuccessScreen onGoToDashboard={() => router.push('/dashboard')} />;
  }

  const tagList = form.tags
    .split(',')
    .map((t) => t.trim())
    .filter(Boolean);

  return (
    <main className="mx-auto max-w-2xl px-6 py-12">
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight">Create a Pool</h1>
        <p className="mt-1 text-sm text-[var(--color-text-muted)]">
          Launch a transparent fundraising campaign on Stellar.
        </p>
      </div>

      {/* Step indicator */}
      <StepIndicator current={step} />

      <div className="mt-8 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 sm:p-8">
        {step === 1 && (
          <Step1
            form={form}
            errors={errors}
            onChange={update}
            onNext={handleNext}
          />
        )}
        {step === 2 && (
          <Step2
            form={form}
            errors={errors}
            onChange={update}
            onNext={handleNext}
            onBack={handleBack}
          />
        )}
        {step === 3 && (
          <Step3
            form={form}
            tagList={tagList}
            submitting={submitting}
            onBack={handleBack}
            onSubmit={handleSubmit}
          />
        )}
      </div>
    </main>
  );
}

/* ── Step indicator ───────────────────────────────────────────────────────── */

function StepIndicator({ current }: { current: Step }) {
  const steps = [
    { n: 1, label: 'Basics' },
    { n: 2, label: 'Goal & Duration' },
    { n: 3, label: 'Preview' },
  ];
  return (
    <nav aria-label="Form steps">
      <ol className="flex items-center gap-0">
        {steps.map(({ n, label }, i) => {
          const done = current > n;
          const active = current === n;
          return (
            <React.Fragment key={n}>
              <li className="flex flex-col items-center gap-1">
                <span
                  className={`flex size-8 items-center justify-center rounded-full text-sm font-semibold transition-colors ${
                    done
                      ? 'bg-brand-600 text-white'
                      : active
                        ? 'border-2 border-brand-600 text-brand-600'
                        : 'border-2 border-[var(--color-border)] text-[var(--color-text-muted)]'
                  }`}
                  aria-current={active ? 'step' : undefined}
                >
                  {done ? <CheckIcon /> : n}
                </span>
                <span
                  className={`text-xs ${active ? 'font-medium text-brand-600' : 'text-[var(--color-text-muted)]'}`}
                >
                  {label}
                </span>
              </li>
              {i < steps.length - 1 && (
                <div
                  className={`mb-4 h-px flex-1 transition-colors ${current > n ? 'bg-brand-600' : 'bg-[var(--color-border)]'}`}
                  aria-hidden="true"
                />
              )}
            </React.Fragment>
          );
        })}
      </ol>
    </nav>
  );
}

/* ── Step 1: Basics ───────────────────────────────────────────────────────── */

interface Step1Props {
  form: FormData;
  errors: FormErrors;
  onChange: (field: keyof FormData, value: string) => void;
  onNext: () => void;
}

function Step1({ form, errors, onChange, onNext }: Step1Props) {
  return (
    <div>
      <h2 className="mb-6 text-lg font-semibold">Basic Information</h2>
      <div className="flex flex-col gap-5">
        <Field
          label="Title"
          required
          error={errors.title}
          hint="Give your pool a clear, descriptive name."
        >
          <input
            id="title"
            type="text"
            value={form.title}
            onChange={(e) => onChange('title', e.target.value)}
            placeholder="e.g. Clean Water Initiative"
            maxLength={100}
            aria-describedby={errors.title ? 'title-error' : undefined}
            className={inputClass(!!errors.title)}
          />
        </Field>

        <Field
          label="Description"
          required
          error={errors.description}
          hint="Explain what this pool is for and how funds will be used."
        >
          <textarea
            id="description"
            value={form.description}
            onChange={(e) => onChange('description', e.target.value)}
            placeholder="Describe your campaign in detail..."
            rows={4}
            maxLength={1000}
            aria-describedby={
              errors.description ? 'description-error' : undefined
            }
            className={inputClass(!!errors.description)}
          />
          <p className="mt-1 text-right text-xs text-[var(--color-text-muted)]">
            {form.description.length}/1000
          </p>
        </Field>

        <Field label="Category" required error={errors.category}>
          <select
            id="category"
            value={form.category}
            onChange={(e) => onChange('category', e.target.value)}
            aria-describedby={errors.category ? 'category-error' : undefined}
            className={inputClass(!!errors.category)}
          >
            <option value="">Select a category</option>
            {CATEGORIES.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </select>
        </Field>
      </div>

      <div className="mt-8 flex justify-end">
        <button onClick={onNext} className={primaryBtn}>
          Next: Goal &amp; Duration
        </button>
      </div>
    </div>
  );
}

/* ── Step 2: Goal & Duration ──────────────────────────────────────────────── */

interface Step2Props {
  form: FormData;
  errors: FormErrors;
  onChange: (field: keyof FormData, value: string | number) => void;
  onNext: () => void;
  onBack: () => void;
}

function Step2({ form, errors, onChange, onNext, onBack }: Step2Props) {
  return (
    <div>
      <h2 className="mb-6 text-lg font-semibold">Goal &amp; Duration</h2>
      <div className="flex flex-col gap-5">
        <Field
          label="Goal Amount (XLM)"
          required
          error={errors.goalAmount}
          hint="Set the total amount you aim to raise."
        >
          <div className="relative">
            <input
              id="goalAmount"
              type="number"
              min="1"
              step="any"
              value={form.goalAmount}
              onChange={(e) => onChange('goalAmount', e.target.value)}
              placeholder="e.g. 5000"
              aria-describedby={
                errors.goalAmount ? 'goalAmount-error' : undefined
              }
              className={`${inputClass(!!errors.goalAmount)} pr-14`}
            />
            <span className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-sm text-[var(--color-text-muted)]">
              XLM
            </span>
          </div>
        </Field>

        <Field label="Duration" required error={errors.duration}>
          <div
            role="radiogroup"
            aria-label="Campaign duration"
            className="flex flex-wrap gap-2"
          >
            {DURATION_OPTIONS.map(({ label, value }) => (
              <button
                key={value}
                type="button"
                role="radio"
                aria-checked={form.duration === value}
                onClick={() => onChange('duration', value)}
                className={`rounded-full border px-4 py-1.5 text-sm font-medium transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600 ${
                  form.duration === value
                    ? 'border-brand-600 bg-brand-600 text-white'
                    : 'border-[var(--color-border)] hover:border-brand-400 hover:text-brand-600'
                }`}
              >
                {label}
              </button>
            ))}
          </div>
        </Field>

        <Field
          label="Banner Image URL"
          hint="Optional. Provide a URL for your pool's banner image."
        >
          <input
            id="imageUrl"
            type="url"
            value={form.imageUrl}
            onChange={(e) => onChange('imageUrl', e.target.value)}
            placeholder="https://example.com/image.jpg"
            className={inputClass(false)}
          />
        </Field>

        <Field
          label="Tags"
          hint="Optional. Comma-separated tags to help people find your pool."
        >
          <input
            id="tags"
            type="text"
            value={form.tags}
            onChange={(e) => onChange('tags', e.target.value)}
            placeholder="e.g. water, africa, community"
            className={inputClass(false)}
          />
        </Field>
      </div>

      <div className="mt-8 flex justify-between">
        <button onClick={onBack} className={secondaryBtn}>
          Back
        </button>
        <button onClick={onNext} className={primaryBtn}>
          Preview Pool
        </button>
      </div>
    </div>
  );
}

/* ── Step 3: Preview ──────────────────────────────────────────────────────── */

interface Step3Props {
  form: FormData;
  tagList: string[];
  submitting: boolean;
  onBack: () => void;
  onSubmit: () => void;
}

function Step3({ form, tagList, submitting, onBack, onSubmit }: Step3Props) {
  const endDate = new Date();
  endDate.setDate(endDate.getDate() + form.duration);

  return (
    <div>
      <h2 className="mb-2 text-lg font-semibold">Preview Your Pool</h2>
      <p className="mb-6 text-sm text-[var(--color-text-muted)]">
        Review the details before creating your pool.
      </p>

      {/* Pool preview card */}
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] overflow-hidden">
        {/* Banner */}
        {form.imageUrl ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img
            src={form.imageUrl}
            alt="Pool banner"
            className="h-40 w-full object-cover"
            onError={(e) => {
              (e.currentTarget as HTMLImageElement).style.display = 'none';
            }}
          />
        ) : (
          <div
            className="h-40 w-full bg-gradient-to-br from-brand-500 to-brand-700"
            aria-hidden="true"
          />
        )}

        <div className="p-5">
          <div className="flex flex-wrap items-start justify-between gap-2">
            <h3 className="text-lg font-bold">{form.title}</h3>
            <span className="rounded-full bg-brand-100 px-2.5 py-0.5 text-xs font-medium text-brand-700">
              {form.category}
            </span>
          </div>
          <p className="mt-2 text-sm text-[var(--color-text-muted)] leading-relaxed">
            {form.description}
          </p>

          <div className="mt-4 grid grid-cols-2 gap-4 sm:grid-cols-3">
            <PreviewStat
              label="Goal"
              value={`${parseFloat(form.goalAmount).toLocaleString()} XLM`}
            />
            <PreviewStat label="Duration" value={`${form.duration} days`} />
            <PreviewStat
              label="Ends"
              value={endDate.toLocaleDateString('en-US', {
                month: 'short',
                day: 'numeric',
                year: 'numeric',
              })}
            />
          </div>

          {tagList.length > 0 && (
            <div className="mt-4 flex flex-wrap gap-1.5">
              {tagList.map((tag) => (
                <span
                  key={tag}
                  className="rounded-full border border-[var(--color-border)] px-2.5 py-0.5 text-xs text-[var(--color-text-muted)]"
                >
                  #{tag}
                </span>
              ))}
            </div>
          )}
        </div>
      </div>

      <div className="mt-8 flex justify-between">
        <button onClick={onBack} disabled={submitting} className={secondaryBtn}>
          Back
        </button>
        <button
          onClick={onSubmit}
          disabled={submitting}
          className={`${primaryBtn} disabled:opacity-60 disabled:cursor-not-allowed`}
        >
          {submitting ? (
            <span className="flex items-center gap-2">
              <SpinnerIcon />
              Creating Pool…
            </span>
          ) : (
            'Create Pool'
          )}
        </button>
      </div>
    </div>
  );
}

/* ── Success screen ───────────────────────────────────────────────────────── */

function SuccessScreen({ onGoToDashboard }: { onGoToDashboard: () => void }) {
  return (
    <main className="mx-auto max-w-2xl px-6 py-24 text-center">
      <div className="flex flex-col items-center gap-4">
        <div className="flex size-16 items-center justify-center rounded-full bg-success-light text-success">
          <CheckCircleIcon />
        </div>
        <h1 className="text-2xl font-bold">Pool Created!</h1>
        <p className="text-[var(--color-text-muted)] max-w-sm">
          Your donation pool has been created successfully. Share it with your
          community to start receiving contributions.
        </p>
        <button onClick={onGoToDashboard} className={`mt-4 ${primaryBtn}`}>
          Go to Dashboard
        </button>
      </div>
    </main>
  );
}

/* ── Field wrapper ────────────────────────────────────────────────────────── */

interface FieldProps {
  label: string;
  required?: boolean;
  error?: string;
  hint?: string;
  children: React.ReactNode;
}

function Field({ label, required, error, hint, children }: FieldProps) {
  const id = label.toLowerCase().replace(/\s+/g, '-');
  return (
    <div>
      <label
        htmlFor={id}
        className="mb-1.5 block text-sm font-medium text-[var(--color-text)]"
      >
        {label}
        {required && (
          <span className="ml-1 text-[var(--color-error)]" aria-hidden="true">
            *
          </span>
        )}
      </label>
      {hint && (
        <p className="mb-1.5 text-xs text-[var(--color-text-muted)]">{hint}</p>
      )}
      {children}
      {error && (
        <p
          id={`${id}-error`}
          role="alert"
          className="mt-1 text-xs text-[var(--color-error)]"
        >
          {error}
        </p>
      )}
    </div>
  );
}

/* ── PreviewStat ──────────────────────────────────────────────────────────── */

function PreviewStat({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="text-xs text-[var(--color-text-muted)]">{label}</p>
      <p className="mt-0.5 font-semibold text-sm">{value}</p>
    </div>
  );
}

/* ── Shared styles ────────────────────────────────────────────────────────── */

function inputClass(hasError: boolean) {
  return `w-full rounded-xl border px-3.5 py-2.5 text-sm bg-[var(--color-surface)] text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:outline-none focus:ring-2 transition-colors ${
    hasError
      ? 'border-[var(--color-error)] focus:ring-[var(--color-error)]'
      : 'border-[var(--color-border)] focus:ring-brand-500'
  }`;
}

const primaryBtn =
  'rounded-full bg-brand-600 px-6 py-2.5 text-sm font-semibold text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600';

const secondaryBtn =
  'rounded-full border border-[var(--color-border)] px-6 py-2.5 text-sm font-medium hover:bg-[var(--color-surface-raised)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600';

/* ── Icons ────────────────────────────────────────────────────────────────── */

function CheckIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={2.5}
      stroke="currentColor"
      className="size-4"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="m4.5 12.75 6 6 9-13.5"
      />
    </svg>
  );
}

function CheckCircleIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={1.5}
      stroke="currentColor"
      className="size-8"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
      />
    </svg>
  );
}

function SpinnerIcon() {
  return (
    <svg
      className="size-4 animate-spin"
      fill="none"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
      <circle
        className="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
      />
      <path
        className="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
      />
    </svg>
  );
}

export default function CreatePoolPage() {
  return (
    <ProtectedRoute>
      <CreatePoolPageContent />
    </ProtectedRoute>
  );
}
