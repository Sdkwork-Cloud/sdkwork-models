# SDKWork Models PC Model Selector Specs

Local contract index for `@sdkwork/models-pc-picker`.

The package owns reusable PC React model-selection presentation. It exports the
legacy two-column `ModelPicker` and a separate single-list
`CompactModelSelector` with an injected
custom-model creation port. Custom model drafts are provider-independent model
records; the consuming runtime resolves provider bindings only after selection.
The package does not own application persistence, provider credentials, SDK
construction, or runtime model registration.

## Verification

- `pnpm --filter @sdkwork/models-pc-picker typecheck`
- `node ../sdkwork-specs/tools/check-component-port-bindings.mjs --root sdkwork-models`
