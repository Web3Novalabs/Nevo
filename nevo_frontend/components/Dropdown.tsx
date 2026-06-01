'use client';

import React, {
  FC,
  HTMLAttributes,
  KeyboardEvent,
  ReactNode,
  useCallback,
  useEffect,
  useId,
  useRef,
  useState,
} from 'react';

export interface DropdownItem {
  label: string;
  value: string;
  icon?: ReactNode;
  disabled?: boolean;
  className?: string;
  children?: DropdownItem[];
  onClick?: () => void;
}

export interface DropdownProps extends Omit<
  HTMLAttributes<HTMLDivElement>,
  'children'
> {
  trigger: ReactNode;
  items: DropdownItem[];
  triggerType?: 'click' | 'hover';
  placement?: 'bottom-start' | 'bottom-end' | 'top-start' | 'top-end';
  menuClassName?: string;
  itemClassName?: string;
}

const ARROW_DOWN = 'ArrowDown';
const ARROW_UP = 'ArrowUp';
const ENTER = 'Enter';
const SPACE = ' ';
const ESCAPE = 'Escape';
const HOME = 'Home';
const END = 'End';

function placementClasses(placement: DropdownProps['placement']): string {
  switch (placement) {
    case 'bottom-end':
      return 'top-full right-0 mt-1';
    case 'top-start':
      return 'bottom-full left-0 mb-1';
    case 'top-end':
      return 'bottom-full right-0 mb-1';
    case 'bottom-start':
    default:
      return 'top-full left-0 mt-1';
  }
}

const DropdownMenu: FC<{
  items: DropdownItem[];
  menuId: string;
  activeIndex: number;
  itemRefs: React.MutableRefObject<(HTMLButtonElement | null)[]>;
  itemClassName?: string;
  onSelect: (item: DropdownItem) => void;
  onKeyDown: (e: KeyboardEvent<HTMLButtonElement>, index: number) => void;
  depth?: number;
}> = ({
  items,
  menuId,
  activeIndex,
  itemRefs,
  itemClassName,
  onSelect,
  onKeyDown,
  depth = 0,
}) => {
  const [openSubmenuIndex, setOpenSubmenuIndex] = useState<number | null>(null);

  return (
    <>
      {items.map((item, index) => {
        const hasChildren = item.children && item.children.length > 0;
        const flatIndex = depth === 0 ? index : index + 1000 * depth;
        return (
          <div key={item.value} className="relative">
            <button
              id={`${menuId}-item-${flatIndex}`}
              role="menuitem"
              aria-haspopup={hasChildren ? 'menu' : undefined}
              aria-expanded={
                hasChildren ? openSubmenuIndex === index : undefined
              }
              aria-disabled={item.disabled}
              tabIndex={activeIndex === flatIndex ? 0 : -1}
              ref={(el) => {
                itemRefs.current[flatIndex] = el;
              }}
              className={[
                'flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm text-left',
                'text-gray-900 dark:text-gray-100',
                'transition-colors',
                item.disabled
                  ? 'opacity-50 cursor-not-allowed'
                  : 'hover:bg-gray-100 dark:hover:bg-gray-700 focus:bg-gray-100 dark:focus:bg-gray-700 focus:outline-none cursor-pointer',
                itemClassName ?? '',
                item.className ?? '',
              ]
                .filter(Boolean)
                .join(' ')}
              disabled={item.disabled}
              onClick={() => {
                if (hasChildren) {
                  setOpenSubmenuIndex(
                    openSubmenuIndex === index ? null : index
                  );
                  return;
                }
                onSelect(item);
              }}
              onMouseEnter={() => {
                if (hasChildren) setOpenSubmenuIndex(index);
              }}
              onMouseLeave={() => {
                if (hasChildren) setOpenSubmenuIndex(null);
              }}
              onKeyDown={(e) => onKeyDown(e, flatIndex)}
            >
              {item.icon && (
                <span className="shrink-0" aria-hidden="true">
                  {item.icon}
                </span>
              )}
              <span className="flex-1">{item.label}</span>
              {hasChildren && (
                <span aria-hidden="true" className="ml-auto">
                  <svg
                    className="h-4 w-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    strokeWidth="2"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      d="M9 5l7 7-7 7"
                    />
                  </svg>
                </span>
              )}
            </button>
            {hasChildren && openSubmenuIndex === index && (
              <div
                role="menu"
                aria-label={item.label}
                className="absolute left-full top-0 z-50 min-w-36 rounded-md border border-gray-200 bg-white p-1 shadow-lg dark:border-gray-700 dark:bg-gray-800"
              >
                <DropdownMenu
                  items={item.children!}
                  menuId={menuId}
                  activeIndex={activeIndex}
                  itemRefs={itemRefs}
                  itemClassName={itemClassName}
                  onSelect={onSelect}
                  onKeyDown={onKeyDown}
                  depth={depth + 1}
                />
              </div>
            )}
          </div>
        );
      })}
    </>
  );
};

