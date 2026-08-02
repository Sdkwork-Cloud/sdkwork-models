# SDKWork Models PC Model Selector Specs

Local contract index for `@sdkwork/models-pc-picker`.

The package owns reusable PC React model-selection presentation. It exports the
legacy two-column `ModelPicker`, the existing `UnifiedAgentModelSelector`, and
the productized `AgentModelAccessSelector`. The latter presents one unified list
of selectable models and access channels: choosing a model selects it
immediately, while opening an official endpoint or relay station shows a
second-level menu with that channel's supported vendor offerings and models. It
returns the selected model, vendor offering, and access channel together.

Database-backed catalog data and persistence callbacks are injected by the
consumer. Non-empty injected data is authoritative; an empty model catalog uses
the generated mainstream fallback and derives official fallback channels.
Configuration API keys exist only in write-command drafts passed to callbacks.
They are absent from public `ModelAccessChannel` projections. The package does
not construct SDK clients, issue requests, persist credentials, or configure
Agent providers.

Official vendor choices are generated from direct providers in
`overlays/clawrouter/providers.json`. Selecting one fills its sdkwork-models
name, Base URL, vendor identity, current authoritative catalog models, and
default model. Relay stations retain editable names, Base URLs, and multiple
vendor offerings. Offering models use ordered `{ modelId, displayName }` rows;
`modelIds` remains a synchronized compatibility projection for existing SDK
consumers.

## Verification

- `pnpm --filter @sdkwork/models-pc-picker test`
- `pnpm --filter @sdkwork/models-pc-picker typecheck`
- `node ../sdkwork-specs/tools/check-component-port-bindings.mjs --root sdkwork-models`
