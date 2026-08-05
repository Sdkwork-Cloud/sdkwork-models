/**
 * Vendor icon catalog for sdkwork-models PC pickers.
 *
 * Icon keys and default colors are selected from the cc-switch icon library
 * (sdkwork-birdcoder/external/cc-switch/src/icons/extracted) wherever cc-switch
 * covers the vendor; the key choices follow cc-switch's own provider presets
 * (for example official Gemini uses `gemini`, Moonshot Kimi uses `kimi`).
 * The actual SVG content is loaded by `./vendorIconSvgs.ts` from the copied
 * assets in `./assets/`.
 *
 * Vendors that cc-switch does not cover (media-generation vendors) use icons
 * sourced as follows, documented per file in `./assets/`:
 * - kuaishou / suno / elevenlabs: official brand glyphs from simple-icons (CC0)
 * - mureka: official brand favicon SVG
 * - runway / luma_ai / vidu / pixverse / black_forest_labs: designed monogram
 *   tiles using the vendor's brand color.
 */

/** sdkwork-models vendor codes mapped to icon keys. */
export const VENDOR_ICON_BY_CODE: Readonly<Record<string, string>> = {
  openai: 'openai',
  anthropic: 'anthropic',
  google: 'gemini',
  xai: 'xai',
  alibaba: 'alibaba',
  deepseek: 'deepseek',
  moonshot: 'kimi',
  zhipu: 'zhipu',
  baidu: 'baidu',
  tencent: 'tencent',
  bytedance: 'bytedance',
  minimax: 'minimax',
  stepfun: 'stepfun',
  stability_ai: 'stability',
  xiaomi: 'xiaomimimo',
  meituan: 'longcat',
  runway: 'runway',
  luma_ai: 'luma',
  vidu: 'vidu',
  pixverse: 'pixverse',
  kuaishou: 'kuaishou',
  black_forest_labs: 'blackforestlabs',
  suno: 'suno',
  mureka: 'mureka',
  elevenlabs: 'elevenlabs',
  // Aliases used by picker consumers outside the models catalog. They cover
  // both vendor codes (`opencode`) and model-brand codes that consumers may
  // pass instead of the canonical vendor code (`kimi`, `mimo`, `gemini`).
  opencode: 'opencode',
  kimi: 'kimi',
  mimo: 'xiaomimimo',
  gemini: 'gemini',
  longcat: 'longcat',
};

/**
 * Per-icon default colors copied from cc-switch icon metadata. Icons with a
 * `currentColor` default or fixed fills keep the surrounding text color.
 */
export const VENDOR_ICON_COLORS: Readonly<Record<string, string>> = {
  anthropic: '#D4915D',
  gemini: '#4285F4',
  alibaba: '#FF6A00',
  deepseek: '#1E88E5',
  kimi: '#1783FF',
  zhipu: '#0F62FE',
  baidu: '#2932E1',
  tencent: '#00A4FF',
  minimax: '#FF6B6B',
  stepfun: '#005AFF',
  xiaomimimo: '#000000',
  longcat: '#29E154',
};

/**
 * Resolve a vendor code to its cc-switch icon key. The lookup is
 * case-insensitive; unknown codes return `undefined` (initials fallback).
 */
export function resolveVendorIconKey(
  vendorCode: string | null | undefined,
): string | undefined {
  if (!vendorCode) {
    return undefined;
  }
  return VENDOR_ICON_BY_CODE[vendorCode.toLowerCase()];
}

/** Default display color for an icon key, if cc-switch declares one. */
export function getVendorIconColor(iconKey: string): string | undefined {
  return VENDOR_ICON_COLORS[iconKey];
}
