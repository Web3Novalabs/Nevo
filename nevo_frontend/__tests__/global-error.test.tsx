import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import * as Sentry from '@sentry/nextjs';
import GlobalError from '@/app/global-error';

jest.mock('@sentry/nextjs', () => ({
  captureException: jest.fn(),
}));

describe('GlobalError', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('calls Sentry.captureException(error) exactly once when mounting with an error', () => {
    const error = new Error('Test crash');
    const reset = jest.fn();

    render(<GlobalError error={error} reset={reset} />);

    expect(
      screen.getByRole('heading', { name: /something went wrong/i })
    ).toBeInTheDocument();
    expect(Sentry.captureException).toHaveBeenCalledTimes(1);
    expect(Sentry.captureException).toHaveBeenCalledWith(error);
  });

  it("calls the reset() prop when clicking 'Try again'", async () => {
    const user = userEvent.setup();
    const error = new Error('Test crash');
    const reset = jest.fn();

    render(<GlobalError error={error} reset={reset} />);

    const button = screen.getByRole('button', { name: /try again/i });
    expect(button).toBeInTheDocument();

    await user.click(button);

    expect(reset).toHaveBeenCalledTimes(1);
  });

  it('captures again when re-rendering with a new error object (effect dependency on error)', () => {
    const firstError = new Error('First crash');
    const secondError = new Error('Second crash');
    const reset = jest.fn();

    const { rerender } = render(
      <GlobalError error={firstError} reset={reset} />
    );

    expect(Sentry.captureException).toHaveBeenCalledTimes(1);
    expect(Sentry.captureException).toHaveBeenCalledWith(firstError);

    // Re-rendering with the same error should not trigger another capture
    rerender(<GlobalError error={firstError} reset={reset} />);
    expect(Sentry.captureException).toHaveBeenCalledTimes(1);

    // Re-rendering with a new error object triggers Sentry.captureException again
    rerender(<GlobalError error={secondError} reset={reset} />);
    expect(Sentry.captureException).toHaveBeenCalledTimes(2);
    expect(Sentry.captureException).toHaveBeenLastCalledWith(secondError);
  });
});
