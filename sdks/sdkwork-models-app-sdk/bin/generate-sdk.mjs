#!/usr/bin/env node
import {
  resolveFamilySdkRoot,
  runModelsSdkGenerator,
} from "../../../tools/models_sdk_generator_runner.mjs";

runModelsSdkGenerator(
  {
    sdkName: "sdkwork-models-app-sdk",
    sdkOwner: "sdkwork-models",
    apiAuthority: "sdkwork-models-app-api",
    sdkDependencies: [
      {
        workspace: "sdkwork-appbase-app-sdk",
        role: "appbase-app-capability",
        required: true,
        dependencyMode: "consumer-sdk",
        apiPrefix: "/app/v3/api",
        apiAuthority: "sdkwork-appbase-app-api",
        generatedTransportImportPolicy: "forbidden",
        packageByLanguage: {
          typescript: "@sdkwork/appbase-app-sdk",
          rust: "sdkwork-appbase-app-sdk",
          java: "com.sdkwork:sdkwork-appbase-app-sdk",
          python: "sdkwork-appbase-app-sdk",
          go: "github.com/sdkwork/sdkwork-appbase-app-sdk",
        },
      },
    ],
    dependencyApiExports: [],
    sdkRoot: resolveFamilySdkRoot(import.meta.url),
    sdkType: "app",
    apiPrefix: "/app/v3/api",
    defaultBaseUrl: "http://127.0.0.1:18080",
    defaultOpenapiFile: "sdkwork-models-app-api.openapi.json",
    standardProfileArgs: ["--standard-profile", "sdkwork-v3"],
    manifestStandardProfile: "sdkwork-v3",
  },
  process.argv.slice(2),
);
