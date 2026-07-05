export type ModelsPickerBucket = 'llms' | 'images' | 'videos' | 'audios' | 'music' | 'sfx';

export interface ModelsPickerVendor {
  code: string;
  name: string;
}

export interface ModelsPickerReferencePrice {
  regionCode: string;
  billingMeter: string;
  unitPrice: string;
  currency: string;
}

export interface ModelsPickerPriceAvailability {
  status: 'reference' | 'unavailable';
  reason?: string | null;
}

export interface ModelsPickerOption {
  id: string;
  catalogKey: string;
  model: string;
  name: string;
  displayName: string;
  desc: string;
  description?: string;
  ver: string;
  versionLabel: string;
  vendorCode: string;
  vendorName: string;
  modalities: string[];
  inputModalities: string[];
  outputModalities: string[];
  capabilities: string[];
  apiFormat?: string;
  contextTokens?: number;
  maxOutputTokens?: number;
  officialReferencePrices: ModelsPickerReferencePrice[];
  priceAvailability: ModelsPickerPriceAvailability;
  providerCodes: string[];
  supportsStreaming: boolean;
  supportsTools: boolean;
  supportsJsonSchema: boolean;
}

export interface ModelsPickerGroup {
  id: string;
  vendor: ModelsPickerVendor;
  llms: ModelsPickerOption[];
  images: ModelsPickerOption[];
  videos: ModelsPickerOption[];
  audios: ModelsPickerOption[];
  music: ModelsPickerOption[];
  sfx: ModelsPickerOption[];
}
