/**
 * Vendor icon SVG assets for sdkwork-models PC pickers.
 *
 * The first group is copied verbatim from the cc-switch icon library
 * (sdkwork-birdcoder/external/cc-switch/src/icons/extracted). The second group
 * covers media-generation vendors cc-switch does not ship: kuaishou / suno /
 * elevenlabs are official brand glyphs from simple-icons (CC0), mureka is the
 * official brand favicon SVG, and runway / luma / vidu / pixverse /
 * blackforestlabs are designed monogram tiles in the vendor brand color.
 * All raw files live in `./assets/` and are imported as raw strings so
 * monochrome (currentColor) icons can be tinted by the surrounding color.
 *
 * The `*.svg?raw` ambient declaration travels with this module so any
 * compilation that includes it (including consumer package tsconfigs) sees
 * the same asset typing.
 */

/// <reference path="./vendor-icons.d.ts" />

import openaiSvg from './assets/openai.svg?raw';
import anthropicSvg from './assets/anthropic.svg?raw';
import geminiSvg from './assets/gemini.svg?raw';
import xaiSvg from './assets/xai.svg?raw';
import alibabaSvg from './assets/alibaba.svg?raw';
import deepseekSvg from './assets/deepseek.svg?raw';
import kimiSvg from './assets/kimi.svg?raw';
import zhipuSvg from './assets/zhipu.svg?raw';
import baiduSvg from './assets/baidu.svg?raw';
import tencentSvg from './assets/tencent.svg?raw';
import bytedanceSvg from './assets/bytedance.svg?raw';
import minimaxSvg from './assets/minimax.svg?raw';
import stepfunSvg from './assets/stepfun.svg?raw';
import stabilitySvg from './assets/stability.svg?raw';
import xiaomimimoSvg from './assets/xiaomimimo.svg?raw';
import longcatSvg from './assets/longcat.svg?raw';
import opencodeSvg from './assets/opencode.svg?raw';
import runwaySvg from './assets/runway.svg?raw';
import lumaSvg from './assets/luma.svg?raw';
import viduSvg from './assets/vidu.svg?raw';
import pixverseSvg from './assets/pixverse.svg?raw';
import kuaishouSvg from './assets/kuaishou.svg?raw';
import blackforestlabsSvg from './assets/blackforestlabs.svg?raw';
import sunoSvg from './assets/suno.svg?raw';
import murekaSvg from './assets/mureka.svg?raw';
import elevenlabsSvg from './assets/elevenlabs.svg?raw';

const VENDOR_ICON_SVGS: Readonly<Record<string, string>> = {
  openai: openaiSvg,
  anthropic: anthropicSvg,
  gemini: geminiSvg,
  xai: xaiSvg,
  alibaba: alibabaSvg,
  deepseek: deepseekSvg,
  kimi: kimiSvg,
  zhipu: zhipuSvg,
  baidu: baiduSvg,
  tencent: tencentSvg,
  bytedance: bytedanceSvg,
  minimax: minimaxSvg,
  stepfun: stepfunSvg,
  stability: stabilitySvg,
  xiaomimimo: xiaomimimoSvg,
  longcat: longcatSvg,
  opencode: opencodeSvg,
  runway: runwaySvg,
  luma: lumaSvg,
  vidu: viduSvg,
  pixverse: pixverseSvg,
  kuaishou: kuaishouSvg,
  blackforestlabs: blackforestlabsSvg,
  suno: sunoSvg,
  mureka: murekaSvg,
  elevenlabs: elevenlabsSvg,
};

/** Raw SVG content for an icon key, or '' when the key is unknown. */
export function getVendorIconSvg(iconKey: string): string {
  return VENDOR_ICON_SVGS[iconKey] ?? '';
}

/** Whether an icon key has copied SVG content in this package. */
export function hasVendorIconSvg(
  iconKey: string | null | undefined,
): boolean {
  return Boolean(iconKey && VENDOR_ICON_SVGS[iconKey]);
}
