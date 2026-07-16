#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HTTP_METHODS = new Set([
  "get",
  "post",
  "put",
  "patch",
  "delete",
  "head",
  "options",
  "trace",
]);

const OFFICIAL_LANGUAGE_ORDER = ["typescript", "rust", "java", "python", "go"];
const DEFAULT_LANGUAGE = "typescript";
const FIXED_SDK_VERSION = "0.1.0";
const STANDARD_PROFILE = "sdkwork-v3";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, "..");
const STANDARD_SDK_GENERATOR_ROOT = path.resolve(workspaceRoot, "../sdkwork-sdk-generator");
const STANDARD_SDK_GENERATOR_BIN = path.join(
  STANDARD_SDK_GENERATOR_ROOT,
  "bin",
  "sdkgen.js",
);

function fail(sdkName, message) {
  process.stderr.write(`[${sdkName}] ${message}\n`);
  process.exit(1);
}

function parseLanguages(raw, sdkName) {
  const values = raw.flatMap((item) => String(item || "").split(","));
  const normalized = [];
  for (const value of values) {
    const language = value.trim().toLowerCase();
    if (!language) {
      continue;
    }
    if (!OFFICIAL_LANGUAGE_ORDER.includes(language)) {
      fail(sdkName, `unsupported language: ${language}`);
    }
    if (!normalized.includes(language)) {
      normalized.push(language);
    }
  }
  return OFFICIAL_LANGUAGE_ORDER.filter((language) => normalized.includes(language));
}

function parseArgs(argv, defaultBaseUrl, sdkName) {
  const result = {
    allLanguages: false,
    languages: [],
    baseUrl: defaultBaseUrl,
    input: null,
    passthrough: [],
  };

  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];
    if (current === "--all-languages") {
      result.allLanguages = true;
      continue;
    }
    if (current === "--language") {
      result.languages.push(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current.startsWith("--language=")) {
      result.languages.push(current.slice("--language=".length));
      continue;
    }
    if (current === "--base-url") {
      result.baseUrl = argv[index + 1] || defaultBaseUrl;
      index += 1;
      continue;
    }
    if (current === "--input") {
      result.input = argv[index + 1] || "";
      index += 1;
      continue;
    }
    if (current.startsWith("--input=")) {
      result.input = current.slice("--input=".length);
      continue;
    }
    if (current === "--") {
      result.passthrough.push(...argv.slice(index + 1));
      break;
    }
    result.passthrough.push(current);
  }

  if (!result.baseUrl || !String(result.baseUrl).trim()) {
    fail(sdkName, "base URL cannot be empty");
  }

  return result;
}

function configuredGeneratorInvocation(configuredPath) {
  if (!configuredPath) {
    return null;
  }
  const trimmed = configuredPath.trim();
  if (!trimmed) {
    return null;
  }
  if (trimmed.endsWith(".js") || trimmed.endsWith(".mjs") || trimmed.endsWith(".cjs")) {
    return {
      command: "node",
      prefixArgs: [trimmed],
      shell: false,
      generatorName: "sdkwork-sdk-generator",
      official: true,
    };
  }
  return {
    command: trimmed,
    prefixArgs: [],
    shell: process.platform === "win32",
    generatorName: "sdkwork-sdk-generator",
    official: true,
  };
}

function isOfficialGeneratorPath(candidate) {
  const normalized = path.resolve(candidate).toLowerCase();
  const standardRoot = path.resolve(STANDARD_SDK_GENERATOR_ROOT).toLowerCase();
  return normalized.startsWith(`${standardRoot}${path.sep}`) || normalized === standardRoot;
}

export function resolveSdkGeneratorInvocation() {
  const configured = configuredGeneratorInvocation(process.env.SDKWORK_SDK_GENERATOR_BIN);
  if (configured) {
    const configuredBin = configured.prefixArgs[0] || configured.command;
    if (!isOfficialGeneratorPath(configuredBin)) {
      throw new Error(
        `SDKWORK_SDK_GENERATOR_BIN must point to ${STANDARD_SDK_GENERATOR_ROOT}; received ${configuredBin}`,
      );
    }
    return configured;
  }

  if (existsSync(STANDARD_SDK_GENERATOR_BIN)) {
    return {
      command: "node",
      prefixArgs: [STANDARD_SDK_GENERATOR_BIN],
      shell: false,
      generatorName: "sdkwork-sdk-generator",
      official: true,
    };
  }

  throw new Error(
    `standard SDK generator not found: ${STANDARD_SDK_GENERATOR_BIN}. Models SDK family generation must use ${STANDARD_SDK_GENERATOR_ROOT}.`,
  );
}

