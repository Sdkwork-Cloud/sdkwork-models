export type ModelPickerMenuPlacement = 'auto' | 'top' | 'bottom';

export interface ModelPickerMenuLayoutInput {
  triggerRect: Pick<DOMRectReadOnly, 'top' | 'right' | 'bottom' | 'left' | 'width'>;
  menuWidth: number;
  menuHeight: number;
  preferredPlacement: ModelPickerMenuPlacement;
  viewportWidth: number;
  viewportHeight: number;
  gap?: number;
  viewportPadding?: number;
  maxPreferredHeight: number;
}

export interface ModelPickerMenuLayoutResult {
  placement: 'top' | 'bottom';
  left: number;
  top: number;
  width: number;
  height: number;
  maxHeight: number;
}

export interface ModelPickerContentMeasureInput {
  vendorHeight: number;
  modelsHeight: number;
  maxPreferredHeight: number;
  minHeight?: number;
  modelsPadding?: number;
}

export interface ModelPickerContentMeasureResult {
  menuHeight: number;
  vendorsScrollable: boolean;
  modelsScrollable: boolean;
}

export function measureModelPickerMenuContent({
  vendorHeight,
  modelsHeight,
  maxPreferredHeight,
  minHeight = 160,
  modelsPadding = 12,
}: ModelPickerContentMeasureInput): ModelPickerContentMeasureResult {
  const modelsBlockHeight = modelsHeight + modelsPadding;
  const naturalHeight = Math.max(vendorHeight, modelsBlockHeight);
  const menuHeight = Math.max(minHeight, Math.min(naturalHeight, maxPreferredHeight));

  return {
    menuHeight,
    vendorsScrollable: vendorHeight > menuHeight,
    modelsScrollable: modelsBlockHeight > menuHeight,
  };
}

export function resolveModelPickerMenuLayout({
  triggerRect,
  menuWidth,
  menuHeight,
  preferredPlacement,
  viewportWidth,
  viewportHeight,
  gap = 8,
  viewportPadding = 16,
  maxPreferredHeight,
}: ModelPickerMenuLayoutInput): ModelPickerMenuLayoutResult {
  const spaceBelow = viewportHeight - triggerRect.bottom - viewportPadding;
  const spaceAbove = triggerRect.top - viewportPadding;
  const width = Math.min(menuWidth, viewportWidth - viewportPadding * 2);

  let placement: 'top' | 'bottom';
  if (preferredPlacement === 'auto') {
    const minComfortableHeight = Math.min(maxPreferredHeight, 240);
    if (spaceBelow >= minComfortableHeight || spaceBelow >= spaceAbove) {
      placement = 'bottom';
    } else {
      placement = 'top';
    }
  } else {
    placement = preferredPlacement;
  }

  const availableHeight = placement === 'bottom' ? spaceBelow - gap : spaceAbove - gap;
  const cappedHeight = Math.max(160, Math.min(maxPreferredHeight, availableHeight));
  const height = Math.min(menuHeight, cappedHeight);
  const maxHeight = cappedHeight;

  let top = placement === 'bottom'
    ? triggerRect.bottom + gap
    : triggerRect.top - gap - height;
  top = Math.max(viewportPadding, Math.min(top, viewportHeight - viewportPadding - height));

  let left = triggerRect.left;
  if (left + width > viewportWidth - viewportPadding) {
    left = triggerRect.right - width;
  }
  if (left < viewportPadding) {
    left = viewportPadding;
  }
  if (left + width > viewportWidth - viewportPadding) {
    left = Math.max(viewportPadding, viewportWidth - viewportPadding - width);
  }

  return {
    placement,
    left,
    top,
    width,
    height,
    maxHeight,
  };
}
