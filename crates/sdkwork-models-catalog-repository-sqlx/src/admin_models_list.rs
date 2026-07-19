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
          AND (cardinality($8::text[]) = 0 OR m.vendor_code = ANY($8))
          AND (
              $9::text IS NULL
              OR m.display_name ILIKE $9
              OR m.model ILIKE $9
          )
          AND (cardinality($10::int[]) = 0 OR m.capability = ANY($10))
        ORDER BY
          COALESCE(m.rank_score, 0) DESC,
          CASE WHEN m.tenant_id = $5 AND m.organization_id = $6 THEN 0 ELSE 1 END,
          m.display_name ASC NULLS LAST,
          m.id ASC
        LIMIT $11 OFFSET $12
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
          AND (cardinality($6::text[]) = 0 OR m.vendor_code = ANY($6))
          AND (
              $7::text IS NULL
              OR m.display_name ILIKE $7
              OR m.model ILIKE $7
          )
          AND (cardinality($8::int[]) = 0 OR m.capability = ANY($8))
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
          AND (
              ? IS NULL
              OR lower(m.display_name) LIKE lower(?)
              OR lower(m.model) LIKE lower(?)
          )
        "#;

pub const LIST_MODELS_ORDER_PAGE_SQLITE: &str = r#"
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
          AND (
              ? IS NULL
              OR lower(m.display_name) LIKE lower(?)
              OR lower(m.model) LIKE lower(?)
          )
        "#;

pub fn sqlite_capability_in_clause(code_count: usize) -> String {
    if code_count == 0 {
        return String::new();
    }
    let placeholders = (0..code_count).map(|_| "?").collect::<Vec<_>>().join(", ");
    format!(" AND m.capability IN ({placeholders})")
}

pub fn sqlite_vendor_codes_in_clause(code_count: usize) -> String {
    if code_count == 0 {
        return String::new();
    }
    let placeholders = (0..code_count).map(|_| "?").collect::<Vec<_>>().join(", ");
    format!(" AND m.vendor_code IN ({placeholders})")
}

pub fn normalized_search_pattern(query: &ListAdminAiModelsQuery) -> Option<String> {
    query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{value}%"))
}

pub fn normalized_vendor_codes(query: &ListAdminAiModelsQuery) -> Vec<String> {
    let mut values = query
        .vendor_codes
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

pub fn optional_non_empty(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub fn capability_codes_from_model_types(value: Option<&str>) -> Vec<i32> {
    value
        .map(|raw| {
            raw.split(',')
                .filter_map(|segment| {
                    let trimmed = segment.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(crate::model_modality::model_type_capability_code(trimmed))
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_codes_from_model_types_parses_admin_labels() {
        let codes = capability_codes_from_model_types(Some("Chat,Image,Embedding"));
        assert_eq!(codes, vec![1, 2, 6]);
    }

    #[test]
    fn normalized_vendor_codes_trims_sorts_and_deduplicates() {
        let query = ListAdminAiModelsQuery {
            subject: sdkwork_models_contract_service::AdminModelSubject {
                tenant_id: 1,
                organization_id: 0,
                operator_id: 1,
                operator_type: 1,
            },
            vendor_id: None,
            vendor_codes: vec![
                " OpenAI ".to_owned(),
                "anthropic".to_owned(),
                "openai".to_owned(),
            ],
            q: None,
            model_types: None,
            page_size: None,
            offset: None,
        };

        assert_eq!(
            normalized_vendor_codes(&query),
            vec!["anthropic".to_owned(), "openai".to_owned()]
        );
    }
}
