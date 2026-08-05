import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import {
  resolveVendorIconKey,
  VENDOR_ICON_BY_CODE,
} from '../src/vendor-icons/vendorIconCatalog.ts';

const vendorsJson = JSON.parse(
  readFileSync(
    fileURLToPath(new URL('../../../../../models/vendors.json', import.meta.url)),
    'utf8',
  ),
);

const assetDir = fileURLToPath(
  new URL('../src/vendor-icons/assets', import.meta.url),
);
const assetNames = new Set(
  readdirSync(assetDir).map((file) => file.replace(/\.svg$/, '')),
);

/**
 * Every vendor in the authoritative models catalog, mapped to its icon key.
 * Media-generation vendors cc-switch does not cover use simple-icons brand
 * glyphs, the official mureka favicon, or designed monogram tiles.
 */
const EXPECTED_ICONED_VENDORS: ReadonlyArray<readonly [string, string]> = [
  ['openai', 'openai'],
  ['anthropic', 'anthropic'],
  ['google', 'gemini'],
  ['xai', 'xai'],
  ['alibaba', 'alibaba'],
  ['deepseek', 'deepseek'],
  ['moonshot', 'kimi'],
  ['zhipu', 'zhipu'],
  ['baidu', 'baidu'],
  ['tencent', 'tencent'],
  ['bytedance', 'bytedance'],
  ['minimax', 'minimax'],
  ['stepfun', 'stepfun'],
  ['stability_ai', 'stability'],
  ['xiaomi', 'xiaomimimo'],
  ['meituan', 'longcat'],
  ['runway', 'runway'],
  ['luma_ai', 'luma'],
  ['vidu', 'vidu'],
  ['pixverse', 'pixverse'],
  ['kuaishou', 'kuaishou'],
  ['black_forest_labs', 'blackforestlabs'],
  ['suno', 'suno'],
  ['mureka', 'mureka'],
  ['elevenlabs', 'elevenlabs'],
];

test('every models catalog vendor has a resolved icon', () => {
  const codes = vendorsJson.vendors.map((vendor: { vendorCode: string }) => vendor.vendorCode);
  assert.ok(codes.length >= 25, 'models catalog vendor count');
  for (const code of codes) {
    const iconKey = resolveVendorIconKey(code);
    assert.ok(iconKey !== undefined, `missing icon resolution for vendor "${code}"`);
    assert.ok(
      EXPECTED_ICONED_VENDORS.some(([vendorCode, key]) => vendorCode === code && key === iconKey),
      `unexpected icon resolution for vendor "${code}" -> "${iconKey}"`,
    );
  }
});

test('known vendors map to the expected cc-switch icon keys', () => {
  for (const [code, iconKey] of EXPECTED_ICONED_VENDORS) {
    assert.equal(resolveVendorIconKey(code), iconKey);
  }
});

test('every mapped icon key has a copied asset file', () => {
  for (const iconKey of new Set(Object.values(VENDOR_ICON_BY_CODE))) {
    assert.ok(assetNames.has(iconKey), `missing copied asset for "${iconKey}"`);
  }
});

test('icon key lookup is case-insensitive and unknown codes resolve to undefined', () => {
  assert.equal(resolveVendorIconKey('DeepSeek'), 'deepseek');
  assert.equal(resolveVendorIconKey('MOONSHOT'), 'kimi');
  assert.equal(resolveVendorIconKey('unknown_vendor_xyz'), undefined);
  assert.equal(resolveVendorIconKey(null), undefined);
  assert.equal(resolveVendorIconKey(undefined), undefined);
});

test('catalog covers consumer brand-code aliases', () => {
  assert.equal(resolveVendorIconKey('opencode'), 'opencode');
  assert.equal(resolveVendorIconKey('kimi'), 'kimi');
  assert.equal(resolveVendorIconKey('mimo'), 'xiaomimimo');
  assert.equal(resolveVendorIconKey('gemini'), 'gemini');
  assert.equal(resolveVendorIconKey('longcat'), 'longcat');
});
