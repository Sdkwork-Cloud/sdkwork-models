import {
  useEffect,
  useId,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent as ReactKeyboardEvent,
} from 'react';
import { createPortal } from 'react-dom';
import { ChevronDown } from 'lucide-react';
import type { ModelVendor } from './agentModelAccessSelectorTypes';

export interface VendorCodeComboboxProps {
  disabled: boolean;
  inputId: string;
  listLabel: string;
  onChange: (code: string) => void;
  options: readonly ModelVendor[];
  placeholder: string;
  value: string;
}

const DROPDOWN_MAX_HEIGHT = 220;
const DROPDOWN_MIN_HEIGHT = 80;
const DROPDOWN_OFFSET = 4;
const VIEWPORT_GUTTER = 8;

function vendorSortOrder(vendor: ModelVendor): number {
  const value = vendor.sortOrder;
  if (typeof value === 'number') {
    return Number.isFinite(value) ? value : Number.MAX_SAFE_INTEGER;
  }
  if (typeof value === 'string' && value.trim()) {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : Number.MAX_SAFE_INTEGER;
  }
  return Number.MAX_SAFE_INTEGER;
}

function compareVendorCode(left: ModelVendor, right: ModelVendor): number {
  const normalizedLeft = left.code.trim().toLowerCase();
  const normalizedRight = right.code.trim().toLowerCase();
  return normalizedLeft < normalizedRight ? -1 : normalizedLeft > normalizedRight ? 1 : 0;
}

function vendorMatchesQuery(vendor: ModelVendor, query: string): boolean {
  return vendor.code.trim().toLowerCase().includes(query)
    || vendor.name.trim().toLowerCase().includes(query);
}

/**
 * Vendor code field with a select-style dropdown and free-text input.
 * The dropdown always shows the complete vendor list with full codes; the
 * typed query only moves the highlight to the first match. Anything the user
 * types that is not in the catalog stays valid so future vendor codes keep
 * working. The popup is anchored in the viewport so it is never clipped by
 * the dialog body's scroll container.
 */
