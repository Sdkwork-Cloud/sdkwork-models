use std::collections::BTreeMap;

use sdkwork_models::ModelCatalog;
use sqlx::{PgConnection, PgPool, Postgres, Row, Transaction};

use crate::model_catalog_import::*;

pub async fn import_postgres_model_catalog(
    pool: &PgPool,
    catalog: &ModelCatalog,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    import_postgres_model_catalog_tx(&mut tx, catalog).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn import_postgres_model_catalog_tx(
    tx: &mut Transaction<'_, Postgres>,
    catalog: &ModelCatalog,
) -> Result<(), sqlx::Error> {
    import_postgres_model_catalog_connection(&mut **tx, catalog).await
}

async fn import_postgres_model_catalog_connection(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
) -> Result<(), sqlx::Error> {
    deactivate_removed_catalog_rows(conn, catalog).await?;
    import_meters(conn, catalog).await?;
    let vendor_ids = import_vendors(conn, catalog).await?;
    let family_ids = import_families(conn, catalog, &vendor_ids).await?;
    let model_ids = import_models(conn, catalog, &vendor_ids, &family_ids).await?;
    import_capabilities(conn, catalog, &model_ids).await?;
    let modality_ids = import_modalities(conn, catalog).await?;
    let endpoint_ids = import_api_endpoints(conn, catalog).await?;
    import_vendor_modalities(conn, catalog, &vendor_ids, &modality_ids).await?;
    import_vendor_api_endpoints(conn, catalog, &vendor_ids, &endpoint_ids).await?;
    import_modality_api_endpoints(conn, catalog, &modality_ids, &endpoint_ids).await?;
    import_model_modalities(conn, catalog, &model_ids, &modality_ids).await?;
    import_model_api_endpoints(conn, catalog, &model_ids, &endpoint_ids).await?;
    import_ai_resources(
        conn,
        catalog,
        &vendor_ids,
        &model_ids,
        &modality_ids,
        &endpoint_ids,
    )
    .await?;
    import_pricing(conn, catalog, &model_ids).await?;
    import_rankings(conn, catalog, &model_ids).await?;
    update_family_defaults(conn, catalog, &model_ids).await?;
    Ok(())
}

async fn deactivate_removed_catalog_rows(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
) -> Result<(), sqlx::Error> {
    let keys = catalog_authority_keys(catalog);
    if keys.vendor_codes.is_empty() {
        return Ok(());
    }

    deactivate_postgres_rows_not_in(
        conn,
        "ai_model_rank_snapshot",
        &keys.vendor_codes,
        "uuid",
        &keys.ranking_uuids,
    )
    .await?;
    deactivate_postgres_rows_not_in(
        conn,
        "ai_model_pricing",
        &keys.vendor_codes,
        "uuid",
        &keys.price_uuids,
    )
    .await?;
    deactivate_postgres_rows_not_in(
        conn,
        "ai_resource",
        &keys.vendor_codes,
        "resource_code",
        &keys.ai_resource_codes,
    )
    .await?;
    deactivate_postgres_rows_not_in(
        conn,
        "ai_model_api_endpoint",
        &keys.vendor_codes,
        "uuid",
        &keys.model_api_endpoint_uuids,
    )
    .await?;
    deactivate_postgres_rows_not_in(
        conn,
        "ai_model_modality",
        &keys.vendor_codes,
        "uuid",
        &keys.model_modality_uuids,
    )
    .await?;
    deactivate_postgres_rows_not_in(
        conn,
        "ai_vendor_api_endpoint",
        &keys.vendor_codes,
        "uuid",
        &keys.vendor_api_endpoint_uuids,
    )
    .await?;
    deactivate_postgres_rows_not_in(
        conn,
        "ai_vendor_modality",
        &keys.vendor_codes,
        "uuid",
        &keys.vendor_modality_uuids,
    )
    .await?;
    deactivate_postgres_rows_not_in(
        conn,
        "ai_model_capability",
        &keys.vendor_codes,
        "uuid",
        &keys.capability_uuids,
    )
    .await?;
    deactivate_postgres_rows_not_in(
        conn,
        "ai_model_family",
        &keys.vendor_codes,
        "uuid",
        &keys.family_uuids,
    )
    .await?;
    deactivate_postgres_rows_not_in(
        conn,
        "ai_model",
        &keys.vendor_codes,
        "catalog_key",
        &keys.catalog_keys,
    )
    .await?;
    Ok(())
}

async fn deactivate_postgres_rows_not_in(
    conn: &mut PgConnection,
    table_name: &str,
    vendor_codes: &[String],
    key_column: &str,
    active_keys: &[String],
) -> Result<(), sqlx::Error> {
    let sql = format!(
        "UPDATE {table_name} SET status = 0, updated_at = CURRENT_TIMESTAMP WHERE tenant_id = 0 AND organization_id = 0 AND vendor_code = ANY($1) AND status = 1 AND ($2::text[] = '{{}}' OR NOT ({key_column} = ANY($2)))"
    );
    sqlx::query(sql.as_str())
        .bind(vendor_codes)
        .bind(active_keys)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

async fn import_meters(conn: &mut PgConnection, catalog: &ModelCatalog) -> Result<(), sqlx::Error> {
    for meter in &catalog.meters {
        let row_id = stable_catalog_id("sdk-meter", &[&meter.meter_code]);
        sqlx::query(
            r#"
            INSERT INTO ai_billing_meter
                (uuid, tenant_id, organization_id, data_scope, status, metadata, meter_code, display_name, description, modality, usage_type, billing_mode, default_unit, default_unit_size, quantity_precision, quantity_source, aggregation_mode, supports_tier, supports_expression, allow_negative_quantity, canonical_price_item_type, sort_order, id)
            VALUES
                ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, 1, 1, 1, CAST($11 AS NUMERIC), $12, 1, 1, false, false, false, 1, $13, $14)
            ON CONFLICT(tenant_id, organization_id, meter_code) DO UPDATE SET
                display_name = excluded.display_name,
                description = excluded.description,
                modality = excluded.modality,
                default_unit_size = excluded.default_unit_size,
                quantity_precision = excluded.quantity_precision,
                sort_order = excluded.sort_order,
                metadata = excluded.metadata,
                deleted_at = NULL,
                deleted_by = NULL,
                status = excluded.status
            "#,
        )
        .bind(stable_uuid("sdk-meter", &[&meter.meter_code]))
        .bind(SYSTEM_TENANT_ID)
        .bind(SYSTEM_ORGANIZATION_ID)
        .bind(SYSTEM_DATA_SCOPE)
        .bind(ACTIVE_STATUS)
        .bind(metadata_json(catalog, "sdkwork_models_meter", serde_json::json!({ "meterCode": meter.meter_code })))
        .bind(&meter.meter_code)
        .bind(&meter.display_name)
        .bind(&meter.description)
        .bind(modality_code(&meter.modality))
        .bind(&meter.default_unit_size)
        .bind(meter.quantity_precision.unwrap_or(0))
        .bind(meter.sort_order.unwrap_or(1000000))
        .bind(row_id)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn import_vendors(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
) -> Result<BTreeMap<String, i64>, sqlx::Error> {
    for item in catalog_vendor_records(catalog) {
        let row_id = stable_catalog_id("sdk-vendor", &[&item.vendor_code]);
        sqlx::query(
            r#"
            INSERT INTO ai_model_vendor
                (uuid, tenant_id, organization_id, data_scope, status, metadata, vendor_code, display_name, legal_name, description, website_url, docs_url, country_region, vendor_type, model_families, capabilities, supported_protocols, client_api_compatibility, open_source, sort_order, id)
            VALUES
                ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11, $12, $13, $14, $15::jsonb, $16::jsonb, $17::jsonb, $18::jsonb, $19, $20, $21)
            ON CONFLICT(tenant_id, organization_id, vendor_code) DO UPDATE SET
                display_name = excluded.display_name,
                legal_name = excluded.legal_name,
                description = excluded.description,
                website_url = excluded.website_url,
                docs_url = excluded.docs_url,
                country_region = excluded.country_region,
                vendor_type = excluded.vendor_type,
                model_families = excluded.model_families,
                capabilities = excluded.capabilities,
                supported_protocols = excluded.supported_protocols,
                client_api_compatibility = excluded.client_api_compatibility,
                open_source = excluded.open_source,
                sort_order = excluded.sort_order,
                metadata = excluded.metadata,
                deleted_at = NULL,
                deleted_by = NULL,
                status = excluded.status
            "#,
        )
        .bind(stable_uuid("sdk-vendor", &[&item.vendor_code]))
        .bind(SYSTEM_TENANT_ID)
        .bind(SYSTEM_ORGANIZATION_ID)
        .bind(SYSTEM_DATA_SCOPE)
        .bind(ACTIVE_STATUS)
        .bind(metadata_json(catalog, "sdkwork_models_vendor", serde_json::json!({ "sourceUrl": item.source_url })))
        .bind(item.vendor_code)
        .bind(item.display_name)
        .bind(item.legal_name)
        .bind(item.description)
        .bind(item.website_url)
        .bind(item.docs_url)
        .bind(item.country_region)
        .bind(vendor_type_code(&item.vendor_type))
        .bind(json_array(&item.model_families))
        .bind(json_array(&item.capabilities))
        .bind(json_array(&item.supported_protocols))
        .bind(serde_json::to_string(&item.client_api_compatibility).unwrap_or_else(|_| "{}".to_owned()))
        .bind(item.open_source)
        .bind(item.sort_order)
        .bind(row_id)
        .execute(&mut *conn)
        .await?;
    }
    load_vendor_ids(conn).await
}

async fn load_vendor_ids(conn: &mut PgConnection) -> Result<BTreeMap<String, i64>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, vendor_code FROM ai_model_vendor WHERE tenant_id = 0 AND organization_id = 0",
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.get("vendor_code"), row.get("id")))
        .collect())
}

