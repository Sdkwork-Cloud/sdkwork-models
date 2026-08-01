import { useEffect, useState, type CSSProperties, type RefObject } from 'react';

const MENU_WIDTH = 420;
const MENU_MAX_HEIGHT = 520;
const MENU_MIN_HEIGHT = 180;
const MENU_OFFSET = 8;
const VIEWPORT_GUTTER = 12;

export function useUnifiedAgentModelSelectorAnchor(
  triggerRef: RefObject<HTMLElement | null>,
  open: boolean,
) {
  const [style, setStyle] = useState<CSSProperties>({});

  useEffect(() => {
    if (!open) {
      return undefined;
    }

    const update = () => {
      const rect = triggerRef.current?.getBoundingClientRect();
      if (!rect) {
        return;
      }
      const width = Math.min(MENU_WIDTH, window.innerWidth - VIEWPORT_GUTTER * 2);
      const left = Math.min(
        Math.max(VIEWPORT_GUTTER, rect.right - width),
        window.innerWidth - width - VIEWPORT_GUTTER,
      );
      const availableAbove = rect.top - MENU_OFFSET - VIEWPORT_GUTTER;
      const availableBelow = window.innerHeight - rect.bottom - MENU_OFFSET - VIEWPORT_GUTTER;
      const placeAbove = availableAbove >= MENU_MIN_HEIGHT || availableAbove >= availableBelow;
      const availableHeight = Math.max(
        MENU_MIN_HEIGHT,
        placeAbove ? availableAbove : availableBelow,
      );
      setStyle(placeAbove ? {
        bottom: Math.max(VIEWPORT_GUTTER, window.innerHeight - rect.top + MENU_OFFSET),
        left,
        maxHeight: Math.min(MENU_MAX_HEIGHT, availableHeight),
        width,
      } : {
        left,
        maxHeight: Math.min(MENU_MAX_HEIGHT, availableHeight),
        top: Math.min(
          window.innerHeight - VIEWPORT_GUTTER - MENU_MIN_HEIGHT,
          rect.bottom + MENU_OFFSET,
        ),
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
  }, [open, triggerRef]);

  return style;
}
