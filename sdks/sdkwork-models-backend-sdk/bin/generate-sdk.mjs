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
        workspace: "sdkwork-appbase-backend-sdk",
        role: "appbase-backend-management-capability",
        required: true,
        dependencyMode: "consumer-sdk",
        apiPrefix: "/backend/v3/api",
        apiAuthority: "sdkwork-appbase-backend-api",
        generatedTransportImportPolicy: "forbidden",
        packageByLanguage: {
          typescript: "@sdkwork/appbase-backend-sdk",
          rust: "sdkwork-appbase-backend-sdk",
          java: "com.sdkwork:sdkwork-appbase-backend-sdk",
          python: "sdkwork-appbase-backend-sdk",
          go: "github.com/sdkwork/sdkwork-appbase-backend-sdk",
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
