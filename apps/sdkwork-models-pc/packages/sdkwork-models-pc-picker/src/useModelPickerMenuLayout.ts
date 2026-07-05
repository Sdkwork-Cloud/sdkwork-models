import { useLayoutEffect, useState, type CSSProperties, type RefObject } from 'react';
import {
  measureModelPickerMenuContent,
  resolveModelPickerMenuLayout,
  type ModelPickerMenuLayoutResult,
  type ModelPickerMenuPlacement,
} from './modelPickerMenuLayout';

const MENU_GAP_PX = 8;
const VIEWPORT_PADDING_PX = 16;
const MENU_Z_INDEX = 10000;

function scrollActiveVendorIntoView(menu: HTMLElement): void {
  const panel = menu.querySelector<HTMLElement>('.sdkwork-model-picker-vendor-list');
  const activeItem = panel?.querySelector<HTMLElement>('.sdkwork-model-picker-vendor-button[data-active="true"]');
  if (!panel || !activeItem) {
    return;
  }

  const panelRect = panel.getBoundingClientRect();
  const itemRect = activeItem.getBoundingClientRect();
  if (itemRect.top < panelRect.top) {
    panel.scrollTop -= panelRect.top - itemRect.top;
  } else if (itemRect.bottom > panelRect.bottom) {
    panel.scrollTop += itemRect.bottom - panelRect.bottom;
  }
}

function scrollActiveModelIntoView(menu: HTMLElement): void {
  const panel = menu.querySelector<HTMLElement>('.sdkwork-model-picker-models');
  const activeItem = panel?.querySelector<HTMLElement>('[data-active="true"]');
  if (!panel || !activeItem) {
    return;
  }

  const panelRect = panel.getBoundingClientRect();
  const itemRect = activeItem.getBoundingClientRect();
  if (itemRect.top < panelRect.top) {
    panel.scrollTop -= panelRect.top - itemRect.top;
  } else if (itemRect.bottom > panelRect.bottom) {
    panel.scrollTop += itemRect.bottom - panelRect.bottom;
  }
}

function measurePickerPanels(menu: HTMLElement) {
  const vendorList = menu.querySelector<HTMLElement>('.sdkwork-model-picker-vendor-list');
  const modelList = menu.querySelector<HTMLElement>('.sdkwork-model-picker-model-list');

  return {
    vendorHeight: vendorList?.scrollHeight ?? 0,
    modelsHeight: modelList?.scrollHeight ?? 0,
  };
}

function isMenuInternalScrollEvent(menu: HTMLElement, event: Event): boolean {
  const target = event.target;
  if (!(target instanceof Node)) {
    return false;
  }
  return menu.contains(target) && target !== menu;
}

export function useModelPickerMenuLayout({
  triggerRef,
  menuRef,
  open,
  preferredPlacement,
  preferredMenuWidth,
  preferredMaxHeight,
  matchTriggerWidth = false,
  layoutKey,
}: {
  triggerRef: RefObject<HTMLElement | null>;
  menuRef: RefObject<HTMLElement | null>;
  open: boolean;
  preferredPlacement: ModelPickerMenuPlacement;
  preferredMenuWidth: number;
  preferredMaxHeight: number;
  matchTriggerWidth?: boolean;
  layoutKey: string;
}): CSSProperties {
  const [menuStyle, setMenuStyle] = useState<CSSProperties>({});

  useLayoutEffect(() => {
    if (!open) {
      setMenuStyle({});
      return undefined;
    }

    const applyLayout = () => {
      const trigger = triggerRef.current;
      const menu = menuRef.current;
      if (!trigger || !menu) {
        return;
      }

      const triggerRect = trigger.getBoundingClientRect();
      const targetMenuWidth = matchTriggerWidth
        ? triggerRect.width
        : preferredMenuWidth;
      const { vendorHeight, modelsHeight } = measurePickerPanels(menu);
      const vendorList = menu.querySelector<HTMLElement>('.sdkwork-model-picker-vendor-list');
      const vendorsPanel = menu.querySelector<HTMLElement>('.sdkwork-model-picker-vendors');
      const vendorHead = vendorsPanel?.querySelector<HTMLElement>('.sdkwork-model-picker-panel-head');
      const vendorPanelHeight = (vendorHead?.offsetHeight ?? 0)
        + (vendorList?.scrollHeight ?? vendorHeight)
        + 12;
      const modelsPanelHeight = modelsHeight + 12;
      const contentMeasure = measureModelPickerMenuContent({
        vendorHeight: vendorPanelHeight,
        modelsHeight: modelsPanelHeight,
        maxPreferredHeight: preferredMaxHeight,
      });

      const layout: ModelPickerMenuLayoutResult = resolveModelPickerMenuLayout({
        triggerRect,
        menuWidth: targetMenuWidth,
        menuHeight: contentMeasure.menuHeight,
        preferredPlacement,
        viewportWidth: window.innerWidth,
        viewportHeight: window.innerHeight,
        gap: MENU_GAP_PX,
        viewportPadding: VIEWPORT_PADDING_PX,
        maxPreferredHeight: preferredMaxHeight,
      });

      vendorsPanel?.classList.toggle('sdkwork-model-picker-vendors--scrollable', contentMeasure.vendorsScrollable);

      setMenuStyle({
        position: 'fixed',
        left: layout.left,
        top: layout.top,
        width: layout.width,
        minWidth: layout.width,
        height: layout.height,
        maxHeight: layout.maxHeight,
        overflow: 'hidden',
        zIndex: MENU_Z_INDEX,
        boxSizing: 'border-box',
      });

      scrollActiveVendorIntoView(menu);
      scrollActiveModelIntoView(menu);
    };

    applyLayout();
    const rafId = window.requestAnimationFrame(applyLayout);

    const handleViewportChange = (event: Event) => {
      const menu = menuRef.current;
      if (!menu) {
        return;
      }
      if (event.type === 'scroll' && isMenuInternalScrollEvent(menu, event)) {
        return;
      }
      applyLayout();
    };

    window.addEventListener('resize', handleViewportChange);
    window.addEventListener('scroll', handleViewportChange, true);
    return () => {
      window.cancelAnimationFrame(rafId);
      window.removeEventListener('resize', handleViewportChange);
      window.removeEventListener('scroll', handleViewportChange, true);
    };
  }, [
    triggerRef,
    menuRef,
    open,
    preferredMaxHeight,
    preferredMenuWidth,
    preferredPlacement,
    matchTriggerWidth,
    layoutKey,
  ]);

  return menuStyle;
}
