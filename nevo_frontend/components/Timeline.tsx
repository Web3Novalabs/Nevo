import React, { FC } from 'react';

export type TimelineStepStatus = 'completed' | 'active' | 'pending' | 'error';

export interface TimelineStep {
  id: string | number;
  label: string;
  description?: string;
  status: TimelineStepStatus;
}

export interface TimelineProps {
  steps: TimelineStep[];
  /** Layout direction */
  orientation?: 'vertical' | 'horizontal';
  className?: string;
}

const statusIcon: Record<TimelineStepStatus, React.ReactNode> = {
  completed: (
    <svg className="w-4 h-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path
        d="M3 8l3.5 3.5L13 4.5"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
  error: (
    <svg className="w-4 h-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path
        d="M4 4l8 8M12 4l-8 8"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
      />
    </svg>
  ),
  active: null,
  pending: null,
};

const statusDot: Record<TimelineStepStatus, string> = {
  completed: 'bg-brand-500 text-white',
  active: 'bg-brand-500 text-white ring-4 ring-brand-100',
  pending: 'bg-[var(--color-border)] text-[var(--color-text-muted)]',
  error: 'bg-error text-white',
};

const statusConnector: Record<TimelineStepStatus, string> = {
  completed: 'bg-brand-500',
  active: 'bg-[var(--color-border)]',
  pending: 'bg-[var(--color-border)]',
  error: 'bg-[var(--color-border)]',
};

const VerticalTimeline: FC<{ steps: TimelineStep[] }> = ({ steps }) => (
  <ol className="flex flex-col gap-0">
    {steps.map((step, i) => {
      const isLast = i === steps.length - 1;
      return (
        <li key={step.id} className="flex gap-4">
          {/* Dot + connector column */}
          <div className="flex flex-col items-center">
            <span
              className={`flex items-center justify-center w-8 h-8 rounded-full shrink-0 text-sm font-semibold transition-colors ${statusDot[step.status]}`}
              aria-label={`Step ${i + 1}: ${step.status}`}
            >
              {statusIcon[step.status] ?? (
                <span className="text-xs">{i + 1}</span>
              )}
            </span>
            {!isLast && (
              <span
                className={`w-0.5 flex-1 min-h-[1.5rem] my-1 rounded-full ${statusConnector[step.status]}`}
              />
            )}
          </div>

          {/* Content */}
          <div className={`pb-6 ${isLast ? 'pb-0' : ''}`}>
            <p
              className={`text-sm font-medium leading-tight ${
                step.status === 'pending'
                  ? 'text-[var(--color-text-muted)]'
                  : 'text-[var(--color-text)]'
              }`}
            >
              {step.label}
            </p>
            {step.description && (
              <p className="text-xs text-[var(--color-text-muted)] mt-0.5">
                {step.description}
              </p>
            )}
          </div>
        </li>
      );
    })}
  </ol>
);

const HorizontalTimeline: FC<{ steps: TimelineStep[] }> = ({ steps }) => (
  <ol className="flex items-start w-full">
    {steps.map((step, i) => {
      const isLast = i === steps.length - 1;
      return (
        <li
          key={step.id}
          className={`flex flex-col items-center ${isLast ? 'flex-none' : 'flex-1'}`}
        >
          <div className="flex items-center w-full">
            <span
              className={`flex items-center justify-center w-8 h-8 rounded-full shrink-0 text-sm font-semibold transition-colors ${statusDot[step.status]}`}
              aria-label={`Step ${i + 1}: ${step.status}`}
            >
              {statusIcon[step.status] ?? (
                <span className="text-xs">{i + 1}</span>
              )}
            </span>
            {!isLast && (
              <span
                className={`flex-1 h-0.5 mx-1 rounded-full ${statusConnector[step.status]}`}
              />
            )}
          </div>
          <div className="mt-2 text-center max-w-[6rem]">
            <p
              className={`text-xs font-medium leading-tight ${
                step.status === 'pending'
                  ? 'text-[var(--color-text-muted)]'
                  : 'text-[var(--color-text)]'
              }`}
            >
              {step.label}
            </p>
            {step.description && (
              <p className="text-[0.65rem] text-[var(--color-text-muted)] mt-0.5 leading-tight">
                {step.description}
              </p>
            )}
          </div>
        </li>
      );
    })}
  </ol>
);

export const Timeline: FC<TimelineProps> = ({
  steps,
  orientation = 'vertical',
  className = '',
}) => (
  <nav aria-label="Progress" className={className}>
    {orientation === 'horizontal' ? (
      <HorizontalTimeline steps={steps} />
    ) : (
      <VerticalTimeline steps={steps} />
    )}
  </nav>
);