export function VendorCodeCombobox({
  disabled,
  inputId,
  listLabel,
  onChange,
  options,
  placeholder,
  value,
}: VendorCodeComboboxProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const listboxId = useId();
  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);
  const [anchor, setAnchor] = useState<CSSProperties | undefined>(undefined);
  const query = value.trim().toLowerCase();
  const sortedOptions = [...options].sort(
    (left, right) => vendorSortOrder(left) - vendorSortOrder(right)
      || compareVendorCode(left, right),
  );
  const preferredIndex = query
    ? sortedOptions.findIndex((vendor) => vendorMatchesQuery(vendor, query))
    : 0;
  const expanded = open && sortedOptions.length > 0;

  // While typing, follow the first matching option without hiding the rest
  // of the list; a non-matching query (custom vendor code) leaves the
  // highlight where it is.
  useEffect(() => {
    if (preferredIndex >= 0) {
      setActiveIndex(preferredIndex);
    }
  }, [preferredIndex]);

  useEffect(() => {
    if (!expanded) {
      setAnchor(undefined);
      return undefined;
    }
    const update = () => {
      const rect = inputRef.current?.getBoundingClientRect();
      if (!rect) {
        return;
      }
      const width = Math.min(rect.width, window.innerWidth - VIEWPORT_GUTTER * 2);
      const left = Math.min(
        Math.max(VIEWPORT_GUTTER, rect.left),
        window.innerWidth - width - VIEWPORT_GUTTER,
      );
      const availableBelow = window.innerHeight - rect.bottom - DROPDOWN_OFFSET - VIEWPORT_GUTTER;
      const availableAbove = rect.top - DROPDOWN_OFFSET - VIEWPORT_GUTTER;
      const placeAbove = availableBelow < DROPDOWN_MIN_HEIGHT
        && availableAbove > availableBelow;
      const availableHeight = Math.max(
        DROPDOWN_MIN_HEIGHT,
        Math.min(DROPDOWN_MAX_HEIGHT, placeAbove ? availableAbove : availableBelow),
      );
      setAnchor(placeAbove ? {
        bottom: window.innerHeight - rect.top + DROPDOWN_OFFSET,
        left,
        maxHeight: availableHeight,
        width,
      } : {
        left,
        maxHeight: availableHeight,
        top: rect.bottom + DROPDOWN_OFFSET,
        width,
      });
    };
    update();
    window.addEventListener('resize', update);
    window.addEventListener('scroll', update, true);
    return () => {
      window.removeEventListener('resize', update);
      window.removeEventListener('scroll', update, true);
    };
  }, [expanded]);

  useEffect(() => {
    if (!open) {
      return undefined;
    }
    // The popup is portaled to the document body, so clicks inside it are not
    // inside the combobox root. pointerdown fires before the option's
    // mousedown, so closing here would unmount the popup before the option can
    // be selected.
    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as Node;
      if (rootRef.current?.contains(target) || listRef.current?.contains(target)) {
        return;
      }
      setOpen(false);
    };
    document.addEventListener('pointerdown', handlePointerDown);
    return () => document.removeEventListener('pointerdown', handlePointerDown);
  }, [open]);

  useEffect(() => {
    if (!expanded) {
      return;
    }
    const active = listRef.current?.querySelector<HTMLElement>('[data-active="true"]');
    if (typeof active?.scrollIntoView === 'function') {
      active.scrollIntoView({ block: 'nearest' });
    }
  }, [activeIndex, expanded]);

  const choose = (code: string) => {
    onChange(code);
    setOpen(false);
  };

  const handleKeyDown = (event: ReactKeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setOpen(true);
      setActiveIndex((current) => Math.min(current + 1, sortedOptions.length - 1));
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActiveIndex((current) => Math.max(current - 1, 0));
    } else if (event.key === 'Enter') {
      if (expanded) {
        event.preventDefault();
        const vendor = sortedOptions[activeIndex];
        // Only commit a listed vendor when it actually matches the typed
        // query; otherwise keep the free-text custom code.
        if (vendor && (!query || vendorMatchesQuery(vendor, query))) {
          choose(vendor.code);
        } else {
          setOpen(false);
        }
      }
    } else if (event.key === 'Escape') {
      if (open) {
        event.preventDefault();
        setOpen(false);
      }
    }
  };

  return (
    <div className="sdkwork-model-access-vendor-combobox" ref={rootRef}>
      <input
        ref={inputRef}
        aria-activedescendant={expanded ? `${listboxId}-${activeIndex}` : undefined}
        aria-autocomplete="list"
        aria-controls={expanded ? listboxId : undefined}
        aria-expanded={expanded}
        autoComplete="off"
        disabled={disabled}
        id={inputId}
        maxLength={128}
        onChange={(event) => {
          onChange(event.target.value);
          setOpen(true);
        }}
        onFocus={() => {
          setOpen(true);
          setActiveIndex(preferredIndex >= 0 ? preferredIndex : 0);
        }}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        role="combobox"
        value={value}
      />
      <span aria-hidden="true" className="sdkwork-model-access-vendor-combobox-chevron">
        <ChevronDown size={15} />
      </span>
      {expanded ? createPortal(
        <ul
          ref={listRef}
          aria-label={listLabel}
          className="sdkwork-model-access-vendor-combobox-popup"
          id={listboxId}
          role="listbox"
          style={anchor}
        >
          {sortedOptions.map((vendor, index) => (
            <li
              aria-selected={index === activeIndex}
              className="sdkwork-model-access-vendor-combobox-option"
              data-active={index === activeIndex || undefined}
              id={`${listboxId}-${index}`}
              key={vendor.code}
              onMouseDown={(event) => {
                event.preventDefault();
                choose(vendor.code);
              }}
              onMouseEnter={() => setActiveIndex(index)}
              role="option"
            >
              <span className="sdkwork-model-access-vendor-combobox-code">{vendor.code}</span>
              <span className="sdkwork-model-access-vendor-combobox-name">{vendor.name}</span>
            </li>
          ))}
        </ul>,
        document.body,
      ) : null}
    </div>
  );
}
