import type { ModelsPickerBucket, ModelsPickerGroup, ModelsPickerOption } from './model-picker-types';

export interface ModelPickerListItem {
  group: ModelsPickerGroup;
  model: ModelsPickerOption;
}

export function normalizeModelPickerQuery(query: string): string {
  return query.trim().toLowerCase();
}

export function modelMatchesPickerQuery(model: ModelsPickerOption, query: string): boolean {
  const normalized = normalizeModelPickerQuery(query);
  if (!normalized) {
    return true;
  }

  const haystack = [
    model.name,
    model.displayName,
    model.model,
    model.desc,
    model.description,
    model.vendorName,
    model.versionLabel,
    model.ver,
  ]
    .filter(Boolean)
    .join(' ')
    .toLowerCase();

  return haystack.includes(normalized);
}

export function listModelPickerItems(
  groups: ModelsPickerGroup[],
  bucket: ModelsPickerBucket,
  activeVendorCode: string | null,
  query: string,
): ModelPickerListItem[] {
  const normalized = normalizeModelPickerQuery(query);
  const scopedGroups = normalized
    ? groups
    : groups.filter((group) => !activeVendorCode || group.vendor.code === activeVendorCode);

  return scopedGroups.flatMap((group) =>
    group[bucket]
      .filter((model) => modelMatchesPickerQuery(model, query))
      .map((model) => ({ group, model })),
  );
}