async fn import_families(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    vendor_ids: &BTreeMap<String, i64>,
) -> Result<BTreeMap<(String, String, String), i64>, sqlx::Error> {
    for vendor in &catalog.vendors {
        let vendor_id = vendor_ids.get(&vendor.vendor.vendor_code).copied();
        for family in &vendor.families {
            let row_id = stable_catalog_id(
                "sdk-family",
                &[&vendor.vendor.vendor_code, &family.family_code],
            );
            sqlx::query(
                r#"
                INSERT INTO ai_model_family
                    (uuid, tenant_id, organization_id, data_scope, status, metadata, vendor_id, vendor_code, family_code, display_name, description, family_type, primary_modality, default_model, sort_order, id)
                VALUES
                    ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                ON CONFLICT(tenant_id, organization_id, vendor_code, family_code) DO UPDATE SET
                    vendor_id = excluded.vendor_id,
                    display_name = excluded.display_name,
                    description = excluded.description,
                    family_type = excluded.family_type,
                    primary_modality = excluded.primary_modality,
                    default_model = excluded.default_model,
                    sort_order = excluded.sort_order,
                    metadata = excluded.metadata,
                    deleted_at = NULL,
                    deleted_by = NULL,
                    status = excluded.status
                "#,
            )
            .bind(stable_uuid(
                "sdk-family",
                &[&vendor.vendor.vendor_code, &family.family_code],
            ))
            .bind(SYSTEM_TENANT_ID)
            .bind(SYSTEM_ORGANIZATION_ID)
            .bind(SYSTEM_DATA_SCOPE)
            .bind(ACTIVE_STATUS)
            .bind(metadata_json(catalog, "sdkwork_models_family", serde_json::json!({ "familyCode": family.family_code })))
            .bind(vendor_id)
            .bind(&vendor.vendor.vendor_code)
            .bind(&family.family_code)
            .bind(&family.display_name)
            .bind(&family.description)
            .bind(family_type_code(&family.family_type))
            .bind(modality_code(&family.primary_modality))
            .bind(&family.default_model)
            .bind(family.sort_order.unwrap_or(1000000))
            .bind(row_id)
            .execute(&mut *conn)
            .await?;
        }
    }
    load_family_ids(conn).await
}

