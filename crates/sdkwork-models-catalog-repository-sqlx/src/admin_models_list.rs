use sdkwork_models_contract_service::ListAdminAiModelsQuery;

pub const LIST_MODELS_BASE_WHERE_POSTGRES: &str = r#"
        WHERE (m.tenant_id IS NULL OR m.tenant_id = 0 OR m.tenant_id = $1)
          AND (m.organization_id IS NULL OR m.organization_id = 0 OR m.organization_id = $2)
          AND m.deleted_at IS NULL
          AND NOT EXISTS (
              SELECT 1
              FROM ai_model tenant_model
              WHERE tenant_model.tenant_id = $3
                AND tenant_model.organization_id = $4
                AND tenant_model.model = m.model
                AND tenant_model.id <> m.id
                AND tenant_model.deleted_at IS NULL
          )
          AND ($7::text IS NULL OR m.vendor_id::text = $7)
          AND ($8::text IS NULL OR m.vendor_code = $8)
          AND (
              $9::text IS NULL
              OR m.display_name ILIKE $9
              OR m.model ILIKE $9
          )
        ORDER BY
          COALESCE(m.rank_score, 0) DESC,
          CASE WHEN m.tenant_id = $5 AND m.organization_id = $6 THEN 0 ELSE 1 END,
          m.display_name ASC NULLS LAST,
          m.id ASC
        LIMIT $10 OFFSET $11
        "#;

pub const LIST_MODELS_COUNT_WHERE_POSTGRES: &str = r#"
        WHERE (m.tenant_id IS NULL OR m.tenant_id = 0 OR m.tenant_id = $1)
          AND (m.organization_id IS NULL OR m.organization_id = 0 OR m.organization_id = $2)
          AND m.deleted_at IS NULL
          AND NOT EXISTS (
              SELECT 1
              FROM ai_model tenant_model
              WHERE tenant_model.tenant_id = $3
                AND tenant_model.organization_id = $4
                AND tenant_model.model = m.model
                AND tenant_model.id <> m.id
                AND tenant_model.deleted_at IS NULL
          )
          AND ($5::text IS NULL OR m.vendor_id::text = $5)
          AND ($6::text IS NULL OR m.vendor_code = $6)
          AND (
              $7::text IS NULL
              OR m.display_name ILIKE $7
              OR m.model ILIKE $7
          )
        "#;

pub const LIST_MODELS_BASE_WHERE_SQLITE: &str = r#"
        WHERE (m.tenant_id IS NULL OR m.tenant_id = 0 OR m.tenant_id = ?)
          AND (m.organization_id IS NULL OR m.organization_id = 0 OR m.organization_id = ?)
          AND m.deleted_at IS NULL
          AND NOT EXISTS (
              SELECT 1
              FROM ai_model tenant_model
              WHERE tenant_model.tenant_id = ?
                AND tenant_model.organization_id = ?
                AND tenant_model.model = m.model
                AND tenant_model.id <> m.id
                AND tenant_model.deleted_at IS NULL
          )
          AND (? IS NULL OR m.vendor_id = ?)
          AND (? IS NULL OR m.vendor_code = ?)
          AND (
              ? IS NULL
              OR lower(m.display_name) LIKE lower(?)
              OR lower(m.model) LIKE lower(?)
          )
        ORDER BY
          CAST(COALESCE(m.rank_score, '0') AS REAL) DESC,
          CASE WHEN m.tenant_id = ? AND m.organization_id = ? THEN 0 ELSE 1 END,
          m.display_name ASC,
          m.id ASC
        LIMIT ? OFFSET ?
        "#;

pub const LIST_MODELS_COUNT_WHERE_SQLITE: &str = r#"
        WHERE (m.tenant_id IS NULL OR m.tenant_id = 0 OR m.tenant_id = ?)
          AND (m.organization_id IS NULL OR m.organization_id = 0 OR m.organization_id = ?)
          AND m.deleted_at IS NULL
          AND NOT EXISTS (
              SELECT 1
              FROM ai_model tenant_model
              WHERE tenant_model.tenant_id = ?
                AND tenant_model.organization_id = ?
                AND tenant_model.model = m.model
                AND tenant_model.id <> m.id
                AND tenant_model.deleted_at IS NULL
          )
          AND (? IS NULL OR m.vendor_id = ?)
          AND (? IS NULL OR m.vendor_code = ?)
          AND (
              ? IS NULL
              OR lower(m.display_name) LIKE lower(?)
              OR lower(m.model) LIKE lower(?)
          )
        "#;

pub fn normalized_search_pattern(query: &ListAdminAiModelsQuery) -> Option<String> {
    query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{value}%"))
}

pub fn optional_non_empty(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}
