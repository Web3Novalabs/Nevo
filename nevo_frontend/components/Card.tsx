import React, { HTMLAttributes } from 'react';

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  variant?: 'elevated' | 'outlined' | 'flat';
  hoverable?: boolean;
}

const Card = React.forwardRef<HTMLDivElement, CardProps>(
  (
    { className = '', variant = 'elevated', hoverable = false, ...props },
    ref
  ) => {
    const baseStyles =
      'rounded-2xl w-full transition-all duration-300 overflow-hidden';

    // Using Nevo's design tokens: --color-surface, --color-surface-raised, --color-border, --color-text, --color-text-muted
    const variants = {
      elevated:
        'bg-[var(--color-surface)] shadow-[0_4px_20px_-4px_rgba(0,0,0,0.05)] dark:shadow-[0_4px_20px_-4px_rgba(0,0,0,0.3)] hover:shadow-[0_8px_30px_-6px_rgba(0,0,0,0.1)] border border-[var(--color-border)]',
      outlined:
        'bg-transparent border border-[var(--color-border)] hover:border-brand-500/50 dark:hover:border-brand-500/50 shadow-none',
      flat: 'bg-[var(--color-surface-raised)] border border-transparent shadow-none',
    };

    const hoverStyles = hoverable
      ? 'hover:-translate-y-1 hover:border-brand-500/30 cursor-pointer'
      : '';

    return (
      <div
        ref={ref}
        data-slot="card"
        className={`${baseStyles} ${variants[variant]} ${hoverStyles} ${className}`}
        {...props}
      />
    );
  }
);
Card.displayName = 'Card';

const CardHeader = React.forwardRef<
  HTMLDivElement,
  HTMLAttributes<HTMLDivElement>
>(({ className = '', ...props }, ref) => (
  <div
    ref={ref}
    data-slot="card-header"
    className={`px-6 pt-6 pb-4 border-b border-[var(--color-border)] ${className}`}
    {...props}
  />
));
CardHeader.displayName = 'CardHeader';

const CardTitle = React.forwardRef<
  HTMLHeadingElement,
  HTMLAttributes<HTMLHeadingElement>
>(({ className = '', ...props }, ref) => (
  <h3
    ref={ref}
    data-slot="card-title"
    className={`text-lg font-bold text-[var(--color-text)] leading-tight tracking-tight ${className}`}
    {...props}
  />
));
CardTitle.displayName = 'CardTitle';

const CardDescription = React.forwardRef<
  HTMLParagraphElement,
  HTMLAttributes<HTMLParagraphElement>
>(({ className = '', ...props }, ref) => (
  <p
    ref={ref}
    data-slot="card-description"
    className={`text-sm text-[var(--color-text-muted)] mt-1.5 leading-relaxed ${className}`}
    {...props}
  />
));
CardDescription.displayName = 'CardDescription';

const CardBody = React.forwardRef<
  HTMLDivElement,
  HTMLAttributes<HTMLDivElement>
>(({ className = '', ...props }, ref) => (
  <div
    ref={ref}
    data-slot="card-body"
    className={`px-6 py-4 ${className}`}
    {...props}
  />
));
CardBody.displayName = 'CardBody';

const CardFooter = React.forwardRef<
  HTMLDivElement,
  HTMLAttributes<HTMLDivElement>
>(({ className = '', ...props }, ref) => (
  <div
    ref={ref}
    data-slot="card-footer"
    className={`px-6 py-4 border-t border-[var(--color-border)] flex items-center gap-3 ${className}`}
    {...props}
  />
));
CardFooter.displayName = 'CardFooter';

export { Card, CardHeader, CardTitle, CardDescription, CardBody, CardFooter };
