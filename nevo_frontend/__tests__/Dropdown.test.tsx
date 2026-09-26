import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Dropdown } from '@/components/Dropdown';
import type { DropdownItem } from '@/components/Dropdown';

const baseItems: DropdownItem[] = [
  { label: 'Item One', value: 'one', onClick: jest.fn() },
  { label: 'Item Two', value: 'two', onClick: jest.fn() },
  { label: 'Disabled Item', value: 'disabled', disabled: true, onClick: jest.fn() },
];

function renderDropdown(items = baseItems) {
  return render(
    <Dropdown trigger={<button type="button">Open</button>} items={items} />
  );
}

describe('Dropdown', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('open / close on click and outside click', () => {
    it('opens the menu when the trigger is clicked', async () => {
      const user = userEvent.setup();
      renderDropdown();

      expect(screen.queryByRole('menu')).not.toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: 'Open' }));
      expect(screen.getByRole('menu')).toBeInTheDocument();
    });

    it('closes the menu when clicking outside the dropdown', async () => {
      const user = userEvent.setup();
      render(
        <div>
          <Dropdown trigger={<button type="button">Open</button>} items={baseItems} />
          <div data-testid="outside">Outside</div>
        </div>
      );

      await user.click(screen.getByRole('button', { name: 'Open' }));
      expect(screen.getByRole('menu')).toBeInTheDocument();

      await user.pointer({ target: screen.getByTestId('outside'), keys: '[MouseLeft]' });
      expect(screen.queryByRole('menu')).not.toBeInTheDocument();
    });

    it('toggles closed when the trigger is clicked again', async () => {
      const user = userEvent.setup();
      renderDropdown();

      await user.click(screen.getByRole('button', { name: 'Open' }));
      expect(screen.getByRole('menu')).toBeInTheDocument();

      await user.click(screen.getByRole('button', { name: 'Open' }));
      expect(screen.queryByRole('menu')).not.toBeInTheDocument();
    });
  });

  describe('Escape key', () => {
    it('closes the menu when Escape is pressed', async () => {
      const user = userEvent.setup();
      renderDropdown();

      await user.click(screen.getByRole('button', { name: 'Open' }));
      expect(screen.getByRole('menu')).toBeInTheDocument();

      await user.keyboard('{Escape}');
      expect(screen.queryByRole('menu')).not.toBeInTheDocument();
    });

    it('returns focus to the trigger after Escape', async () => {
      const user = userEvent.setup();
      renderDropdown();

      const trigger = screen.getByRole('button', { name: 'Open' });
      await user.click(trigger);

      // Move focus into the menu by arrowing down
      await user.keyboard('{ArrowDown}');
      // Press Escape — focus should go back to the trigger wrapper
      await user.keyboard('{Escape}');

      // The trigger div (role=button) or its child button should be focused
      const triggerEl = document.getElementById(
        (document.querySelector('[aria-haspopup="menu"]') as HTMLElement)?.id
      );
      expect(document.activeElement).toBe(triggerEl ?? trigger);
    });
  });

  describe('item interaction', () => {
    it('calls onClick and closes the menu when a non-disabled item is clicked', async () => {
      const user = userEvent.setup();
      const onClick = jest.fn();
      const items: DropdownItem[] = [
        { label: 'Action', value: 'action', onClick },
      ];
      render(
        <Dropdown trigger={<button type="button">Open</button>} items={items} />
      );

      await user.click(screen.getByRole('button', { name: 'Open' }));
      await user.click(screen.getByRole('menuitem', { name: 'Action' }));

      expect(onClick).toHaveBeenCalledTimes(1);
      expect(screen.queryByRole('menu')).not.toBeInTheDocument();
    });

    it('does not call onClick when a disabled item is clicked', async () => {
      const user = userEvent.setup();
      const onClick = jest.fn();
      const items: DropdownItem[] = [
        { label: 'Disabled', value: 'dis', disabled: true, onClick },
      ];
      render(
        <Dropdown trigger={<button type="button">Open</button>} items={items} />
      );

      await user.click(screen.getByRole('button', { name: 'Open' }));
      // The disabled button should not fire onClick
      const disabledItem = screen.getByRole('menuitem', { name: 'Disabled' });
      expect(disabledItem).toBeDisabled();
      await user.click(disabledItem);
      expect(onClick).not.toHaveBeenCalled();
    });
  });

  describe('keyboard navigation', () => {
    it('navigates items with ArrowDown and ArrowUp', async () => {
      const user = userEvent.setup();
      renderDropdown();

      await user.click(screen.getByRole('button', { name: 'Open' }));
      const items = screen.getAllByRole('menuitem');

      // First item is focused on open
      expect(items[0]).toHaveFocus();

      await user.keyboard('{ArrowDown}');
      expect(items[1]).toHaveFocus();

      await user.keyboard('{ArrowUp}');
      expect(items[0]).toHaveFocus();
    });

    it('jumps to first item with Home key', async () => {
      const user = userEvent.setup();
      renderDropdown();

      await user.click(screen.getByRole('button', { name: 'Open' }));
      await user.keyboard('{ArrowDown}');
      await user.keyboard('{Home}');

      const items = screen.getAllByRole('menuitem');
      expect(items[0]).toHaveFocus();
    });

    it('jumps to last enabled item with End key', async () => {
      const user = userEvent.setup();
      renderDropdown();

      await user.click(screen.getByRole('button', { name: 'Open' }));
      await user.keyboard('{End}');

      // Last enabled item is 'Item Two' (index 1) because 'Disabled Item' is disabled
      const items = screen.getAllByRole('menuitem');
      // End moves to last enabled index
      expect(items[1]).toHaveFocus();
    });
  });
});
