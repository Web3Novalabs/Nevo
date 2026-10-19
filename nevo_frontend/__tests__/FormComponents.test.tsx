import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  Checkbox,
  FormField,
  Input,
  Select,
  Textarea,
} from '@/components/form';

describe('Form components', () => {
  describe('Input', () => {
    it('renders text input', () => {
      render(<Input label="Title" />);
      expect(screen.getByLabelText(/Title/)).toHaveAttribute('type', 'text');
    });

    it('renders email, number, and password types', () => {
      const { rerender } = render(<Input type="email" label="Email" />);
      expect(screen.getByLabelText(/Email/)).toHaveAttribute('type', 'email');

      rerender(<Input type="number" label="Amount" />);
      expect(screen.getByLabelText(/Amount/)).toHaveAttribute('type', 'number');

      rerender(<Input type="password" label="Password" />);
      expect(screen.getByLabelText(/Password/)).toHaveAttribute(
        'type',
        'password'
      );
    });

    it('renders label, helper text, and required indicator', () => {
      render(
        <Input label="Email" helperText="We never share your email." required />
      );
      expect(screen.getByLabelText(/Email/)).toBeRequired();
      expect(screen.getByText('*')).toBeInTheDocument();
      expect(
        screen.getByText('We never share your email.')
      ).toBeInTheDocument();
    });

    it('shows error state with alert message', () => {
      render(<Input label="Name" error="Name is required" />);
      const input = screen.getByLabelText(/Name/);
      expect(input).toHaveAttribute('aria-invalid', 'true');
      expect(input).toHaveAttribute('aria-describedby', `${input.id}-error`);
      expect(screen.getByRole('alert')).toHaveTextContent('Name is required');
    });

    it('is disabled when the disabled prop is set', () => {
      render(<Input label="Name" disabled />);
      expect(screen.getByLabelText(/Name/)).toBeDisabled();
    });

    it('accepts keyboard input', async () => {
      render(<Input label="Title" />);
      const input = screen.getByLabelText(/Title/);
      await userEvent.type(input, 'Clean Water');
      expect(input).toHaveValue('Clean Water');
    });
  });

  describe('Textarea', () => {
    it('renders multi-line input with label', async () => {
      render(<Textarea label="Description" placeholder="Describe..." />);
      const textarea = screen.getByLabelText(/Description/);
      expect(textarea.tagName).toBe('TEXTAREA');
      await userEvent.type(textarea, 'Hello\nWorld');
      expect(textarea).toHaveValue('Hello\nWorld');
    });

    it('shows error message', () => {
      render(<Textarea label="Bio" error="Bio is too short" />);
      const textarea = screen.getByLabelText(/Bio/);
      expect(textarea).toHaveAttribute('aria-invalid', 'true');
      expect(textarea).toHaveAttribute(
        'aria-describedby',
        `${textarea.id}-error`
      );
      expect(screen.getByRole('alert')).toHaveTextContent('Bio is too short');
    });

    it('is disabled when the disabled prop is set', () => {
      render(<Textarea label="Bio" disabled />);
      expect(screen.getByLabelText(/Bio/)).toBeDisabled();
    });
  });

  describe('Select', () => {
    const options = [
      { label: 'Education', value: 'education' },
      { label: 'Health', value: 'health' },
    ];

    it('renders options with placeholder', () => {
      render(
        <Select
          label="Category"
          placeholder="Select a category"
          options={options}
        />
      );
      expect(screen.getByLabelText(/Category/)).toBeInTheDocument();
      expect(
        screen.getByRole('option', { name: 'Select a category' })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('option', { name: 'Education' })
      ).toBeInTheDocument();
    });

    it('supports keyboard selection', async () => {
      render(<Select label="Category" options={options} />);
      const select = screen.getByLabelText(/Category/);
      await userEvent.selectOptions(select, 'health');
      expect(select).toHaveValue('health');
    });

    it('shows error state', () => {
      render(
        <Select label="Category" options={options} error="Pick a category" />
      );
      const select = screen.getByLabelText(/Category/);
      expect(select).toHaveAttribute('aria-invalid', 'true');
      expect(select).toHaveAttribute('aria-describedby', `${select.id}-error`);
      expect(screen.getByRole('alert')).toHaveTextContent('Pick a category');
    });

    it('is disabled when the disabled prop is set', () => {
      render(<Select label="Category" options={options} disabled />);
      expect(screen.getByLabelText(/Category/)).toBeDisabled();
    });
  });

  describe('Checkbox', () => {
    it('associates label with checkbox for accessibility', async () => {
      const onChange = jest.fn();
      render(
        <Checkbox
          label="Accept terms"
          helperText="Required to continue."
          onChange={onChange}
        />
      );
      const checkbox = screen.getByRole('checkbox', { name: /Accept terms/ });
      expect(checkbox).not.toBeChecked();
      await userEvent.click(checkbox);
      expect(onChange).toHaveBeenCalled();
    });

    it('shows error message and invalid state', () => {
      render(<Checkbox label="Subscribe" error="You must subscribe" />);
      const checkbox = screen.getByRole('checkbox', { name: 'Subscribe' });
      expect(checkbox).toHaveAttribute('aria-invalid', 'true');
      expect(checkbox).toHaveAttribute(
        'aria-describedby',
        `${checkbox.id}-error`
      );
      expect(screen.getByRole('alert')).toHaveTextContent('You must subscribe');
    });

    it('is disabled when the disabled prop is set', () => {
      render(<Checkbox label="Subscribe" disabled />);
      expect(screen.getByRole('checkbox', { name: 'Subscribe' })).toBeDisabled();
    });

    it('is keyboard focusable', () => {
      render(<Checkbox label="Remember me" />);
      const checkbox = screen.getByRole('checkbox');
      checkbox.focus();
      expect(checkbox).toHaveFocus();
    });
  });

  describe('FormField', () => {
    it('associates the label and helper with a custom control', () => {
      render(
        <FormField
          id="custom"
          label="Custom"
          helperText="Helpful hint"
        >
          <input id="custom" />
        </FormField>
      );
      const input = screen.getByLabelText('Custom');
      expect(input).toHaveAttribute('aria-describedby', 'custom-helper');
      expect(screen.getByText('Helpful hint')).toHaveAttribute(
        'id',
        'custom-helper'
      );
    });

    it('associates the error with a custom control', () => {
      render(
        <FormField id="custom" label="Custom" error="Invalid">
          <input id="custom" />
        </FormField>
      );

      const input = screen.getByLabelText('Custom');
      expect(input).toHaveAttribute('aria-describedby', 'custom-error');
      expect(screen.getByRole('alert')).toHaveTextContent('Invalid');
    });

    it('preserves disabled state on a custom control', () => {
      render(
        <FormField id="custom" label="Custom">
          <input id="custom" disabled />
        </FormField>
      );

      expect(screen.getByLabelText('Custom')).toBeDisabled();
    });
  });
});