async fn load_family_ids(
    conn: &mut PgConnection,
) -> Result<BTreeMap<(String, String, String), i64>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, vendor_code, family_code FROM ai_model_family WHERE tenant_id = 0 AND organization_id = 0")
        .fetch_all(&mut *conn)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                (
                    row.get("vendor_code"),
                    String::new(),
                    row.get("family_code"),
                ),
                row.get("id"),
            )
        })
        .collect())
}

async fn import_models(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    vendor_ids: &BTreeMap<String, i64>,
    family_ids: &BTreeMap<(String, String, String), i64>,
) -> Result<BTreeMap<String, i64>, sqlx::Error> {
    for (model_catalog_key, (vendor, model)) in catalog_identity_models(catalog) {
        let row_id = stable_catalog_id("sdk-model", &[&model.vendor_code, &model.model_id]);
        let vendor_id = vendor_ids.get(&model.vendor_code).copied();
        let family_id = family_ids
            .get(&(
                model.vendor_code.clone(),
                String::new(),
                model.family_code.clone(),
            ))
            .copied();
        sqlx::query(
                r#"
                INSERT INTO ai_model
                    (uuid, tenant_id, organization_id, data_scope, status, metadata, catalog_key, model, display_name, vendor_id, vendor_code, vendor_name_snapshot, family_id, family_code, provider_hint, model_family, capability, capabilities, modalities, input_modalities, output_modalities, color_token, docs_url, api_format, context_tokens, max_input_tokens, max_output_tokens, supports_streaming, supports_tools, supports_json_schema, performance_profile, rank_score, release_stage, shelf_state, routing_state, replacement_model, description, id)
                VALUES
                    ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18::jsonb, $19::jsonb, $20::jsonb, $21::jsonb, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31::jsonb, CAST($32 AS NUMERIC), $33, $34, $35, $36, $37, $38)
                ON CONFLICT(tenant_id, organization_id, catalog_key) DO UPDATE SET
                    display_name = excluded.display_name,
                    vendor_id = excluded.vendor_id,
                    vendor_code = excluded.vendor_code,
                    vendor_name_snapshot = excluded.vendor_name_snapshot,
                    family_id = excluded.family_id,
                    family_code = excluded.family_code,
                    model_family = excluded.model_family,
                    capability = excluded.capability,
                    capabilities = excluded.capabilities,
                    modalities = excluded.modalities,
                    input_modalities = excluded.input_modalities,
                    output_modalities = excluded.output_modalities,
                    color_token = excluded.color_token,
                    docs_url = excluded.docs_url,
                    api_format = excluded.api_format,
                    context_tokens = excluded.context_tokens,
                    max_input_tokens = excluded.max_input_tokens,
                    max_output_tokens = excluded.max_output_tokens,
                    supports_streaming = excluded.supports_streaming,
                    supports_tools = excluded.supports_tools,
                    supports_json_schema = excluded.supports_json_schema,
                    performance_profile = excluded.performance_profile,
                    rank_score = excluded.rank_score,
                    release_stage = excluded.release_stage,
                    shelf_state = excluded.shelf_state,
                    routing_state = excluded.routing_state,
                    replacement_model = excluded.replacement_model,
                    description = excluded.description,
                    metadata = excluded.metadata,
                    deleted_at = NULL,
                    deleted_by = NULL,
                    status = excluded.status
            "#,
        )
        .bind(stable_uuid(
            "sdk-model",
            &[&model.vendor_code, &model.model_id],
        ))
        .bind(SYSTEM_TENANT_ID)
        .bind(SYSTEM_ORGANIZATION_ID)
        .bind(SYSTEM_DATA_SCOPE)
        .bind(catalog_model_status(model))
        .bind(metadata_json(catalog, "sdkwork_models_model", serde_json::json!({ "sourceUrl": model.source.source_url, "lifecycle": model.lifecycle })))
        .bind(&model_catalog_key)
        .bind(&model.model_id)
        .bind(&model.display_name)
        .bind(vendor_id)
        .bind(&model.vendor_code)
        .bind(model.vendor_name.as_deref().unwrap_or(&vendor.vendor.display_name))
        .bind(family_id)
        .bind(&model.family_code)
        .bind(format!("{}_direct", model.vendor_code))
        .bind(&model.family_code)
        .bind(capability_code(&model.primary_capability))
        .bind(model_capabilities_json(model))
        .bind(model_modalities_json(model))
        .bind(json_array(&model.input_modalities))
        .bind(json_array(&model.output_modalities))
        .bind(&model.color_token)
        .bind(&model.source.source_url)
        .bind(&model.api_format)
        .bind(model.context_tokens)
        .bind(model.max_input_tokens)
        .bind(model.max_output_tokens)
        .bind(model.supports_streaming)
        .bind(model.supports_tools)
        .bind(model.supports_json_schema)
        .bind(serde_json::json!({
            "latencyP50Ms": model.latency_p50_ms,
            "latencyP95Ms": model.latency_p95_ms,
            "winRate": model.win_rate,
            "trendScore": model.trend_score,
            "strengths": model.strengths,
        }).to_string())
        .bind(model.rank_score.as_deref().unwrap_or("0"))
        .bind(release_stage_code(&model.release_stage))
        .bind(shelf_state_code(&model.shelf_state))
        .bind(routing_state_code(&model.routing_state))
        .bind(&model.replacement_model)
        .bind(&model.description)
        .bind(row_id)
        .execute(&mut *conn)
        .await?;
    }
    load_model_ids(conn).await
}

async fn load_model_ids(conn: &mut PgConnection) -> Result<BTreeMap<String, i64>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, catalog_key FROM ai_model WHERE tenant_id = 0 AND organization_id = 0",
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.get("catalog_key"), row.get("id")))
        .collect())
}

