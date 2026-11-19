import React from 'react';
import { render, screen } from '@testing-library/react';
import { Spinner } from '@/components/Spinner';

describe('Spinner', () => {
  it('exposes a labeled status role to screen readers', () => {
    render(<Spinner label="Loading donations" />);

    expect(
      screen.getByRole('status', { name: 'Loading donations' })
    ).toBeInTheDocument();
  });

  it.each([
    ['sm', 'h-4 w-4'],
    ['md', 'h-6 w-6'],
    ['lg', 'h-8 w-8'],
    ['xl', 'h-12 w-12'],
  ] as const)('applies the expected %s size classes', (size, className) => {
    render(<Spinner size={size} />);

    expect(screen.getByRole('status')).toHaveClass(...className.split(' '));
  });

  it('defaults to the medium size', () => {
    render(<Spinner />);

    expect(screen.getByRole('status')).toHaveClass('h-6', 'w-6');
  });
});
