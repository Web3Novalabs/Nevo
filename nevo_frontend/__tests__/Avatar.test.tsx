import React from 'react';
import { render, screen } from '@testing-library/react';
import { Avatar } from '@/components/Avatar';

describe('Avatar', () => {
  it('shows initials for a name', () => {
    render(<Avatar name="Alice Bob" />);
    expect(
      screen.getByRole('img', { name: 'Avatar for Alice Bob' })
    ).toHaveTextContent('AB');
  });

  it('shows single initial for single-word name', () => {
    render(<Avatar name="Alice" />);
    expect(screen.getByRole('img')).toHaveTextContent('A');
  });

  it('shows ? when no name provided', () => {
    render(<Avatar />);
    expect(screen.getByRole('img')).toHaveTextContent('?');
  });

  it('applies size classes', () => {
    const { container } = render(<Avatar name="X" size="lg" />);
    expect(container.firstChild).toHaveClass('h-14');
  });

  it('shows loading state when src is provided', () => {
    render(<Avatar src="http://example.com/img.png" name="User" />);
    expect(screen.getByLabelText('Loading avatar')).toBeInTheDocument();
  });
});
