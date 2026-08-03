# SDKWork Models 目录同步计划（2026-08-03）

## 背景
模型目录最后同步于 2026-07-26（catalogVersion 2026.07.26.1）。经 4 路调研代理核查 25 家 vendor 官方模型状态（OpenAI/Anthropic/Google/DeepSeek/xAI/中国厂商/多模态厂商），发现：目录状态错误、官方停服未标记、新模型缺失、sources 登记缺失等问题。

## 一、状态修正（目录与官方冲突，约 4-5 个文件）

| 模型文件 | 当前状态 | 官方状态 | 动作 |
|---|---|---|---|
| `google/global/models/gemini-3.5-flash.json` | deprecated/hidden/catalog_only | **active**（GA 5/19，无弃用日期） | → active/listed/enabled |
| `google/global/models/gemini-3.1-flash-lite.json` | deprecated/hidden/catalog_only | **active**（最早关停 2027-05） | → active/listed/enabled |
| `google/global/models/gemini-3.1-pro.json` | active | 官方仍为 preview（无 GA ID） | → preview（保留文件） |
| `anthropic/global/models/claude-fable-5.json` | catalog_only/hidden | **active**（7/1 恢复全球部署） | → active/listed/enabled，更新 description |
| `anthropic/global/models/claude-mythos-5.json` | catalog_only/hidden | 仅限 Project Glasswing 获批组织（受限但仍活跃） | → preview/listed（受限注明） |

## 二、标记弃用（官方已停服/弃用，保留文件，约 8 个文件）
目录惯例（参照 kimi-k2.5）：`lifecycle=deprecated`、`shelfState=hidden`、`routingState=catalog_only`、`releaseStage=deprecated`，从 requiredModels 移到 supportedModels，保留 pricing 文件：

- `tencent/cn/models/hunyuan-turbos-latest.json`（2026-06-26 停服，替代 hy3）
- `tencent/cn/models/hunyuan-2.0-instruct-20251111.json`（同上）
- `tencent/cn/models/hunyuan-2.0-thinking-20251109.json`（同上）
- `xiaomi/cn/models/mimo-v2-flash.json` + `xiaomi/global/models/mimo-v2-flash.json`（6/30 下线）
- `luma_ai/global/models/ray-2.json` + `ray-flash-2.json`（官方明确 deprecated）
- Google 即将关停（官方给出明确日期）：`imagen-4.0-generate-001`（8/17）、`imagen-4.0-fast-generate-001`（8/17）、`imagen-4.0-ultra-generate-001`（8/17）、`gemini-2.5-flash-image`（10/2）→ 标记 deprecated（imagen-4.0 三兄弟从 required 移 supported）

## 三、新增模型（高置信，3 个模型 + 对应 pricing 文件）
按已确认目录格式创建 `models/<vendor>/<region>/models/<modelId>.json` + `pricing/<modelId>.json`（含 rankScore/trendScore/capabilities 等全字段，observedAt=2026-08-03）：

1. **MiniMax H3**（`minimax/global`，视频模型，2026-07-31 发布，参照 hailuo-2.3 模板，familyCode=minimax-video）— 执行时搜索官方定价
2. **FLUX 3**（`black_forest_labs/global`，2026-07-23 发布，多模态 early access）— 标 preview + catalog_only（无公开 API ID），如无公开定价则 pricing 可仅含 api_request 或按目录惯例
3. **Google nano-banana-2 系列**（imagen-4.0 官方替代，模型 ID `nano-banana-2`/`nano-banana-2-lite`/`nano-banana-pro` 待执行时以官方文档为准核实 ID 与定价）— 若执行时无法确认 ID/定价，则仅补已有 ID 的部分

同时补充 sources 未登记但目录已存在的模型到 `sources/vendor-sources.json`：
- alibaba/cn: `qwen3.8-max-preview`（→ supported）
- anthropic/global: `claude-opus-5`（→ required）
- bytedance/cn: `doubao-seedance-2-5-260623`（→ required）
- google/global: `gemini-3.5-flash-lite`、`gemini-3.6-flash`（→ required）
- moonshot/cn+global: `kimi-k3`（→ required）

## 四、sources 文件更新
- `sources/vendor-sources.json`：catalogVersion→2026.08.03.1、observedAt→2026-08-03、各 vendor `lastCheckedAt`→2026-08-03、required/supported 按上述调整
- `sources/official-model-snapshots.json`：catalogVersion、observedAt、各 vendor/region observedAt→2026-08-03，模型列表同步（新增 H3/FLUX 3/nano-banana-2；移除已退役），sourceSnapshotHash 保持原值（无法重新抓取官方页面）或按新模型列表更新
- `sdkwork-models.json`：catalogVersion → `2026.08.03.1`、generatedAt → 2026-08-03

## 五、重建与生成
1. `node tools/sync-catalog.mjs`（对齐 capabilities/定价、校验 requiredModels）
2. `node tools/build-index.mjs`（重建 models/index.json + vendors.json）
3. `node tools/generate-mainstream-agent-model-catalog.mjs`（重新生成 PC 端 UI 目录 `mainstreamAgentModelCatalog.generated.ts` + `officialModelVendorPresets.generated.ts`）
4. 如需 SDK 重新生成则运行 `tools/models-sdk-generate.mjs`（视验证结果决定）

## 六、验证
- `node tools/validate-catalog.mjs`
- `node tools/freshness-report.mjs`（须无 error 级报警）
- `node tools/catalog-audit.mjs`
- `node tools/catalog-diff.mjs`（对比变更摘要）
- 如涉及 SDK/前端：`node <sdkwork-specs>/tools/check-app-sdk-consumer-imports.mjs --workspace <root>`、`pnpm run check`

## 备注
- 工作区已有未提交的 PC 应用修改（ModelManagementSettingsCenter 等），不属于本次任务，不触碰
- 执行期间会为 MiniMax H3 / nano-banana-2 / FLUX 3 补充官方定价搜索（WebSearch），若定价无法确认则标记 catalog_only 并注明
- 不在本次范围的调研发现（次要新模型：qwen3.7-plus/flash、mimo-v2.5-tts、grok-voice-2.0、uni-1.1、Aleph 2.0、pixverse-r1 等）按用户选择不收录，但会在最终报告中列出供后续参考