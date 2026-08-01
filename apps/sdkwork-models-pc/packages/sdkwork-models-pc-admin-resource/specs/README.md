# SDKWork Models Resource Administration Specs

`component.spec.json` is the machine-readable contract for the Models PC backend-admin resource feature.

The package owns resource-group administration UI and its thin generated backend SDK service facade. Interactive groups, group members, and assignable resources are server-paginated. Existing group metadata updates never replace membership collections; individual member additions and removals use the generated member update and delete operations.

Root SDKWork standards remain authoritative. The relevant authorities are linked from `component.spec.json`.

