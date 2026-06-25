# Changelog

## 2026.06.24.4

- Aligned `vendor-sources.json` supportedModels with on-disk catalog entries for
  Baidu, Google, Moonshot, OpenAI, and Tencent orphan models.
- Added Alibaba Cloud cn `qwen3.7-max` with official CNY pricing and cache rows;
  set it as the cn Qwen default; moved `qwen3.6-max-preview` to supportedModels.

## 2026.06.24.3

- Added Doubao Seed 2.1 (`doubao-seed-2-1-pro-260628`, `doubao-seed-2-1-turbo-260628`)
  from the 2026-06-23 Volcengine Ark release with official CNY pricing and cache rows.
- Set Doubao Seed 2.1 Pro as the cn default model; Seed 2.0 family entries remain listed.

## 2026.06.24.2

- Added StepFun (`stepfun/cn`) with `step-3.5-flash`, `step-3.5-flash-2603`, and
  `step-3.7-flash` from official StepFun pricing docs.
- Expanded ByteDance Doubao (豆包) coverage with `doubao-seed-2-0-code-preview-260215`
  in cn/global and `doubao-seed-2-0-pro-260215` in global.
- Added Zhipu `glm-5.2` as the new GLM default model.
- Marked Anthropic `claude-fable-5` as catalog-only after the 2026-06-12 suspension
  and added `claude-mythos-5` as a hidden catalog-only entry.

## 2026.06.24.1

- Re-verified official pricing for all 18 vendor regions against current vendor
  documentation; confirmed no unit price changes were required for the synced
  release scope.
- Refreshed source evidence timestamps across model, pricing, vendor, and
  official snapshot manifests.
- Regenerated catalog indexes and release metadata for the new catalog version.

## 2026.05.07.1

- Introduced the vendor-scoped `sdkwork-models` catalog layout.
- Added canonical billing meters, model facts, pricing files, ranking snapshots,
  validation tools, freshness policy, release metadata, and SDK entrypoints.
