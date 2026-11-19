import React from 'react';
import { render } from '@testing-library/react';
import { CheckIcon, CopyIcon, InfoIcon, XIcon } from '@/components/icons';

const ICONS = [
  ['CopyIcon', CopyIcon],
  ['CheckIcon', CheckIcon],
  ['XIcon', XIcon],
  ['InfoIcon', InfoIcon],
] as const;

describe('icons', () => {
  it.each(ICONS)('%s renders an svg without throwing', (_name, Icon) => {
    const { container } = render(<Icon />);
    const svg = container.querySelector('svg');

    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('aria-hidden', 'true');
    expect(svg?.querySelector('path')).toBeInTheDocument();
  });

  it.each(ICONS)('%s applies the default size-4 className', (_name, Icon) => {
    const { container } = render(<Icon />);

    expect(container.querySelector('svg')).toHaveAttribute('class', 'size-4');
  });

  it.each(ICONS)('%s forwards a custom className', (_name, Icon) => {
    const { container } = render(<Icon className="size-6 text-red-500" />);

    expect(container.querySelector('svg')).toHaveAttribute(
      'class',
      'size-6 text-red-500'
    );
  });
});
