## 目标

sdkwork-models 目前**没有任何 icon 资源**：`ModelPicker` 的 vendor 列只显示文字；`UnifiedAgentModelSelector` 全部模型回退到 Bot 图标；`AgentModelAccessSelector` 未传 `renderModelIcon` 时图标槽为空。方案：从 sdkwork-birdcoder `external/cc-switch` 选取并拷贝图标到 sdkwork-models，新增 vendor 图标模块，修复所有组件默认图标显示。

## 现状（已核实）

- cc-switch 图标目录：`sdkwork-birdcoder/external/cc-switch/src/icons/extracted/`（SVG 文件 = `index.ts` 内联 SVG 内容一致），含 `metadata.ts` 默认色。cc-switch 自身 preset 的选图惯例：官方 Anthropic→`anthropic`、OpenAI→`openai`、Gemini→`gemini`、xAI→`xai`、Kimi→`kimi`、DeepSeek→`deepseek`、智谱→`zhipu`、百度→`baidu`、火山→`huoshan/byteplus/doubao`、混元→`hunyuan`、阶跃→`stepfun`、MiniMax→`minimax`、小米 MiMo→`xiaomimimo`。
- sdkwork-models 25 个 vendor（`models/vendors.json`），其中 15 个可在 cc-switch 找到对应图标：openai、anthropic、google、xai、alibaba、deepseek、moonshot、zhipu、baidu、tencent、bytedance、minimax、stepfun、stability_ai、xiaomi；其余 10 个（runway、luma_ai、vidu、pixverse、kuaishou、meituan、black_forest_labs、suno、mureka、elevenlabs）cc-switch 无图标 → 首字母回退（与 cc-switch ProviderIcon 行为一致）。
- 3 个 icon 消费组件均在 `apps/sdkwork-models-pc/packages/sdkwork-models-pc-picker/`；消费方（birdcoder）通过 Vite alias 直接引源码。

## 实施步骤

### 1. 拷贝图标到 sdkwork-models（16 个 SVG 文件）
从 `sdkwork-birdcoder/external/cc-switch/src/icons/extracted/` 拷贝以下文件到 `apps/sdkwork-models-pc/packages/sdkwork-models-pc-picker/src/vendor-icons/assets/`：
`openai.svg` `anthropic.svg` `gemini.svg` `xai.svg` `alibaba.svg` `deepseek.svg` `kimi.svg` `zhipu.svg` `baidu.svg` `tencent.svg` `bytedance.svg` `minimax.svg` `stepfun.svg` `stability.svg` `xiaomimimo.svg` `opencode.svg`（opencode 供 birdcoder workbench ModelPicker 的 `opencode` vendor code 使用）

### 2. 新增 `src/vendor-icons/` 模块（picker 包内，组件自持）
- `vendorIconCatalog.ts`：vendor code → cc-switch icon key 映射（按上表 + cc-switch 惯例）、图标默认色（取自 cc-switch metadata）；`resolveVendorIconKey()` / `hasVendorIcon()` / `getVendorIconSvg()`（`import ... from './assets/*.svg?raw'`）/ `getVendorIconColor()`。
- `VendorIcon.tsx`：复用组件 `VendorIcon`，props：`iconKey?` `vendorCode?` `name` `size?` `className?` `showFallback?`；内联 SVG 渲染（`?raw` 字符串 + `dangerouslySetInnerHTML`，`currentColor` 图标按默认色着色，与 cc-switch ProviderIcon 一致）；未命中时首字母徽标回退（cc-switch 同款样式），`showFallback=false` 时返回 null。
- `vendor-icons.css`：图标元素/回退徽标样式 + ModelPicker vendor 行图标布局（label 包裹器 `display:flex`，保持消费方现有 2 列 grid 不破版）。
- `styles.d.ts` 增加 `declare module '*.svg?raw'`。
- `src/index.ts` 导出 `VendorIcon`、`resolveVendorIconKey` 等。
- 新增 `tests/vendorIconCatalog.test.ts`：读取 `models/vendors.json` 断言每个 vendor 可确定性解析（映射内 key 均有 SVG 内容，未映射 vendor 回退 null）。

### 3. 修复组件 icon 显示
- `model-picker-types.ts`：`ModelsPickerVendor`/`ModelsPickerOption` 增加可选 `icon?: string`（显式 icon key，消费方可覆盖）。
- `ModelPicker.tsx`：vendor 列按钮加 vendor 图标（16px，`group.vendor.icon ?? group.vendor.code` 解析）；模型行加 vendor 图标（16px，`model.icon ?? model.vendorCode` 解析）。图标+名称包在 `.sdkwork-model-picker-vendor-label` flex 容器内，不破坏消费方现有 `grid-template-columns: minmax(0,1fr) auto` 布局。
- `UnifiedAgentModelSelector.tsx`：默认图标改为按 `option.iconKey`/`option.vendorCode` 解析的 `VendorIcon`；仅未解析的 custom 模型保留 Bot 兜底。
- `AgentModelAccessSelector.tsx`：默认模型图标 = `renderModelIcon?.(model) ?? VendorIcon(iconKey/vendorCode)`（`showFallback=false`，未知名 vendor 继续走 `data-no-icon` 折叠，保留消费方 opt-out 语义）。

### 4. 验证
- `pnpm --filter @sdkwork/models-pc-picker test`
- `pnpm --filter @sdkwork/models-pc-picker typecheck`
- `pnpm run check`（仓库校验，如可行）

## 说明
- 映射放客户端组件包内（picker 自持），不改 `models/vendors.json` / 数据库 / API / 生成 SDK（避免数据库迁移与生成物变更，符合改动范围约束）。
- 不触碰生成目录；不新建包；无新建依赖。