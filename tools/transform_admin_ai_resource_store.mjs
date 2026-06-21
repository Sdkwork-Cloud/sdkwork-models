import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const sqliteHeader = `use std::collections::HashMap;

use sdkwork_models_contract_service::{
    AdminAiResourceGroupItem, AdminAiResourceGroupResourceItem, AdminAiResourceItem,
    AdminAiResourceMemberCommand, AdminAiResourceMemberItem, AdminAiResourceReadFuture,
    AdminAiResourceStore, CreateAdminAiResourceCommand, CreateAdminAiResourceGroupCommand,
    DeleteAdminAiResourceGroupCommand, DomainError, DomainResult,
    ListAdminAiResourceGroupResourcesQuery, ListAdminAiResourceGroupsQuery,
    ListAdminAiResourcesQuery, UpdateAdminAiResourceCommand, UpdateAdminAiResourceGroupCommand,
};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use crate::routing_config_change::{
    record_sqlite_ai_routing_config_change, AiRoutingConfigChange,
};

`;

const postgresHeader = `use std::collections::HashMap;

use sdkwork_models_contract_service::{
    AdminAiResourceGroupItem, AdminAiResourceGroupResourceItem, AdminAiResourceItem,
    AdminAiResourceMemberCommand, AdminAiResourceMemberItem, AdminAiResourceReadFuture,
    AdminAiResourceStore, CreateAdminAiResourceCommand, CreateAdminAiResourceGroupCommand,
    DeleteAdminAiResourceGroupCommand, DomainError, DomainResult,
    ListAdminAiResourceGroupResourcesQuery, ListAdminAiResourceGroupsQuery,
    ListAdminAiResourcesQuery, UpdateAdminAiResourceCommand, UpdateAdminAiResourceGroupCommand,
};
use sqlx::{PgPool, Postgres, Row, Transaction};

use crate::routing_config_change::{
    record_postgres_ai_routing_config_change, AiRoutingConfigChange,
};

`;

function normalize(content) {
  return content.replace(/\r\n/g, "\n");
}

function transformBody(body) {
  return body
    .replaceAll("crate::ports::", "sdkwork_models_contract_service::")
    .replaceAll("crate::domain::", "sdkwork_models_contract_service::");
}

function transformSqlite(content) {
  content = normalize(content);
  const start = content.indexOf("const AI_RESOURCE_TARGET_TYPE");
  const body = transformBody(content.slice(start));
  return sqliteHeader + body;
}

function transformPostgres(content) {
  content = normalize(content);
  const start = content.indexOf("const AI_RESOURCE_TARGET_TYPE");
  let body = transformBody(content.slice(start));
  body = body.replaceAll("SqliteAdminAiResourceStore", "PostgresAdminAiResourceStore");
  body = body.replaceAll("SqlitePool", "PgPool");
  body = body.replaceAll("Transaction<'_, Sqlite>", "Transaction<'_, Postgres>");
  body = body.replaceAll("Transaction<'a, Sqlite>", "Transaction<'a, Postgres>");
  return postgresHeader + body;
}

const sqlitePath = path.join(
  root,
  "crates/sdkwork-models-catalog-repository-sqlx/src/sqlite/admin_ai_resource_store.rs",
);
const postgresPath = path.join(
  root,
  "crates/sdkwork-models-catalog-repository-sqlx/src/postgres/admin_ai_resource_store.rs",
);
const sqliteSource = fs.readFileSync(
  path.join(
    root,
    "crates/sdkwork-models-catalog-repository-sqlx/src/sqlite/admin_ai_resource_store.source.rs",
  ),
  "utf8",
);
const postgresSource = fs.readFileSync(
  path.join(
    root,
    "crates/sdkwork-models-catalog-repository-sqlx/src/postgres/admin_ai_resource_store.source.rs",
  ),
  "utf8",
);

fs.writeFileSync(sqlitePath, transformSqlite(sqliteSource));
fs.writeFileSync(postgresPath, transformPostgres(postgresSource));
console.log("transformed admin ai resource stores");
