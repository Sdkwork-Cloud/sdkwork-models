import type { Model, ModelCreateInput, ModelUpdateInput, VendorCreateInput } from './modelService';

type KnownVendorOption = {
  id: string;
  name: string;
  desc: string;
};

const DEFAULT_VENDOR_COLOR = 'bg-indigo-500';
const DEFAULT_VENDOR_DESCRIPTION = 'Custom model vendor';
const MODEL_TYPES: readonly Model['type'][] = ['Chat', 'Image', 'Audio', 'Embedding', 'Music', 'SoundEffect', 'Video'];
export const MODEL_PRICING_REGIONS = [
  { code: 'cn', currency: 'CNY', labelKey: 'admin.model.modelModal.pricingRegion.cn' },
  { code: 'global', currency: 'USD', labelKey: 'admin.model.modelModal.pricingRegion.global' },
] as const;

type ModelPricingRegionCode = typeof MODEL_PRICING_REGIONS[number]['code'];
type ModelPricingRegion = typeof MODEL_PRICING_REGIONS[number];

export function createVendorInputFromForm(
  formData: FormData,
  vendorSelection: string,
  knownVendors: readonly KnownVendorOption[],
  vendorDescription: string,
): VendorCreateInput | null {
  const selectedVendor = knownVendors.find(vendor => vendor.id === vendorSelection);
  const name = vendorSelection === 'custom'
    ? readFormText(formData, 'customName')
    : selectedVendor?.name.trim() ?? '';

  if (!name) {
    return null;
  }

  return {
    name,
    status: 'active',
    color: DEFAULT_VENDOR_COLOR,
    description: firstNonEmpty(
      vendorDescription,
      readFormText(formData, 'description'),
      selectedVendor?.desc,
      DEFAULT_VENDOR_DESCRIPTION,
    ),
  };
}

export function createModelInputFromForm(formData: FormData, vendorId: string): ModelCreateInput {
  const regionPrices = readRegionPrices(formData);
  return {
    vendorId: vendorId.trim(),
    model: readFormText(formData, 'model'),
    displayName: readOptionalFormText(formData, 'displayName'),
    type: readModelType(formData.get('type')),
    regionPrices,
    contextTokens: readRequiredFormText(formData, 'contextTokens'),
    maxOutputTokens: readOptionalNonNegativeInteger(formData, 'maxOutputTokens'),
    description: readOptionalFormText(formData, 'description'),
    capabilityIntro: readOptionalFormText(formData, 'capabilityIntro'),
    limitations: readCsvFormText(formData, 'limitations'),
    supportedLanguages: readCsvFormText(formData, 'supportedLanguages'),
    useCases: readCsvFormText(formData, 'useCases'),
    supportsStreaming: readFormBoolean(formData, 'supportsStreaming'),
    supportsTools: readFormBoolean(formData, 'supportsTools'),
    supportsJsonSchema: readFormBoolean(formData, 'supportsJsonSchema'),
  };
}

export function updateModelInputFromForm(
  formData: FormData,
  vendorId: string,
  currentModel: Model,
): ModelUpdateInput {
  return {
    ...createModelInputFromForm(formData, vendorId),
    currentType: currentModel.type,
  };
}

function readFormText(formData: FormData, key: string): string {
  const value = formData.get(key);
  return typeof value === 'string' ? value.trim() : '';
}

function readOptionalFormText(formData: FormData, key: string): string | null {
  const normalized = readFormText(formData, key);
  return normalized || null;
}

function readRequiredFormText(formData: FormData, key: string): string {
  const normalized = readFormText(formData, key);
  if (!normalized) {
    throw new Error(`${key} is required`);
  }
  return normalized;
}

function readModelType(value: FormDataEntryValue | null): Model['type'] {
  if (typeof value !== 'string' || !value.trim()) {
    throw new Error('Model type is required');
  }
  const normalized = value.trim();
  if (MODEL_TYPES.includes(normalized as Model['type'])) {
    return normalized as Model['type'];
  }
  throw new Error(`Unsupported model type: ${normalized}`);
}

function readDecimalText(value: FormDataEntryValue | null): string {
  return typeof value === 'string' ? value.trim().replace(/,/g, '') : '';
}

function readOptionalDecimalText(value: FormDataEntryValue | null): string {
  return readDecimalText(value);
}

function readRegionPrices(formData: FormData): ModelCreateInput['regionPrices'] {
  return MODEL_PRICING_REGIONS
    .map((region) => readRegionPrice(formData, region))
    .filter((price): price is NonNullable<ModelCreateInput['regionPrices']>[number] => price !== null);
}

function readRegionPrice(formData: FormData, region: ModelPricingRegion): ModelCreateInput['regionPrices'][number] | null {
  const regionCode: ModelPricingRegionCode = region.code;
  const price = {
    regionCode,
    currency: region.currency,
    priceIn: readDecimalText(formData.get(`priceIn.${regionCode}`)),
    priceOut: readDecimalText(formData.get(`priceOut.${regionCode}`)),
    cacheReadPrice: readOptionalDecimalText(formData.get(`cacheReadPrice.${regionCode}`)),
    cacheWritePrice: readOptionalDecimalText(formData.get(`cacheWritePrice.${regionCode}`)),
  };
  return hasRequiredRegionPrice(price) ? price : null;
}

function hasRequiredRegionPrice(price: ModelCreateInput['regionPrices'][number]): boolean {
  return Boolean(price.priceIn || price.priceOut || price.cacheReadPrice || price.cacheWritePrice);
}

function readOptionalNonNegativeInteger(formData: FormData, key: string): number | null {
  const value = readFormText(formData, key);
  if (!value) {
    return null;
  }
  if (!/^\d+$/.test(value)) {
    throw new Error(`${key} must be a non-negative integer`);
  }
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed)) {
    throw new Error(`${key} must be a non-negative integer`);
  }
  return parsed;
}

function readCsvFormText(formData: FormData, key: string): string[] {
  const value = readFormText(formData, key);
  if (!value) {
    return [];
  }
  return uniqueStrings(value.split(/[\n,]/u).map(item => item.trim()).filter(Boolean));
}

function readFormBoolean(formData: FormData, key: string): boolean {
  const values = formData.getAll(key).filter((value): value is string => typeof value === 'string');
  const value = values.at(-1)?.trim().toLowerCase() ?? '';
  return value === 'true' || value === '1' || value === 'yes' || value === 'on';
}

function uniqueStrings(values: string[]): string[] {
  const seen = new Set<string>();
  const result: string[] = [];
  for (const value of values) {
    const key = value.toLowerCase();
    if (!seen.has(key)) {
      seen.add(key);
      result.push(value);
    }
  }
  return result;
}

function firstNonEmpty(...values: Array<string | undefined>): string {
  for (const value of values) {
    const normalized = value?.trim();
    if (normalized) {
      return normalized;
    }
  }
  return '';
}
