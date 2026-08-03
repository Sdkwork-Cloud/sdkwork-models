# SDKWork Models PC Model Management Specs

Local contract index for `@sdkwork/models-pc-model-management`.

The package owns the PC model-management settings center presentation. It is a
split-pane settings surface:

- **Left supplier list** — three groups. The relay-stations group leads with
  the default BirdCoder official relay entry (a single entry) followed by the
  user's relay stations. The official-suppliers group lists only the official
  supplier configurations the user added (never the vendor presets by
  default); the custom-configurations group lists custom channels. The
  BirdCoder entry is selected by default.
- **Right configuration panel** — the BirdCoder entry shows the official
  vendor presets read-only as a grid (vendor name, Base URL, protocol,
  models, default model) as the reference overview. Relay/custom channels
  show their configuration summary. The panel is never replaced by a form and
  uses the full available width.
- **Create/edit dialog** — every group's "+" button opens the shared
  model-access configuration dialog (`ModelAccessChannelConfigurationDialog`
  from `@sdkwork/models-pc-picker`, the same dialog the chat surface uses)
  positioned on that group's kind tab (official/relay/custom). Editing an
  existing channel opens the same dialog with the channel's own kind. Saving
  persists through the injected callback only.

All data (official vendor presets, client-local channels, catalog models,
Agent provider options, localized messages) and persistence callbacks are
injected by the consumer. The package performs no data fetching and persists
nothing itself. Channel editing reuses the model-access picker's draft,
validation, and offering-editing building blocks so the interaction stays
identical to the chat surface.

## Verification

- `pnpm --filter @sdkwork/models-pc-model-management typecheck`
- `node ../sdkwork-specs/tools/check-component-port-bindings.mjs --root sdkwork-models`