export const Dropdown: FC<DropdownProps> = ({
  trigger,
  items,
  triggerType = 'click',
  placement = 'bottom-start',
  menuClassName = '',
  itemClassName = '',
  className = '',
  ...props
}) => {
  const uid = useId();
  const menuId = `dropdown-menu-${uid}`;
  const triggerId = `dropdown-trigger-${uid}`;

  const [isOpen, setIsOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);

  const containerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);

  const close = useCallback(() => {
    setIsOpen(false);
    setActiveIndex(0);
  }, []);

  const open = useCallback(() => {
    setIsOpen(true);
    setActiveIndex(0);
  }, []);

  // Click-outside to close
  useEffect(() => {
    if (!isOpen) return;

    function handlePointerDown(e: PointerEvent) {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        close();
      }
    }

    document.addEventListener('pointerdown', handlePointerDown);
    return () => document.removeEventListener('pointerdown', handlePointerDown);
  }, [isOpen, close]);

  // Focus the active item when open or active index changes
  useEffect(() => {
    if (isOpen && itemRefs.current[activeIndex]) {
      itemRefs.current[activeIndex]?.focus();
    }
  }, [isOpen, activeIndex]);

  const flatItems = items; // top-level items for keyboard nav index

  function handleTriggerKeyDown(e: KeyboardEvent<HTMLDivElement>) {
    if (e.key === ARROW_DOWN || e.key === ARROW_UP) {
      e.preventDefault();
      if (!isOpen) {
        open();
        setActiveIndex(e.key === ARROW_DOWN ? 0 : flatItems.length - 1);
      }
    }
    if (e.key === ESCAPE && isOpen) {
      e.preventDefault();
      close();
    }
  }

  function handleItemKeyDown(
    e: KeyboardEvent<HTMLButtonElement>,
    index: number
  ) {
    const enabledIndices = flatItems
      .map((_, i) => i)
      .filter((i) => !flatItems[i].disabled);

    const currentPos = enabledIndices.indexOf(index);

    if (e.key === ARROW_DOWN) {
      e.preventDefault();
      const next =
        enabledIndices[Math.min(currentPos + 1, enabledIndices.length - 1)];
      setActiveIndex(next);
    } else if (e.key === ARROW_UP) {
      e.preventDefault();
      const prev = enabledIndices[Math.max(currentPos - 1, 0)];
      setActiveIndex(prev);
    } else if (e.key === HOME) {
      e.preventDefault();
      setActiveIndex(enabledIndices[0]);
    } else if (e.key === END) {
      e.preventDefault();
      setActiveIndex(enabledIndices[enabledIndices.length - 1]);
    } else if (e.key === ESCAPE) {
      e.preventDefault();
      close();
      document.getElementById(triggerId)?.focus();
    } else if (e.key === ENTER || e.key === SPACE) {
      e.preventDefault();
      itemRefs.current[index]?.click();
    }
  }

  function handleSelect(item: DropdownItem) {
    item.onClick?.();
    close();
  }

  const hoverProps =
    triggerType === 'hover'
      ? {
          onMouseEnter: open,
          onMouseLeave: close,
        }
      : {};

  return (
    <div
      ref={containerRef}
      className={`relative inline-block ${className}`}
      onKeyDown={handleTriggerKeyDown}
      {...hoverProps}
      {...props}
    >
      <div
        id={triggerId}
        role="button"
        aria-haspopup="menu"
        aria-expanded={isOpen}
        aria-controls={menuId}
        tabIndex={0}
        className="cursor-pointer focus:outline-none"
        onClick={() => {
          if (triggerType === 'click') {
            isOpen ? close() : open();
          }
        }}
        onKeyDown={(e) => {
          if (e.key === ENTER || e.key === SPACE) {
            e.preventDefault();
            if (triggerType === 'click') {
              isOpen ? close() : open();
            }
          }
        }}
      >
        {trigger}
      </div>

      {isOpen && (
        <div
          id={menuId}
          role="menu"
          aria-labelledby={triggerId}
          className={[
            'absolute z-50 min-w-36 rounded-md border border-gray-200 bg-white p-1 shadow-lg',
            'dark:border-gray-700 dark:bg-gray-800',
            placementClasses(placement),
            menuClassName,
          ]
            .filter(Boolean)
            .join(' ')}
        >
          <DropdownMenu
            items={items}
            menuId={menuId}
            activeIndex={activeIndex}
            itemRefs={itemRefs}
            itemClassName={itemClassName}
            onSelect={handleSelect}
            onKeyDown={handleItemKeyDown}
          />
        </div>
      )}
    </div>
  );
};