function collectOperations(openapiDocument) {
  const operations = [];
  for (const [pathKey, pathItem] of Object.entries(openapiDocument.paths || {})) {
    if (!pathItem || typeof pathItem !== "object") {
      continue;
    }
    for (const [methodName, operation] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(methodName)) {
        continue;
      }
      operations.push({
        operationId: operation?.operationId
          ? String(operation.operationId)
          : `${methodName}.${pathKey}`,
        method: methodName.toUpperCase(),
        path: pathKey,
      });
    }
  }
  operations.sort((left, right) => left.operationId.localeCompare(right.operationId));
  return operations;
}

function writeTypeScriptComposedOperations(family, manifest, operations) {
  const composedRoot = path.join(
    family.sdkRoot,
    `${family.sdkName}-typescript`,
    "composed",
  );
  const operationLines = operations
    .map(
      (item) =>
        `  "${item.operationId}": { method: "${item.method}", path: "${item.path}" },`,
    )
    .join("\n");
  const source = `// Generated by tools/models_sdk_generator_runner.mjs from the owner-only Models OpenAPI input.
// This file is outside generated/server-openapi so SDK ownership metadata does not pollute sdkgen output.

export const sdkMetadata = {
  name: "${manifest.sdkName}",
  packageName: "${manifest.generatedPackages.typescript.packageName}",
  sdkOwner: "${manifest.sdkOwner}",
  apiAuthority: "${manifest.apiAuthority}",
  language: "typescript",
  standardProfile: "${manifest.standardProfile}",
  baseUrl: "${manifest.baseUrl}",
  apiPrefix: "${manifest.apiPrefix}",
  sdkDependencies: ${JSON.stringify(manifest.sdkDependencies ?? [])},
  dependencyApiExports: ${JSON.stringify(manifest.dependencyApiExports ?? [])},
};

export const operations = {
${operationLines}
} as const;
`;

  mkdirSync(composedRoot, { recursive: true });
  writeFileSync(path.join(composedRoot, "operations.ts"), source, "utf8");
}

function writeModelsFamilyMetadata({
  openapiPath,
  family,
  generatorName,
  baseUrl,
}) {
  const openapiDocument = JSON.parse(readFileSync(openapiPath, "utf8"));
  const operations = collectOperations(openapiDocument);
  const relativeOpenapiPath = toPosixPath(path.relative(family.sdkRoot, openapiPath));
  const generatedPackages =
    operations.length === 0
      ? {}
      : Object.fromEntries(OFFICIAL_LANGUAGE_ORDER.map((language) => [
          language,
          {
            language,
            packageName: `${family.sdkName}-generated-${language}`,
            generatedOutput: `${family.sdkName}-${language}/generated/server-openapi`,
          },
        ]));
  const manifestPath = path.join(family.sdkRoot, "sdk-manifest.json");
  const currentManifest = existsSync(manifestPath)
    ? JSON.parse(readFileSync(manifestPath, "utf8"))
    : {};
  const manifest = {
    ...currentManifest,
    schemaVersion: 1,
    sdkName: family.sdkName,
    sdkOwner: family.sdkOwner,
    apiAuthority: family.apiAuthority,
    sdkFamily: family.sdkName,
    sdkType: family.sdkType,
    apiPrefix: family.apiPrefix,
    generationInputSpec: relativeOpenapiPath,
    generatedPackages,
    sdkDependencies: family.sdkDependencies || [],
    dependencyApiExports: family.dependencyApiExports || [],
    generatorName,
    baseUrl,
    standardProfile: family.manifestStandardProfile,
    fixedSdkVersion: FIXED_SDK_VERSION,
    ownerOnlyOperationCount: operations.length,
    managedBy: "tools/models_sdk_generator_runner.mjs",
  };

  mkdirSync(family.sdkRoot, { recursive: true });
  writeFileSync(
    manifestPath,
    `${JSON.stringify(manifest, null, 2)}\n`,
    "utf8",
  );

  if (operations.length > 0) {
    writeTypeScriptComposedOperations(family, manifest, operations);
  }
}

function removeStaleGeneratedTrackingFiles(outputPath) {
  for (const fileName of ["sdk-manifest.json", "source-openapi.json"]) {
    const filePath = path.join(outputPath, fileName);
    if (existsSync(filePath)) {
      rmSync(filePath, { force: true });
    }
  }
}

function writeGeneratedSourceOpenApi({ openapiPath, outputPath }) {
  const openapiDocument = JSON.parse(readFileSync(openapiPath, "utf8"));
  writeFileSync(
    path.join(outputPath, "source-openapi.json"),
    `${JSON.stringify(openapiDocument, null, 2)}\n`,
    "utf8",
  );
}

function toPosixPath(value) {
  return value.replace(/\\/g, "/");
}