async fn import_capabilities(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    model_ids: &BTreeMap<String, i64>,
) -> Result<(), sqlx::Error> {
    for (model_catalog_key, (_, model)) in public_catalog_identity_models(catalog) {
        let model_id = model_ids.get(&model_catalog_key).copied();
        let capabilities = if model.capabilities.is_empty() {
            vec![model.primary_capability.clone()]
        } else {
            model.capabilities.clone()
        };
        for (index, capability) in capabilities.iter().enumerate() {
            let row_id = stable_catalog_id(
                "sdk-cap",
                &[&model.vendor_code, &model.model_id, capability],
            );
            sqlx::query(
                    r#"
                    INSERT INTO ai_model_capability
                        (uuid, tenant_id, organization_id, data_scope, status, metadata, model_id, catalog_key, model, vendor_code, capability, capability_code, modality, input_modalities, output_modalities, endpoint_formats, supported, schema_version, sort_order, id)
                    VALUES
                        ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11, $12, $13, $14::jsonb, $15::jsonb, $16::jsonb, true, $17, $18, $19)
                    ON CONFLICT(uuid) DO UPDATE SET
                        model_id = excluded.model_id,
                        catalog_key = excluded.catalog_key,
                        model = excluded.model,
                        vendor_code = excluded.vendor_code,
                        capability = excluded.capability,
                        capability_code = excluded.capability_code,
                        modality = excluded.modality,
                        input_modalities = excluded.input_modalities,
                        output_modalities = excluded.output_modalities,
                        endpoint_formats = excluded.endpoint_formats,
                        supported = excluded.supported,
                        schema_version = excluded.schema_version,
                        sort_order = excluded.sort_order,
                        metadata = excluded.metadata,
                        deleted_at = NULL,
                        deleted_by = NULL,
                        status = excluded.status
                    "#,
            )
            .bind(stable_uuid(
                "sdk-cap",
                &[&model.vendor_code, &model.model_id, capability],
            ))
            .bind(SYSTEM_TENANT_ID)
            .bind(SYSTEM_ORGANIZATION_ID)
            .bind(SYSTEM_DATA_SCOPE)
            .bind(ACTIVE_STATUS)
            .bind(metadata_json(catalog, "sdkwork_models_capability", serde_json::json!({ "capability": capability })))
            .bind(model_id)
            .bind(&model_catalog_key)
            .bind(&model.model_id)
            .bind(&model.vendor_code)
            .bind(capability_code(capability))
            .bind(capability)
            .bind(primary_modality(model))
            .bind(json_array(&model.input_modalities))
            .bind(json_array(&model.output_modalities))
            .bind(serde_json::json!([model.api_format]).to_string())
            .bind(&catalog.manifest.schema_version)
            .bind((index as i32) + 1)
            .bind(row_id)
            .execute(&mut *conn)
            .await?;
        }
    }
    Ok(())
}

async fn import_modalities(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
) -> Result<BTreeMap<String, i64>, sqlx::Error> {
    for item in catalog_modality_projections(catalog) {
        let row_id = stable_catalog_id("sdk-modality", &[&item.modality_code]);
        sqlx::query(
            r#"
            INSERT INTO ai_modality
                (uuid, tenant_id, organization_id, data_scope, status, metadata, modality_code, display_name, modality_group, description, input_supported, output_supported, sort_order, id)
            VALUES
                ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT(tenant_id, organization_id, modality_code) DO UPDATE SET
                display_name = excluded.display_name,
                modality_group = excluded.modality_group,
                description = excluded.description,
                input_supported = excluded.input_supported,
                output_supported = excluded.output_supported,
                sort_order = excluded.sort_order,
                metadata = excluded.metadata,
                deleted_at = NULL,
                deleted_by = NULL,
                status = excluded.status
            "#,
        )
        .bind(item.uuid)
        .bind(SYSTEM_TENANT_ID)
        .bind(SYSTEM_ORGANIZATION_ID)
        .bind(SYSTEM_DATA_SCOPE)
        .bind(ACTIVE_STATUS)
        .bind(metadata_json(
            catalog,
            "sdkwork_models_modality",
            serde_json::json!({ "modalityCode": &item.modality_code }),
        ))
        .bind(item.modality_code)
        .bind(item.display_name)
        .bind(item.modality_group)
        .bind(item.description)
        .bind(item.input_supported)
        .bind(item.output_supported)
        .bind(item.sort_order)
        .bind(row_id)
        .execute(&mut *conn)
        .await?;
    }
    load_modality_ids(conn).await
}

async fn load_modality_ids(conn: &mut PgConnection) -> Result<BTreeMap<String, i64>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, modality_code FROM ai_modality WHERE tenant_id = 0 AND organization_id = 0",
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.get("modality_code"), row.get("id")))
        .collect())
}

async fn import_api_endpoints(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
) -> Result<BTreeMap<String, i64>, sqlx::Error> {
    for item in catalog_api_endpoint_projections(catalog) {
        let row_id = stable_catalog_id("sdk-api-endpoint", &[&item.endpoint_code]);
        sqlx::query(
            r#"
            INSERT INTO ai_api_endpoint
                (uuid, tenant_id, organization_id, data_scope, status, metadata, endpoint_code, protocol_code, display_name, method, path_template, request_schema, response_schema, streaming_supported, sort_order, id)
            VALUES
                ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11, '{}'::jsonb, '{}'::jsonb, $12, $13, $14)
            ON CONFLICT(tenant_id, organization_id, endpoint_code) DO UPDATE SET
                protocol_code = excluded.protocol_code,
                display_name = excluded.display_name,
                method = excluded.method,
                path_template = excluded.path_template,
                request_schema = excluded.request_schema,
                response_schema = excluded.response_schema,
                streaming_supported = excluded.streaming_supported,
                sort_order = excluded.sort_order,
                metadata = excluded.metadata,
                deleted_at = NULL,
                deleted_by = NULL,
                status = excluded.status
            "#,
        )
        .bind(item.uuid)
        .bind(SYSTEM_TENANT_ID)
        .bind(SYSTEM_ORGANIZATION_ID)
        .bind(SYSTEM_DATA_SCOPE)
        .bind(ACTIVE_STATUS)
        .bind(metadata_json(
            catalog,
            "sdkwork_models_api_endpoint",
            serde_json::json!({ "endpointCode": &item.endpoint_code }),
        ))
        .bind(item.endpoint_code)
        .bind(item.protocol_code)
        .bind(item.display_name)
        .bind(item.method)
        .bind(item.path_template)
        .bind(item.streaming_supported)
        .bind(item.sort_order)
        .bind(row_id)
        .execute(&mut *conn)
        .await?;
    }
    load_api_endpoint_ids(conn).await
}

