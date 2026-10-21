import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { CopyButton } from '@/components/CopyButton';
import { useCopyToClipboard } from '@/hooks/useCopyToClipboard';

jest.mock('@/hooks/useCopyToClipboard');
jest.mock('@/components/Toast', () => ({
  toast: jest.fn(),
}));

const mockUseCopyToClipboard = useCopyToClipboard as jest.MockedFunction<
  typeof useCopyToClipboard
>;

describe('CopyButton', () => {
  const mockCopy = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    mockUseCopyToClipboard.mockReturnValue({
      copied: false,
      error: null,
      copy: mockCopy,
    });
  });

  it('mocks useCopyToClipboard and verifies clicking triggers copy(text)', async () => {
    const user = userEvent.setup();
    render(<CopyButton text="https://example.com" />);

    const button = screen.getByRole('button', { name: 'Copy to clipboard' });
    await user.click(button);

    expect(mockCopy).toHaveBeenCalledWith('https://example.com');
  });

  it('swaps label and icon between Copy and Copied! states', () => {
    const { rerender } = render(<CopyButton text="https://example.com" />);

    expect(screen.getByText('Copy')).toBeInTheDocument();

    mockUseCopyToClipboard.mockReturnValue({
      copied: true,
      error: null,
      copy: mockCopy,
    });

    rerender(<CopyButton text="https://example.com" />);

    expect(screen.queryByText('Copy')).not.toBeInTheDocument();
    expect(screen.getByText('Copied!')).toBeInTheDocument();
    expect(screen.getByRole('button')).toHaveAttribute('aria-label', 'Copied');
  });

  it('hides the text label when iconOnly prop is set', () => {
    render(<CopyButton text="https://example.com" iconOnly />);

    expect(screen.queryByText('Copy')).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Copy to clipboard' })).toBeInTheDocument();
  });
});