function syncModelsAssemblyMetadata(family, openapiPath) {
  const assemblyPath = path.join(family.sdkRoot, "sdk-manifest.json");
  let assembly = {};
  if (existsSync(assemblyPath)) {
    assembly = JSON.parse(readFileSync(assemblyPath, "utf8"));
  }

  const relativeOpenapiPath = toPosixPath(path.relative(family.sdkRoot, openapiPath));
  assembly.workspace = assembly.workspace || family.sdkName;
  assembly.sdkFamily = family.sdkName;
  assembly.sdkOwner = family.sdkOwner;
  assembly.apiAuthority = family.apiAuthority;
  assembly.authoritySpec = relativeOpenapiPath;
  assembly.generationInputSpec = relativeOpenapiPath;
  assembly.derivedSpecs = {
    ...(assembly.derivedSpecs || {}),
    default: relativeOpenapiPath,
  };
  assembly.discoverySurface = {
    ...(assembly.discoverySurface || {}),
    sdkTarget: family.sdkType,
    apiPrefix: family.apiPrefix,
    generatedProtocols: ["http-openapi"],
    manualTransports: [],
  };
  assembly.sdkDependencies = family.sdkDependencies || [];
  assembly.dependencyApiExports = family.dependencyApiExports || [];

  writeFileSync(assemblyPath, `${JSON.stringify(assembly, null, 2)}\n`, "utf8");
}

export function runModelsSdkGenerator(family, argv) {
  const sdkName = family.sdkName;
  const sdkRoot = family.sdkRoot;
  const args = parseArgs(argv, family.defaultBaseUrl, sdkName);
  const openapiPath = args.input
    ? path.isAbsolute(args.input)
      ? args.input
      : path.resolve(workspaceRoot, args.input)
    : path.join(workspaceRoot, "generated", "openapi", family.defaultOpenapiFile);

  if (!existsSync(openapiPath)) {
    fail(sdkName, `openapi file not found: ${openapiPath}`);
  }
  syncModelsAssemblyMetadata(family, openapiPath);

  const openapiDocument = JSON.parse(readFileSync(openapiPath, "utf8"));
  const operations = collectOperations(openapiDocument);
  if (operations.length === 0) {
    writeModelsFamilyMetadata({
      openapiPath,
      family,
      generatorName: "sdkwork-sdk-generator",
      baseUrl: args.baseUrl,
    });
    process.stdout.write(
      `[${sdkName}] no owner-only operations; synced SDK family metadata without transport generation\n`,
    );
    return;
  }

  const languages = args.allLanguages
    ? OFFICIAL_LANGUAGE_ORDER
    : parseLanguages(args.languages.length > 0 ? args.languages : [DEFAULT_LANGUAGE], sdkName);
  let generator;
  try {
    generator = resolveSdkGeneratorInvocation();
  } catch (error) {
    fail(sdkName, error instanceof Error ? error.message : String(error));
  }

  for (const language of languages) {
    const outputPath = path.join(
      sdkRoot,
      `${sdkName}-${language}`,
      "generated",
      "server-openapi",
    );
    const packageName = `${sdkName}-generated-${language}`;
    const commandArgs = [
      "generate",
      "--input",
      openapiPath,
      "--output",
      outputPath,
      "--name",
      sdkName,
      "--type",
      family.sdkType,
      "--language",
      language,
      "--base-url",
      args.baseUrl,
      "--api-prefix",
      family.apiPrefix,
      "--fixed-sdk-version",
      FIXED_SDK_VERSION,
      "--sdk-root",
      sdkRoot,
      "--sdk-name",
      sdkName,
      "--package-name",
      packageName,
      ...family.standardProfileArgs,
      ...args.passthrough,
    ];

    const result = spawnSync(generator.command, [...generator.prefixArgs, ...commandArgs], {
      cwd: sdkRoot,
      stdio: "inherit",
      shell: generator.shell,
    });

    if (result.error) {
      fail(sdkName, `failed to start generator for ${language}: ${result.error.message}`);
    }
    if (typeof result.status === "number" && result.status !== 0) {
      fail(sdkName, `generator failed for ${language} with exit code ${result.status}`);
    }
    if (result.signal) {
      fail(sdkName, `generator terminated by signal ${result.signal}`);
    }

    removeStaleGeneratedTrackingFiles(outputPath);
    writeGeneratedSourceOpenApi({ openapiPath, outputPath });
  }

  writeModelsFamilyMetadata({
    openapiPath,
    family,
    generatorName: generator.generatorName,
    baseUrl: args.baseUrl,
  });
}

export function resolveFamilySdkRoot(importMetaUrl) {
  return path.resolve(path.dirname(fileURLToPath(importMetaUrl)), "..");
}

export { STANDARD_PROFILE };
