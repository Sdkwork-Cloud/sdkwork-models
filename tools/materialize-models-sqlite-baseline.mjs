#!/usr/bin/env node
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const postgresPath = join(
  root,
  "database/ddl/baseline/postgres/0001_sdkwork-models_baseline.sql",
);
const sqlitePath = join(
  root,
  "database/ddl/baseline/sqlite/0001_sdkwork-models_baseline.sql",
);

const postgres = readFileSync(postgresPath, "utf8");
const sqlite = postgres
  .replace(/^-- Owned by data\/sdkwork-models$/m, "-- Owned by sdkwork-models (sqlite mirror)")
  .replace(/\bTIMESTAMPTZ\b/g, "TEXT")
  .replace(/\bJSONB\b/g, "TEXT")
  .replace(/DEFAULT CURRENT_TIMESTAMP/g, "DEFAULT (datetime('now'))")
  .replace(/DEFAULT '{}'::jsonb/g, "DEFAULT '{}'");

mkdirSync(dirname(sqlitePath), { recursive: true });
writeFileSync(
  sqlitePath,
  `-- SDKWork Models catalog module baseline (sqlite)\n-- Generated from postgres baseline by tools/materialize-models-sqlite-baseline.mjs\n\n${sqlite.trim()}\n`,
  "utf8",
);
console.log(`materialized ${sqlitePath}`);
