# SDKWork Models Backend SDK Specs

`component.spec.json` is the machine-readable SDK family contract for `@sdkwork/models-backend-sdk`.

The authored OpenAPI authority and generator workflow own generated transport code. Consumers use the composed package export and typed resource methods; generated output under `generated/server-openapi` is never edited manually.

Resource and resource-group list methods return one typed `{ items, pageInfo }` page. Resource-group membership uses dedicated single-resource update and delete methods so interactive clients do not aggregate or replace entire membership collections.

