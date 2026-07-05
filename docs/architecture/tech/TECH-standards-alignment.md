# SDKWork Standards Alignment (Architecture Copy)

Owner: SDKWork maintainers  
Updated: 2026-07-05

**Canonical verified matrix:** [docs/standards-alignment.md](../../standards-alignment.md) — that document is the single source of truth for production-readiness posture and must match `pnpm run verify` evidence.

This architecture copy summarizes framework integration for readers of `TECH_ARCHITECTURE.md` only. Do not maintain divergent status tables here.

## Repository Identity

| Item | Value |
| --- | --- |
| Application key | `sdkwork-models` |
| Component | `@sdkwork/models-catalog` |
| Domain / capability | `intelligence` / `catalog` |
| Catalog version | See `sdkwork-models.json` (currently `2026.07.05.3`) |

## Framework Integration

| Framework | Status | Notes |
| --- | --- | --- |
| `sdkwork-specs` | Aligned | Root standards + local `specs/` |
| `sdkwork-utils` | Aligned | `SdkWorkPageData`, `SdkWorkApiResponse`, `@sdkwork/utils` |
| `sdkwork-web-framework` | Aligned | Route manifests, IAM adapter, envelope handlers |
| `sdkwork-database` | Aligned | Baseline DDL includes voice + video profile tables; empty migrations at init |
| `sdkwork-sdk-generator` | Aligned | OpenAPI export + SDK generation |
| `sdkwork-iam-web-adapter` | Aligned | Backend routes require declared permissions |

## TTS Voice Catalog

Contract: `specs/voice-catalog.spec.json`

- Storage: `models/{vendor}/{region}/voices.json`, `model-voices/{modelId}.json`
- SDK helpers: `listVoices`, `listVoicesForModel`, `listModelsForVoice` (all language SDKs)
- App API: public `voices.list`, `modelVoices.list`
- Backend API: `intelligence.models.read` on voice list routes
- Envelope: `SdkWorkPageData` list (`data.items` + `data.pageInfo`)

## Video Generation Profiles

Contract: `specs/video-generation-profile.spec.json`

- Storage: `models/{vendor}/{region}/model-video-profiles/{modelId}.json`
- SDK helpers: `listVideoProfiles`, `listVideoProfilesForModel`, `findVideoProfile` (all language SDKs)
- App API: public `videoProfiles.list`, `modelVideoProfiles.list`
- Backend API: `intelligence.models.read` on video profile list routes
- Database: `ai_model_video_profile` populated during catalog sync import
- Validation: every `primaryCapability: video` model requires a profile file

## Verification

```powershell
pnpm run verify
```

See [docs/standards-alignment.md](../../standards-alignment.md) for the full alignment matrix and composed-host integration notes.
