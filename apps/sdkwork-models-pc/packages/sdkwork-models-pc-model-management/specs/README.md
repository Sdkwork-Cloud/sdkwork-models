# SDKWork Models PC Model Management Specs

Local contract index for `@sdkwork/models-pc-model-management`.

The package owns the PC model-management settings center presentation. It is a
split-pane settings surface:

- **Left supplier list** — three groups: the default BirdCoder official
  platform entry (plus any locally configured official channels), relay
  stations, and custom configurations. The BirdCoder entry is selected by
  default.
- **Right configuration panel** — the BirdCoder entry shows the official
  vendor presets read-only (vendor name, Base URL, protocol, models, default
  model). Relay/custom channels show their configuration summary and switch to
  an inline editing form; saving persists through the injected callback only.

All data (official vendor presets, client-local channels, catalog models,
Agent provider options, localized messages) and persistence callbacks are
injected by the consumer. The package performs no data fetching and persists
nothing itself. Channel editing reuses the model-access picker's draft,
validation, and offering-editing building blocks so the interaction stays
identical to the chat surface.

## Verification

- `pnpm --filter @sdkwork/models-pc-model-management typecheck`
- `node ../sdkwork-specs/tools/check-component-port-bindings.mjs --root sdkwork-models`
