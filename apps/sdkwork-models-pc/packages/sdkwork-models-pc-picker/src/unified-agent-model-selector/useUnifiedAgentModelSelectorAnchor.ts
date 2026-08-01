import { useEffect, useState, type CSSProperties, type RefObject } from 'react';

const MENU_WIDTH = 420;
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
      setStyle({
        bottom: Math.max(VIEWPORT_GUTTER, window.innerHeight - rect.top + 8),
        left,
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
