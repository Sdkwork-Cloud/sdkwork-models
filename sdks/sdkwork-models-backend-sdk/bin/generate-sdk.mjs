#!/usr/bin/env node
import {
  resolveFamilySdkRoot,
  runModelsSdkGenerator,
} from "../../../tools/models_sdk_generator_runner.mjs";

runModelsSdkGenerator(
  {
    sdkName: "sdkwork-models-backend-sdk",
    sdkOwner: "sdkwork-models",
    apiAuthority: "sdkwork-models-backend-api",
    sdkDependencies: [
      {
        workspace: "sdkwork-iam-backend-sdk",
        role: "appbase-backend-management-capability",
        required: true,
        dependencyMode: "consumer-sdk",
        apiPrefix: "/backend/v3/api",
        apiAuthority: "sdkwork-iam-backend-api",
        generatedTransportImportPolicy: "forbidden",
        packageByLanguage: {
          typescript: "@sdkwork/iam-backend-sdk",
          rust: "sdkwork-iam-backend-sdk",
          java: "com.sdkwork:sdkwork-iam-backend-sdk",
          python: "sdkwork-iam-backend-sdk",
          go: "github.com/sdkwork/sdkwork-iam-backend-sdk",
        },
      },
    ],
    dependencyApiExports: [],
    sdkRoot: resolveFamilySdkRoot(import.meta.url),
    sdkType: "backend",
    apiPrefix: "/backend/v3/api",
    defaultBaseUrl: "http://127.0.0.1:18080",
    defaultOpenapiFile: "sdkwork-models-backend-api.openapi.json",
    standardProfileArgs: ["--standard-profile", "sdkwork-v3"],
    manifestStandardProfile: "sdkwork-v3",
  },
  process.argv.slice(2),
);
