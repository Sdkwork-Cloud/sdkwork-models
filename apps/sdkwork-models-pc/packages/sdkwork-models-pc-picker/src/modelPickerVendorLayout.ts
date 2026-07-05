export interface ModelPickerVendorLayoutInput {
  vendorNames: readonly string[];
  variant: 'default' | 'flat';
  menuWidth?: number;
}

function estimateVendorLabelWidth(label: string): number {
  let width = 0;
  for (const char of label) {
    if (/[\u4e00-\u9fff\u3400-\u4dbf\uf900-\ufaff]/.test(char)) {
      width += 11;
      continue;
    }
    if (/\s/.test(char)) {
      width += 4;
      continue;
    }
    width += 6.5;
  }
  return width;
}

export function resolveModelPickerVendorColumnWidth({
  vendorNames,
  variant,
  menuWidth,
}: ModelPickerVendorLayoutInput): number {
  const longestLabelWidth = vendorNames.reduce(
    (max, name) => Math.max(max, estimateVendorLabelWidth(name.trim())),
    0,
  );
  const countGutter = 32;
  const horizontalPadding = 28;
  const min = variant === 'flat' ? 128 : 132;
  const maxByMenu = menuWidth
    ? Math.floor(menuWidth * (variant === 'flat' ? 0.4 : 0.44))
    : variant === 'flat'
      ? 252
      : 216;
  const max = Math.max(min, maxByMenu);

  const naturalWidth = longestLabelWidth + countGutter + horizontalPadding;
  return Math.min(max, Math.max(min, naturalWidth));
}

export function resolveModelPickerMenuWidth({
  vendorColumnWidth,
  variant,
  matchTriggerWidth,
  triggerWidth,
}: {
  vendorColumnWidth: number;
  variant: 'default' | 'flat';
  matchTriggerWidth: boolean;
  triggerWidth?: number;
}): number {
  const modelsMinWidth = variant === 'flat' ? 200 : 260;
  const naturalWidth = vendorColumnWidth + modelsMinWidth;

  if (matchTriggerWidth && triggerWidth) {
    return triggerWidth;
  }

  const preferred = variant === 'flat' ? 460 : 520;
  return Math.max(preferred, naturalWidth);
}

export function resolveModelPickerMenuGridTemplate(vendorColumnWidth: number): string {
  return `${vendorColumnWidth}px minmax(0, 1fr)`;
}
