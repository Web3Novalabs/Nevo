import React from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Modal } from '@/components/Modal';

describe('Modal', () => {
  it('wraps Tab and Shift+Tab within focusable elements', async () => {
    const user = userEvent.setup();
    render(
      <Modal open onClose={jest.fn()} title="Dialog">
        <button type="button">Last action</button>
      </Modal>
    );

    const closeButton = screen.getByRole('button', { name: 'Close dialog' });
    const lastAction = screen.getByRole('button', { name: 'Last action' });

    lastAction.focus();
    await user.tab();
    expect(closeButton).toHaveFocus();

    await user.tab({ shift: true });
    expect(lastAction).toHaveFocus();
  });

  it('returns focus to the modal when focus starts outside it', () => {
    render(
      <>
        <button type="button">Outside</button>
        <Modal open onClose={jest.fn()} title="Dialog">
          <button type="button">Action</button>
        </Modal>
      </>
    );

    const outsideButton = screen.getByRole('button', { name: 'Outside' });
    const closeButton = screen.getByRole('button', { name: 'Close dialog' });
    outsideButton.focus();
    fireEvent.keyDown(document, { key: 'Tab' });

    expect(closeButton).toHaveFocus();
  });

  it('calls onClose when Escape is pressed', () => {
    const onClose = jest.fn();
    render(<Modal open onClose={onClose} title="Dialog" />);

    fireEvent.keyDown(document, { key: 'Escape' });

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('closes on backdrop click unless disabled', () => {
    const onClose = jest.fn();
    const { rerender } = render(<Modal open onClose={onClose} title="Dialog" />);
    const backdrop = document.querySelector('[aria-hidden="true"]')!;

    fireEvent.click(backdrop);
    expect(onClose).toHaveBeenCalledTimes(1);

    onClose.mockClear();
    rerender(
      <Modal
        open
        onClose={onClose}
        title="Dialog"
        disableBackdropClose
      />
    );
    fireEvent.click(document.querySelector('[aria-hidden="true"]')!);

    expect(onClose).not.toHaveBeenCalled();
  });

  it('locks body scrolling while open and restores the previous value', () => {
    document.body.style.overflow = 'clip';
    const { rerender } = render(<Modal open onClose={jest.fn()} title="Dialog" />);

    expect(document.body.style.overflow).toBe('hidden');

    rerender(<Modal open={false} onClose={jest.fn()} title="Dialog" />);

    expect(document.body.style.overflow).toBe('clip');
  });
});
