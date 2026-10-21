import React from 'react';
import { render, screen } from '@testing-library/react';
import ProgressBar from './ProgressBar';

describe('ProgressBar', () => {
  describe('percentage calculation and clamping', () => {
    it('clamps pct to 100 when value > max', () => {
      const { container } = render(
        <ProgressBar value={150} max={100} showPercent />
      );
      const fill = container.querySelector('.bg-brand-500') as HTMLElement;
      expect(fill).toBeInTheDocument();
      expect(fill.style.width).toBe('100%');
      expect(screen.getByText('100%')).toBeInTheDocument();
    });

    it('yields pct 0 when max === 0 (no division by zero)', () => {
      const { container } = render(<ProgressBar value={50} max={0} />);
      const fill = container.querySelector('.bg-brand-500') as HTMLElement;
      expect(fill).toBeInTheDocument();
      expect(fill.style.width).toBe('0%');
      expect(fill.style.width).not.toContain('NaN');
    });

    it('computes pct correctly for a normal value', () => {
      const { container } = render(<ProgressBar value={25} max={100} />);
      const fill = container.querySelector('.bg-brand-500') as HTMLElement;
      expect(fill.style.width).toBe('25%');
    });
  });

  describe('aria attributes', () => {
    it('reflects value/0/max in aria-valuenow/min/max', () => {
      render(<ProgressBar value={42} max={100} />);
      const bar = screen.getByRole('progressbar');
      expect(bar).toHaveAttribute('aria-valuenow', '42');
      expect(bar).toHaveAttribute('aria-valuemin', '0');
      expect(bar).toHaveAttribute('aria-valuemax', '100');
    });

    it('reflects custom max in aria-valuemax', () => {
      render(<ProgressBar value={5} max={20} />);
      const bar = screen.getByRole('progressbar');
      expect(bar).toHaveAttribute('aria-valuenow', '5');
      expect(bar).toHaveAttribute('aria-valuemin', '0');
      expect(bar).toHaveAttribute('aria-valuemax', '20');
    });
  });

  describe('variant classes', () => {
    const variants: Array<{
      variant: 'brand' | 'success' | 'warning' | 'error';
      track: string;
      fill: string;
    }> = [
      { variant: 'brand', track: 'bg-brand-100', fill: 'bg-brand-500' },
      { variant: 'success', track: 'bg-success-light', fill: 'bg-success' },
      { variant: 'warning', track: 'bg-warning-light', fill: 'bg-warning' },
      { variant: 'error', track: 'bg-error-light', fill: 'bg-error' },
    ];

    it.each(variants)(
      'applies track/fill classes for variant $variant',
      ({ variant, track, fill }) => {
        const { container } = render(
          <ProgressBar value={50} max={100} variant={variant} />
        );
        const trackEl = container.querySelector(`.${track}`);
        const fillEl = container.querySelector(`.${fill}`);
        expect(trackEl).toBeInTheDocument();
        expect(fillEl).toBeInTheDocument();
      }
    );

    it('defaults to brand variant', () => {
      const { container } = render(<ProgressBar value={50} max={100} />);
      expect(container.querySelector('.bg-brand-100')).toBeInTheDocument();
      expect(container.querySelector('.bg-brand-500')).toBeInTheDocument();
    });
  });

  describe('size classes', () => {
    const sizes: Array<{ size: 'sm' | 'md' | 'lg'; height: string }> = [
      { size: 'sm', height: 'h-1.5' },
      { size: 'md', height: 'h-2.5' },
      { size: 'lg', height: 'h-4' },
    ];

    it.each(sizes)(
      'applies track height $height for size $size',
      ({ size, height }) => {
        const { container } = render(
          <ProgressBar value={50} max={100} size={size} />
        );
        const trackEl = container.querySelector('.bg-brand-100');
        expect(trackEl).toHaveClass(height);
      }
    );

    it('defaults to md size', () => {
      const { container } = render(<ProgressBar value={50} max={100} />);
      const trackEl = container.querySelector('.bg-brand-100');
      expect(trackEl).toHaveClass('h-2.5');
    });
  });

  describe('labels', () => {
    it('renders label when provided', () => {
      render(<ProgressBar value={50} max={100} label="Uploading" />);
      expect(screen.getByText('Uploading')).toBeInTheDocument();
    });

    it('renders valueLabel and maxLabel when provided', () => {
      render(
        <ProgressBar
          value={30}
          max={100}
          valueLabel="30 MB"
          maxLabel="100 MB"
        />
      );
      expect(screen.getByText('30 MB')).toBeInTheDocument();
      expect(screen.getByText('100 MB')).toBeInTheDocument();
    });

    it('does not render percent text when showPercent is false', () => {
      render(<ProgressBar value={50} max={100} />);
      expect(screen.queryByText('50%')).not.toBeInTheDocument();
    });
  });
});