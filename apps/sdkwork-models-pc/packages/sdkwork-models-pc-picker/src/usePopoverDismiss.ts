import { useEffect, type RefObject } from 'react';

export function usePopoverDismiss(
  triggerRef: RefObject<HTMLElement | null>,
  open: boolean,
  onDismiss: () => void,
  menuRef?: RefObject<HTMLElement | null>,
): void {
  useEffect(() => {
    if (!open) {
      return undefined;
    }

    const handlePointerDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (triggerRef.current?.contains(target) || menuRef?.current?.contains(target)) {
        return;
      }
      onDismiss();
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onDismiss();
      }
    };

    document.addEventListener('mousedown', handlePointerDown);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('mousedown', handlePointerDown);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [menuRef, onDismiss, open, triggerRef]);
}