async fn load_api_endpoint_ids(
    conn: &mut PgConnection,
) -> Result<BTreeMap<String, i64>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, endpoint_code FROM ai_api_endpoint WHERE tenant_id = 0 AND organization_id = 0",
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.get("endpoint_code"), row.get("id")))
        .collect())
}

async fn import_vendor_modalities(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    vendor_ids: &BTreeMap<String, i64>,
    modality_ids: &BTreeMap<String, i64>,
) -> Result<(), sqlx::Error> {
    for item in catalog_vendor_modality_projections(catalog) {
        let row_id = stable_catalog_id(
            "sdk-vendor-modality",
            &[&item.vendor_code, &item.modality_code],
        );
        sqlx::query(
            r#"
            INSERT INTO ai_vendor_modality
                (uuid, tenant_id, organization_id, data_scope, status, metadata, vendor_id, vendor_code, modality_id, modality_code, supported, sort_order, id)
            VALUES
                ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, true, $11, $12)
            ON CONFLICT(tenant_id, organization_id, vendor_code, modality_code) DO UPDATE SET
                vendor_id = excluded.vendor_id,
                modality_id = excluded.modality_id,
                supported = excluded.supported,
                sort_order = excluded.sort_order,
                metadata = excluded.metadata,
                deleted_at = NULL,
                deleted_by = NULL,
                status = excluded.status
            "#,
        )
        .bind(item.uuid)
        .bind(SYSTEM_TENANT_ID)
        .bind(SYSTEM_ORGANIZATION_ID)
        .bind(SYSTEM_DATA_SCOPE)
        .bind(ACTIVE_STATUS)
        .bind(metadata_json(
            catalog,
            "sdkwork_models_vendor_modality",
            serde_json::json!({ "vendorCode": &item.vendor_code, "modalityCode": &item.modality_code }),
        ))
        .bind(vendor_ids.get(&item.vendor_code).copied())
        .bind(item.vendor_code)
        .bind(modality_ids.get(&item.modality_code).copied())
        .bind(item.modality_code)
        .bind(item.sort_order)
        .bind(row_id)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn import_vendor_api_endpoints(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    vendor_ids: &BTreeMap<String, i64>,
    endpoint_ids: &BTreeMap<String, i64>,
) -> Result<(), sqlx::Error> {
    for item in catalog_vendor_api_endpoint_projections(catalog) {
        let row_id = stable_catalog_id(
            "sdk-vendor-endpoint",
            &[&item.vendor_code, &item.endpoint_code],
        );
        sqlx::query(
            r#"
            INSERT INTO ai_vendor_api_endpoint
                (uuid, tenant_id, organization_id, data_scope, status, metadata, vendor_id, vendor_code, api_endpoint_id, endpoint_code, supported, sort_order, id)
            VALUES
                ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, true, $11, $12)
            ON CONFLICT(tenant_id, organization_id, vendor_code, endpoint_code) DO UPDATE SET
                vendor_id = excluded.vendor_id,
                api_endpoint_id = excluded.api_endpoint_id,
                supported = excluded.supported,
                sort_order = excluded.sort_order,
                metadata = excluded.metadata,
                deleted_at = NULL,
                deleted_by = NULL,
                status = excluded.status
            "#,
        )
        .bind(item.uuid)
        .bind(SYSTEM_TENANT_ID)
        .bind(SYSTEM_ORGANIZATION_ID)
        .bind(SYSTEM_DATA_SCOPE)
        .bind(ACTIVE_STATUS)
        .bind(metadata_json(
            catalog,
            "sdkwork_models_vendor_api_endpoint",
            serde_json::json!({ "vendorCode": &item.vendor_code, "endpointCode": &item.endpoint_code }),
        ))
        .bind(vendor_ids.get(&item.vendor_code).copied())
        .bind(item.vendor_code)
        .bind(endpoint_ids.get(&item.endpoint_code).copied())
        .bind(item.endpoint_code)
        .bind(item.sort_order)
        .bind(row_id)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn import_modality_api_endpoints(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    modality_ids: &BTreeMap<String, i64>,
    endpoint_ids: &BTreeMap<String, i64>,
) -> Result<(), sqlx::Error> {
    for item in catalog_modality_api_endpoint_projections(catalog) {
        let row_id = stable_catalog_id(
            "sdk-modality-endpoint",
            &[&item.modality_code, &item.endpoint_code],
        );
        sqlx::query(
            r#"
            INSERT INTO ai_modality_api_endpoint
                (uuid, tenant_id, organization_id, data_scope, status, metadata, modality_id, modality_code, api_endpoint_id, endpoint_code, supported, sort_order, id)
            VALUES
                ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, true, $11, $12)
            ON CONFLICT(tenant_id, organization_id, modality_code, endpoint_code) DO UPDATE SET
                modality_id = excluded.modality_id,
                api_endpoint_id = excluded.api_endpoint_id,
                supported = excluded.supported,
                sort_order = excluded.sort_order,
                metadata = excluded.metadata,
                deleted_at = NULL,
                deleted_by = NULL,
                status = excluded.status
            "#,
        )
        .bind(item.uuid)
        .bind(SYSTEM_TENANT_ID)
        .bind(SYSTEM_ORGANIZATION_ID)
        .bind(SYSTEM_DATA_SCOPE)
        .bind(ACTIVE_STATUS)
        .bind(metadata_json(
            catalog,
            "sdkwork_models_modality_api_endpoint",
            serde_json::json!({ "modalityCode": &item.modality_code, "endpointCode": &item.endpoint_code }),
        ))
        .bind(modality_ids.get(&item.modality_code).copied())
        .bind(item.modality_code)
        .bind(endpoint_ids.get(&item.endpoint_code).copied())
        .bind(item.endpoint_code)
        .bind(item.sort_order)
        .bind(row_id)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn import_model_modalities(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    model_ids: &BTreeMap<String, i64>,
    modality_ids: &BTreeMap<String, i64>,
) -> Result<(), sqlx::Error> {
    for item in catalog_model_modality_projections(catalog) {
        let row_id = stable_catalog_id(
            "sdk-model-modality",
            &[&item.catalog_key, &item.modality_code, &item.direction],
        );
        sqlx::query(
            r#"
            INSERT INTO ai_model_modality
                (uuid, tenant_id, organization_id, data_scope, status, metadata, model_id, catalog_key, model, vendor_code, modality_id, modality_code, direction, supported, sort_order, id)
            VALUES
                ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11, $12, $13, true, $14, $15)
            ON CONFLICT(tenant_id, organization_id, catalog_key, modality_code, direction) DO UPDATE SET
                model_id = excluded.model_id,
                model = excluded.model,
                vendor_code = excluded.vendor_code,
                modality_id = excluded.modality_id,
                supported = excluded.supported,
                sort_order = excluded.sort_order,
                metadata = excluded.metadata,
                deleted_at = NULL,
                deleted_by = NULL,
                status = excluded.status
            "#,
        )
        .bind(item.uuid)
        .bind(SYSTEM_TENANT_ID)
        .bind(SYSTEM_ORGANIZATION_ID)
        .bind(SYSTEM_DATA_SCOPE)
        .bind(ACTIVE_STATUS)
        .bind(metadata_json(
            catalog,
            "sdkwork_models_model_modality",
            serde_json::json!({ "catalogKey": &item.catalog_key, "modalityCode": &item.modality_code, "direction": &item.direction }),
        ))
        .bind(model_ids.get(&item.catalog_key).copied())
        .bind(item.catalog_key)
        .bind(item.model)
        .bind(item.vendor_code)
        .bind(modality_ids.get(&item.modality_code).copied())
        .bind(item.modality_code)
        .bind(item.direction)
        .bind(item.sort_order)
        .bind(row_id)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn import_model_api_endpoints(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    model_ids: &BTreeMap<String, i64>,
    endpoint_ids: &BTreeMap<String, i64>,
) -> Result<(), sqlx::Error> {
    for item in catalog_model_api_endpoint_projections(catalog) {
        let row_id = stable_catalog_id(
            "sdk-model-endpoint",
            &[&item.catalog_key, &item.endpoint_code],
        );
        sqlx::query(
            r#"
            INSERT INTO ai_model_api_endpoint
                (uuid, tenant_id, organization_id, data_scope, status, metadata, model_id, catalog_key, model, vendor_code, api_endpoint_id, endpoint_code, provider_native_model, default_parameters, supports_streaming, supported, sort_order, id)
            VALUES
                ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11, $12, $13, $14::jsonb, $15, true, $16, $17)
            ON CONFLICT(tenant_id, organization_id, catalog_key, endpoint_code) DO UPDATE SET
                model_id = excluded.model_id,
                model = excluded.model,
                vendor_code = excluded.vendor_code,
                api_endpoint_id = excluded.api_endpoint_id,
                provider_native_model = excluded.provider_native_model,
                default_parameters = excluded.default_parameters,
                supports_streaming = excluded.supports_streaming,
                supported = excluded.supported,
                sort_order = excluded.sort_order,
                metadata = excluded.metadata,
                deleted_at = NULL,
                deleted_by = NULL,
                status = excluded.status
            "#,
        )
        .bind(item.uuid)
        .bind(SYSTEM_TENANT_ID)
        .bind(SYSTEM_ORGANIZATION_ID)
        .bind(SYSTEM_DATA_SCOPE)
        .bind(ACTIVE_STATUS)
        .bind(metadata_json(
            catalog,
            "sdkwork_models_model_api_endpoint",
            serde_json::json!({ "catalogKey": &item.catalog_key, "endpointCode": &item.endpoint_code }),
        ))
        .bind(model_ids.get(&item.catalog_key).copied())
        .bind(item.catalog_key)
        .bind(item.model)
        .bind(item.vendor_code)
        .bind(endpoint_ids.get(&item.endpoint_code).copied())
        .bind(item.endpoint_code)
        .bind(item.provider_native_model)
        .bind(item.default_parameters)
        .bind(item.supports_streaming)
        .bind(item.sort_order)
        .bind(row_id)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn import_ai_resources(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    vendor_ids: &BTreeMap<String, i64>,
    model_ids: &BTreeMap<String, i64>,
    modality_ids: &BTreeMap<String, i64>,
    endpoint_ids: &BTreeMap<String, i64>,
) -> Result<(), sqlx::Error> {
    for item in catalog_ai_resource_projections(catalog) {
        let row_id = stable_catalog_id("sdk-cap-resource", &[&item.resource_code]);
        let vendor_id = item
            .vendor_code
            .as_ref()
            .and_then(|vendor_code| vendor_ids.get(vendor_code).copied());
        let model_id = item
            .catalog_key
            .as_ref()
            .and_then(|catalog_key| model_ids.get(catalog_key).copied());
        let modality_id = item
            .modality_code
            .as_ref()
            .and_then(|modality_code| modality_ids.get(modality_code).copied());
        let api_endpoint_id = item
            .api_endpoint_code
            .as_ref()
            .and_then(|endpoint_code| endpoint_ids.get(endpoint_code).copied());
        sqlx::query(
            r#"
            INSERT INTO ai_resource
                (uuid, tenant_id, organization_id, data_scope, status, metadata, resource_code, resource_type, display_name, vendor_id, vendor_code, modality_id, modality_code, api_endpoint_id, api_code, model_id, catalog_key, model, provider_native_model, resource_schema, metadata_schema, description, sort_order, id)
            VALUES
                ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20::jsonb, $21::jsonb, $22, $23, $24)
            ON CONFLICT(tenant_id, organization_id, resource_code) DO UPDATE SET
                resource_type = excluded.resource_type,
                display_name = excluded.display_name,
                vendor_id = excluded.vendor_id,
                vendor_code = excluded.vendor_code,
                modality_id = excluded.modality_id,
                modality_code = excluded.modality_code,
                api_endpoint_id = excluded.api_endpoint_id,
                api_code = excluded.api_code,
                model_id = excluded.model_id,
                catalog_key = excluded.catalog_key,
                model = excluded.model,
                provider_native_model = excluded.provider_native_model,
                resource_schema = excluded.resource_schema,
                metadata_schema = excluded.metadata_schema,
                description = excluded.description,
                sort_order = excluded.sort_order,
                metadata = excluded.metadata,
                deleted_at = NULL,
                deleted_by = NULL,
                status = excluded.status
            "#,
        )
        .bind(item.uuid)
        .bind(SYSTEM_TENANT_ID)
        .bind(SYSTEM_ORGANIZATION_ID)
        .bind(SYSTEM_DATA_SCOPE)
        .bind(ACTIVE_STATUS)
        .bind(metadata_json(
            catalog,
            "sdkwork_models_ai_resource",
            serde_json::json!({
                "resourceCode": &item.resource_code,
                "resourceType": &item.resource_kind,
                "compositionMode": &item.composition_mode
            }),
        ))
        .bind(item.resource_code)
        .bind(item.resource_kind)
        .bind(item.display_name)
        .bind(vendor_id)
        .bind(item.vendor_code)
        .bind(modality_id)
        .bind(item.modality_code)
        .bind(api_endpoint_id)
        .bind(item.api_endpoint_code)
        .bind(model_id)
        .bind(item.catalog_key)
        .bind(item.model)
        .bind(item.provider_native_model)
        .bind(item.capability_schema)
        .bind(item.metadata_schema)
        .bind(item.description)
        .bind(item.sort_order)
        .bind(row_id)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn import_pricing(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    model_ids: &BTreeMap<String, i64>,
) -> Result<(), sqlx::Error> {
    let public_model_keys = public_catalog_identity_models(catalog)
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    for vendor in &catalog.vendors {
        for pricing in &vendor.pricing {
            let model_catalog_key = model_catalog_key(&pricing.vendor_code, &pricing.model_id);
            if !public_model_keys.contains(&model_catalog_key) {
                continue;
            }
            let pricing_catalog_key = pricing_catalog_key(&pricing.vendor_code, &pricing.model_id);
            for (index, price) in pricing.prices.iter().enumerate() {
                let row_id = stable_catalog_id(
                    "sdk-price",
                    &[
                        &pricing.vendor_code,
                        &pricing.region_code,
                        &pricing.model_id,
                        &price.price_id,
                    ],
                );
                let model_id = model_ids.get(&model_catalog_key).copied();
                let meter_id: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM ai_billing_meter WHERE tenant_id = 0 AND organization_id = 0 AND meter_code = $1",
                )
                .bind(&price.meter_code)
                .fetch_optional(&mut *conn)
                .await?;
                sqlx::query(
                    r#"
                    INSERT INTO ai_model_pricing
                        (uuid, tenant_id, organization_id, data_scope, status, metadata, model_id, catalog_key, model, vendor_code, region_code, provider_code, price_side, pricing_scope, billing_type, billing_mode, billing_meter_id, billing_meter_code, price_item_type, unit, unit_size, metering_mode, quantity_source, minimum_quantity, quantity_step, included_quantity, unit_price, currency, rounding_mode, min_charge_amount, pricing_formula_mode, price_origin, priority, price_version, source_url, observed_at, effective_from, id)
                    VALUES
                        ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11, $12, $13, $14, 1, 1, $15, $16, 1, 1, CAST($17 AS NUMERIC), 1, 1, CAST($18 AS NUMERIC), CAST($19 AS NUMERIC), 0, CAST($20 AS NUMERIC), $21, 1, 0, 1, 1, $22, $23, $24, $25::timestamptz, $26::timestamptz, $27)
                    ON CONFLICT(uuid) DO UPDATE SET
                        model_id = excluded.model_id,
                        catalog_key = excluded.catalog_key,
                        model = excluded.model,
                        vendor_code = excluded.vendor_code,
                        region_code = excluded.region_code,
                        provider_code = excluded.provider_code,
                        price_side = excluded.price_side,
                        pricing_scope = excluded.pricing_scope,
                        billing_meter_id = excluded.billing_meter_id,
                        billing_meter_code = excluded.billing_meter_code,
                        unit_size = excluded.unit_size,
                        minimum_quantity = excluded.minimum_quantity,
                        quantity_step = excluded.quantity_step,
                        unit_price = excluded.unit_price,
                        currency = excluded.currency,
                        priority = excluded.priority,
                        price_version = excluded.price_version,
                        source_url = excluded.source_url,
                        observed_at = excluded.observed_at,
                        effective_from = excluded.effective_from,
                        metadata = excluded.metadata,
                        deleted_at = NULL,
                        deleted_by = NULL,
                        status = excluded.status
                    "#,
                )
                .bind(stable_uuid(
                    "sdk-price",
                    &[
                        &pricing.vendor_code,
                        &pricing.region_code,
                        &pricing.model_id,
                        &price.price_id,
                    ],
                ))
                .bind(SYSTEM_TENANT_ID)
                .bind(SYSTEM_ORGANIZATION_ID)
                .bind(SYSTEM_DATA_SCOPE)
                .bind(ACTIVE_STATUS)
                .bind(metadata_json(catalog, "sdkwork_models_pricing", serde_json::json!({
                    "priceId": price.price_id,
                    "priceSide": price.price_side,
                    "sourceUrl": price.source.source_url
                })))
                .bind(model_id)
                .bind(&pricing_catalog_key)
                .bind(&pricing.model_id)
                .bind(&pricing.vendor_code)
                .bind(&pricing.region_code)
                .bind(price_provider_code(
                    &pricing.vendor_code,
                    &pricing.region_code,
                    &price.price_side,
                    price.pricing_scope.as_deref(),
                ))
                .bind(price_side_code(&price.price_side))
                .bind(pricing_scope_code(price.pricing_scope.as_deref()))
                .bind(meter_id)
                .bind(&price.meter_code)
                .bind(&price.unit_size)
                .bind(&price.minimum_quantity)
                .bind(price.quantity_step.as_deref().unwrap_or("1"))
                .bind(&price.unit_price)
                .bind(price.currency.as_deref().unwrap_or(&pricing.currency))
                .bind((index as i32) + 1)
                .bind(&catalog.manifest.catalog_version)
                .bind(&price.source.source_url)
                .bind(&price.source.observed_at)
                .bind(&price.effective_from)
                .bind(row_id)
                .execute(&mut *conn)
                .await?;
            }
        }
    }
    Ok(())
}

async fn import_rankings(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    model_ids: &BTreeMap<String, i64>,
) -> Result<(), sqlx::Error> {
    let model_map = public_catalog_identity_models(catalog);
    for vendor in &catalog.vendors {
        for snapshot in &vendor.rankings {
            for item in &snapshot.items {
                let item_catalog_key =
                    pricing_catalog_key(&vendor.vendor.vendor_code, &item.model_id);
                let model_lookup_key =
                    model_catalog_key(&vendor.vendor.vendor_code, &item.model_id);
                let Some((_, model)) = model_map.get(&model_lookup_key) else {
                    continue;
                };
                let row_id = stable_catalog_id(
                    "sdk-rank",
                    &[
                        &snapshot.snapshot_date,
                        &snapshot.rank_scope,
                        &vendor.vendor.vendor_code,
                        &vendor.vendor.region_code,
                        &item.model_id,
                    ],
                );
                sqlx::query(
                    r#"
                    INSERT INTO ai_model_rank_snapshot
                        (uuid, tenant_id, organization_id, source_type, source_version, status, metadata, snapshot_date, snapshot_period, rank_scope, model_id, catalog_key, model, vendor_code, region_code, vendor_name_snapshot, provider_code, modality, rank_no, previous_rank_no, color_token, pricing_text, strengths, latency_p50_ms, latency_p95_ms, win_rate, trend_score, rank_payload, id)
                    VALUES
                        ($1, $2, $3, $4, 1, $5, $6::jsonb, $7::date, 1, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21::jsonb, $22, $23, CAST($24 AS NUMERIC), CAST($25 AS NUMERIC), $26::jsonb, $27)
                    ON CONFLICT(tenant_id, organization_id, snapshot_date, snapshot_period, rank_scope, vendor_code, region_code, catalog_key) DO UPDATE SET
                        model_id = excluded.model_id,
                        catalog_key = excluded.catalog_key,
                        vendor_code = excluded.vendor_code,
                        region_code = excluded.region_code,
                        vendor_name_snapshot = excluded.vendor_name_snapshot,
                        provider_code = excluded.provider_code,
                        modality = excluded.modality,
                        rank_no = excluded.rank_no,
                        previous_rank_no = excluded.previous_rank_no,
                        color_token = excluded.color_token,
                        pricing_text = excluded.pricing_text,
                        strengths = excluded.strengths,
                        latency_p50_ms = excluded.latency_p50_ms,
                        latency_p95_ms = excluded.latency_p95_ms,
                        win_rate = excluded.win_rate,
                        trend_score = excluded.trend_score,
                        rank_payload = excluded.rank_payload,
                        metadata = excluded.metadata,
                        status = excluded.status
                    "#,
                )
                .bind(stable_uuid(
                    "sdk-rank",
                    &[
                        &snapshot.snapshot_date,
                        &snapshot.rank_scope,
                        &vendor.vendor.vendor_code,
                        &vendor.vendor.region_code,
                        &item.model_id,
                    ],
                ))
                .bind(SYSTEM_TENANT_ID)
                .bind(SYSTEM_ORGANIZATION_ID)
                .bind("sdkwork_models")
                .bind(ACTIVE_STATUS)
                .bind(metadata_json(catalog, "sdkwork_models_ranking", serde_json::json!({ "rankScope": snapshot.rank_scope })))
                .bind(&snapshot.snapshot_date)
                .bind(&snapshot.rank_scope)
                .bind(model_ids.get(&model_lookup_key).copied())
                .bind(&item_catalog_key)
                .bind(&item.model_id)
                .bind(&model.vendor_code)
                .bind(&vendor.vendor.region_code)
                .bind(model.vendor_name.as_deref().unwrap_or(&vendor.vendor.display_name))
                .bind(format!(
                    "{}_{}_direct",
                    model.vendor_code, vendor.vendor.region_code
                ))
                .bind(primary_modality(model))
                .bind(item.rank_no)
                .bind(item.previous_rank_no)
                .bind(&model.color_token)
                .bind(item.pricing_text.clone().unwrap_or_else(|| "catalog reference".to_owned()))
                .bind(json_array(&model.strengths))
                .bind(model.latency_p50_ms)
                .bind(model.latency_p95_ms)
                .bind(model.win_rate.as_deref().unwrap_or("0"))
                .bind(model.trend_score.as_deref().unwrap_or("0"))
                .bind(serde_json::json!({ "modelId": item.model_id, "rankNo": item.rank_no }).to_string())
                .bind(row_id)
                .execute(&mut *conn)
                .await?;
            }
        }
    }
    Ok(())
}

async fn update_family_defaults(
    conn: &mut PgConnection,
    catalog: &ModelCatalog,
    model_ids: &BTreeMap<String, i64>,
) -> Result<(), sqlx::Error> {
    for vendor in &catalog.vendors {
        for family in &vendor.families {
            if let Some(default_model) = &family.default_model {
                let default_catalog_key =
                    model_catalog_key(&vendor.vendor.vendor_code, default_model);
                sqlx::query(
                    r#"
                    UPDATE ai_model_family
                    SET default_model_id = $1, default_model = $2
                    WHERE tenant_id = 0 AND organization_id = 0 AND vendor_code = $3 AND family_code = $4
                    "#,
                )
                .bind(model_ids.get(&default_catalog_key).copied())
                .bind(default_model)
                .bind(&vendor.vendor.vendor_code)
                .bind(&family.family_code)
                .execute(&mut *conn)
                .await?;
            }
        }
    }
    Ok(())
}
