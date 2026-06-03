use sdkwork_models::validation::is_decimal_string;
use sdkwork_models::{
    find_meter, find_model, find_model_by_vendor_region, find_protocol, get_best_reference_price,
    get_model_prices, get_model_region_prices, list_available_models,
    list_client_api_compatibility_by_vendor, list_meters, list_models, list_models_by_capability,
    list_models_by_modality, list_models_by_protocol, list_protocols, list_protocols_by_vendor,
    list_vendor_regions, list_vendors, load_bundled_catalog, load_catalog, load_vendor_catalog,
    validate_catalog, ModelCatalog, ModelFilter,
};
use std::collections::BTreeSet;
use std::fs;

fn unsupported_client_api_compatibility_json() -> &'static str {
    r#""clientApiCompatibility":{
        "codex":{"clientApiCode":"codex","displayName":"Codex","supportStatus":"unsupported","protocolCodes":[],"apiCodes":[],"resourceCodes":[],"notes":"Test fixture vendor does not expose the Codex client API surface directly.","source":{"sourceUrl":"https://sdkwork.cloud/standards/sdkwork-models/client-api-compatibility","observedAt":"2026-06-03T00:00:00Z"}},
        "claude_code":{"clientApiCode":"claude_code","displayName":"Claude Code","supportStatus":"unsupported","protocolCodes":[],"apiCodes":[],"resourceCodes":[],"notes":"Test fixture vendor does not expose the Claude Code client API surface directly.","source":{"sourceUrl":"https://sdkwork.cloud/standards/sdkwork-models/client-api-compatibility","observedAt":"2026-06-03T00:00:00Z"}},
        "gemini_cli":{"clientApiCode":"gemini_cli","displayName":"Gemini CLI","supportStatus":"unsupported","protocolCodes":[],"apiCodes":[],"resourceCodes":[],"notes":"Test fixture vendor does not expose the Gemini CLI client API surface directly.","source":{"sourceUrl":"https://sdkwork.cloud/standards/sdkwork-models/client-api-compatibility","observedAt":"2026-06-03T00:00:00Z"}}
    }"#
}

#[test]
fn bundled_catalog_loads_and_queries_models() {
    let catalog = load_bundled_catalog().expect("catalog should load");

    assert_eq!("2026.05.08.1", catalog.manifest.catalog_version);
    assert!(catalog.vendors.len() >= 3);
    assert_eq!(
        Some("openai"),
        find_model(&catalog, "openai/gpt-5.5").map(|model| model.vendor_code.as_str())
    );
    assert_eq!(
        Some("global"),
        find_model(&catalog, "openai/gpt-5.5").map(|model| model.region_code.as_str())
    );
    assert_eq!(
        Some("openai"),
        find_model_by_vendor_region(&catalog, "openai", "global", "gpt-5.5")
            .map(|model| model.vendor_code.as_str())
    );
    assert!(find_model(&catalog, "openai/global/gpt-5.5").is_none());
    assert!(list_vendors(&catalog)
        .iter()
        .all(|vendor| vendor.vendor_code != "minimax" || vendor.legal_name.is_some()));
    assert!(list_vendor_regions(&catalog)
        .iter()
        .any(|region| region.vendor_code == "minimax" && region.region_code == "cn"));
    assert_eq!(
        1,
        list_vendors(&catalog)
            .iter()
            .filter(|vendor| vendor.vendor_code == "minimax")
            .count()
    );
    assert!(list_meters(&catalog)
        .iter()
        .any(|meter| meter.meter_code == "llm_input_token"));
    assert_eq!(
        Some("1000000"),
        find_meter(&catalog, "llm_input_token").map(|meter| meter.default_unit_size.as_str())
    );
    assert!(find_meter(&catalog, "missing_meter").is_none());
    assert!(!list_models(
        &catalog,
        ModelFilter {
            vendor_code: Some("openai"),
            region_code: Some("global"),
            family_code: Some("gpt-5"),
            ..ModelFilter::default()
        }
    )
    .is_empty());
    assert!(!list_models(
        &catalog,
        ModelFilter {
            release_stage: Some("active"),
            shelf_state: Some("listed"),
            routing_state: Some("enabled"),
            api_format: Some("openai_compatible"),
            ..ModelFilter::default()
        }
    )
    .is_empty());
    assert!(!list_models_by_capability(&catalog, "chat").is_empty());
    assert!(!list_models_by_modality(&catalog, "text", "text").is_empty());
    assert!(list_protocols(&catalog).len() >= 4);
    assert_eq!(
        Some("OpenAI Responses API"),
        find_protocol(&catalog, "openai_responses").map(|protocol| protocol.display_name.as_str())
    );
    assert!(find_protocol(&catalog, "missing_protocol").is_none());
    assert!(list_protocols_by_vendor(&catalog, "openai")
        .iter()
        .any(|protocol| protocol.protocol_code == "openai_responses"));
    assert!(list_models_by_protocol(&catalog, "openai_responses")
        .iter()
        .all(|model| model.api_format == "openai_responses"));
    assert!(list_vendors(&catalog)
        .iter()
        .any(|vendor| vendor.vendor_code == "openai"
            && vendor
                .supported_protocols
                .iter()
                .any(|protocol| protocol == "openai_responses")));
    assert!(list_client_api_compatibility_by_vendor(&catalog, "openai")
        .iter()
        .any(|item| item.client_api_code == "codex" && item.support_status == "supported"));
    let available_models = list_available_models(&catalog, ModelFilter::default());
    assert!(!available_models.is_empty());
    let model_keys = list_models(&catalog, ModelFilter::default())
        .into_iter()
        .map(|model| model.catalog_key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        model_keys.len(),
        model_keys.iter().collect::<BTreeSet<_>>().len()
    );
    assert!(available_models
        .iter()
        .all(|model| !sdkwork_models::get_model_prices(&catalog, &model.catalog_key).is_empty()));
    assert!(available_models
        .iter()
        .all(|model| model.routing_state == "enabled" && model.shelf_state == "listed"));
    assert!(available_models
        .iter()
        .any(|model| model.catalog_key == "kuaishou/kling-v3-0-preview"
            && model.region_code == "global"));
    assert_eq!(
        Some("global"),
        find_model(&catalog, "kuaishou/kling-v3-0-preview").map(|model| model.region_code.as_str())
    );
    assert_eq!(
        Some("cn"),
        find_model_by_vendor_region(&catalog, "kuaishou", "cn", "kling-v3-0-preview")
            .map(|model| model.region_code.as_str())
    );
    assert!(list_available_models(
        &catalog,
        ModelFilter {
            region_code: Some("cn"),
            ..ModelFilter::default()
        }
    )
    .iter()
    .all(|model| model.catalog_key != "kuaishou/kling-v3-0-preview"));
    assert!(list_available_models(
        &catalog,
        ModelFilter {
            region_code: Some("global"),
            ..ModelFilter::default()
        }
    )
    .iter()
    .any(|model| model.catalog_key == "kuaishou/kling-v3-0-preview"));
    assert!(!get_model_region_prices(&catalog, "openai/gpt-5.5", "global").is_empty());
    assert!(get_model_region_prices(&catalog, "openai/gpt-5.5", "cn").is_empty());
    assert_eq!(
        Some("5.000000"),
        get_best_reference_price(&catalog, "openai/gpt-5.5", "llm_input_token")
            .map(|price| price.unit_price.as_str())
    );
    assert!(
        get_best_reference_price(&catalog, "openai/global/gpt-5.5", "llm_input_token").is_none()
    );
    assert!(validate_catalog(&catalog).is_empty());
}

#[test]
fn decimal_string_validation_matches_catalog_schema() {
    assert!(is_decimal_string("0"));
    assert!(is_decimal_string("1"));
    assert!(is_decimal_string("1.0"));
    assert!(is_decimal_string("1000000.000001"));

    assert!(!is_decimal_string(""));
    assert!(!is_decimal_string("01"));
    assert!(!is_decimal_string("1."));
    assert!(!is_decimal_string(".1"));
    assert!(!is_decimal_string("-1"));
}

#[test]
fn catalog_key_parser_keeps_slash_delimited_provider_model_ids_intact() {
    let catalog: ModelCatalog = serde_json::from_str(
        r#"{
          "manifest": {
            "name": "sdkwork-models",
            "schemaVersion": "1.1.0",
            "catalogVersion": "2026.05.08.1",
            "generatedAt": "2026-06-02T00:00:00Z",
            "modelsRoot": "models",
            "schemasRoot": "schemas"
          },
          "meters": [],
          "protocols": [],
          "vendors": [
            {
              "vendorCode": "openrouter",
              "regionCode": "global",
              "vendor": {
                "vendorCode": "openrouter",
                "regionCode": "global",
                "displayName": "OpenRouter",
                "vendorType": "commercial",
                "marketScope": "global",
                "billingCurrency": "USD",
                "billingJurisdiction": "US",
                "operatingRegions": ["GLOBAL"],
                "capabilities": ["chat"],
                "supportedProtocols": ["openai_compatible"],
                "clientApiCompatibility": {
                  "codex":{"clientApiCode":"codex","displayName":"Codex","supportStatus":"unsupported","protocolCodes":[],"apiCodes":[],"resourceCodes":[],"notes":"Test fixture vendor does not expose the Codex client API surface directly.","source":{"sourceUrl":"https://sdkwork.cloud/standards/sdkwork-models/client-api-compatibility","observedAt":"2026-06-03T00:00:00Z"}},
                  "claude_code":{"clientApiCode":"claude_code","displayName":"Claude Code","supportStatus":"unsupported","protocolCodes":[],"apiCodes":[],"resourceCodes":[],"notes":"Test fixture vendor does not expose the Claude Code client API surface directly.","source":{"sourceUrl":"https://sdkwork.cloud/standards/sdkwork-models/client-api-compatibility","observedAt":"2026-06-03T00:00:00Z"}},
                  "gemini_cli":{"clientApiCode":"gemini_cli","displayName":"Gemini CLI","supportStatus":"unsupported","protocolCodes":[],"apiCodes":[],"resourceCodes":[],"notes":"Test fixture vendor does not expose the Gemini CLI client API surface directly.","source":{"sourceUrl":"https://sdkwork.cloud/standards/sdkwork-models/client-api-compatibility","observedAt":"2026-06-03T00:00:00Z"}}
                },
                "openSource": false,
                "source": {"sourceUrl": "https://openrouter.ai", "observedAt": "2026-06-02"}
              },
              "families": [],
              "models": [
                {
                  "catalogKey": "openrouter/anthropic/claude-3-opus",
                  "modelId": "anthropic/claude-3-opus",
                  "displayName": "Claude 3 Opus through OpenRouter",
                  "vendorCode": "openrouter",
                  "regionCode": "global",
                  "familyCode": "anthropic",
                  "primaryCapability": "chat",
                  "capabilities": ["chat"],
                  "inputModalities": ["text"],
                  "outputModalities": ["text"],
                  "apiFormat": "openai_compatible",
                  "lifecycle": "current",
                  "releaseStage": "active",
                  "shelfState": "listed",
                  "routingState": "enabled",
                  "source": {"sourceUrl": "https://openrouter.ai", "observedAt": "2026-06-02"}
                }
              ],
              "pricing": [
                {
                  "catalogKey": "openrouter/anthropic/claude-3-opus",
                  "vendorCode": "openrouter",
                  "regionCode": "global",
                  "modelId": "anthropic/claude-3-opus",
                  "currency": "USD",
                  "prices": [
                    {
                      "priceId": "openrouter-claude-opus-input",
                      "priceSide": "input",
                      "meterCode": "llm_input_token",
                      "unitSize": "1000000",
                      "unitPrice": "15.000000",
                      "minimumQuantity": "0",
                      "effectiveFrom": "2026-06-02",
                      "source": {"sourceUrl": "https://openrouter.ai", "observedAt": "2026-06-02"}
                    }
                  ]
                }
              ],
              "rankings": []
            }
          ]
        }"#,
    )
    .expect("slash-delimited model id catalog");

    assert_eq!(
        "anthropic/claude-3-opus",
        find_model(&catalog, "openrouter/anthropic/claude-3-opus")
            .map(|model| model.model_id.as_str())
            .unwrap()
    );
    assert_eq!(
        1,
        get_model_prices(&catalog, "openrouter/anthropic/claude-3-opus").len()
    );
    assert_eq!(
        1,
        get_model_region_prices(&catalog, "openrouter/anthropic/claude-3-opus", "global").len()
    );
    assert!(find_model(&catalog, "openrouter/global/anthropic/claude-3-opus").is_none());
    assert!(get_model_prices(&catalog, "openrouter/global/anthropic/claude-3-opus").is_empty());
}

#[test]
fn local_loader_uses_index_as_source_of_truth() {
    let temp_dir = tempfile::tempdir().expect("temp catalog root");
    let root = temp_dir.path();
    fs::create_dir_all(root.join("models/openai/global/models")).expect("models dir");
    fs::create_dir_all(root.join("models/openai/global/pricing")).expect("pricing dir");
    fs::create_dir_all(root.join("models/unlisted/global/models")).expect("unlisted models dir");
    fs::create_dir_all(root.join("models/unlisted/global/pricing")).expect("unlisted pricing dir");
    fs::write(
        root.join("sdkwork-models.json"),
        r#"{"name":"sdkwork-models","schemaVersion":"1.1.0","catalogVersion":"2026.05.08.1","generatedAt":"2026-05-08T00:00:00Z","modelsRoot":"models","schemasRoot":"schemas"}"#,
    )
    .expect("manifest");
    fs::write(
        root.join("models/meters.json"),
        r#"{"meters":[{"meterCode":"llm_input_token","displayName":"LLM input tokens","modality":"text","defaultUnitSize":"1000000"}]}"#,
    )
    .expect("meters");
    fs::write(
        root.join("models/protocols.json"),
        r#"{"protocols":[{"protocolCode":"openai_compatible","vendorOrigin":"openai","displayName":"OpenAI Chat Completions Compatible","family":"openai","docsUrl":"https://example.com","maturity":"stable"}]}"#,
    )
    .expect("protocols");
    fs::write(
        root.join("models/index.json"),
        r#"{"schemaVersion":"1.1.0","catalogVersion":"2026.05.08.1","generatedAt":"2026-05-08T00:00:00Z","vendorCount":1,"regionCount":1,"modelCount":1,"pricingFileCount":1,"vendors":[{"vendorCode":"openai","regionCode":"global","path":"openai/global/vendor.json","familiesPath":"openai/global/families.json","modelsPath":"openai/global/models","modelFiles":["openai/global/models/gpt-5.5.json"],"pricingPath":"openai/global/pricing","pricingFiles":["openai/global/pricing/gpt-5.5.json"],"rankingsPath":"openai/global/rankings.json","modelCount":1,"pricingFileCount":1,"rankingSnapshotCount":0,"sha256":"test"}]}"#,
    )
    .expect("index");
    fs::write(
        root.join("models/openai/global/vendor.json"),
        format!(
            r#"{{"vendorCode":"openai","regionCode":"global","displayName":"OpenAI","vendorType":"commercial","marketScope":"global","billingCurrency":"USD","billingJurisdiction":"US","operatingRegions":["GLOBAL"],"capabilities":["chat"],"supportedProtocols":["openai_compatible"],{},"source":{{"sourceUrl":"https://example.com","observedAt":"2026-05-08"}}}}"#,
            unsupported_client_api_compatibility_json()
        ),
    )
    .expect("vendor");
    fs::write(
        root.join("models/openai/global/families.json"),
        r#"{"vendorCode":"openai","regionCode":"global","families":[{"familyCode":"gpt-5","displayName":"GPT-5","familyType":"llm","primaryModality":"text"}]}"#,
    )
    .expect("families");
    fs::write(
        root.join("models/openai/global/rankings.json"),
        r#"{"vendorCode":"openai","regionCode":"global","snapshots":[]}"#,
    )
    .expect("rankings");
    fs::write(
        root.join("models/openai/global/models/gpt-5.5.json"),
        r#"{"catalogKey":"openai/gpt-5.5","modelId":"gpt-5.5","displayName":"GPT-5.5","vendorCode":"openai","regionCode":"global","familyCode":"gpt-5","primaryCapability":"chat","capabilities":["chat"],"inputModalities":["text"],"outputModalities":["text"],"apiFormat":"openai_compatible","lifecycle":"current","releaseStage":"active","shelfState":"listed","routingState":"enabled","source":{"sourceUrl":"https://example.com","observedAt":"2026-05-08"}}"#,
    )
    .expect("model");
    fs::write(
        root.join("models/openai/global/pricing/gpt-5.5.json"),
        r#"{"catalogKey":"openai/gpt-5.5","vendorCode":"openai","regionCode":"global","modelId":"gpt-5.5","currency":"USD","prices":[{"priceId":"gpt-5.5-input","priceSide":"input","meterCode":"llm_input_token","unitSize":"1000000","unitPrice":"5.000000","minimumQuantity":"0","effectiveFrom":"2026-05-08","source":{"sourceUrl":"https://example.com","observedAt":"2026-05-08"}}]}"#,
    )
    .expect("pricing");
    fs::write(
        root.join("models/unlisted/global/vendor.json"),
        format!(
            r#"{{"vendorCode":"unlisted","regionCode":"global","displayName":"Unlisted","vendorType":"commercial","marketScope":"global","billingCurrency":"USD","billingJurisdiction":"US","operatingRegions":["GLOBAL"],"capabilities":["chat"],"supportedProtocols":["openai_compatible"],{},"source":{{"sourceUrl":"https://example.com","observedAt":"2026-05-08"}}}}"#,
            unsupported_client_api_compatibility_json()
        ),
    )
    .expect("unlisted vendor");

    let catalog = load_catalog(root).expect("catalog should load from index");
    assert_eq!(1, catalog.vendors.len());
    assert!(find_model(&catalog, "openai/gpt-5.5").is_some());
    assert!(list_vendor_regions(&catalog)
        .iter()
        .all(|region| region.vendor_code != "unlisted"));
}

#[test]
fn direct_vendor_loader_reads_nested_provider_model_id_files() {
    let temp_dir = tempfile::tempdir().expect("temp vendor root");
    let root = temp_dir.path();
    fs::create_dir_all(root.join("models/anthropic")).expect("nested models dir");
    fs::create_dir_all(root.join("pricing/anthropic")).expect("nested pricing dir");
    fs::write(
        root.join("vendor.json"),
        format!(
            r#"{{"vendorCode":"openrouter","regionCode":"global","displayName":"OpenRouter","vendorType":"commercial","marketScope":"global","billingCurrency":"USD","billingJurisdiction":"US","operatingRegions":["GLOBAL"],"capabilities":["chat"],"supportedProtocols":["openai_compatible"],{},"source":{{"sourceUrl":"https://openrouter.ai","observedAt":"2026-06-02"}}}}"#,
            unsupported_client_api_compatibility_json()
        ),
    )
    .expect("vendor");
    fs::write(
        root.join("families.json"),
        r#"{"vendorCode":"openrouter","regionCode":"global","families":[{"familyCode":"anthropic","displayName":"Anthropic","familyType":"llm","primaryModality":"text"}]}"#,
    )
    .expect("families");
    fs::write(
        root.join("rankings.json"),
        r#"{"vendorCode":"openrouter","regionCode":"global","snapshots":[]}"#,
    )
    .expect("rankings");
    fs::write(
        root.join("models/anthropic/claude-3-opus.json"),
        r#"{"catalogKey":"openrouter/anthropic/claude-3-opus","modelId":"anthropic/claude-3-opus","displayName":"Claude 3 Opus through OpenRouter","vendorCode":"openrouter","regionCode":"global","familyCode":"anthropic","primaryCapability":"chat","capabilities":["chat"],"inputModalities":["text"],"outputModalities":["text"],"apiFormat":"openai_compatible","lifecycle":"current","releaseStage":"active","shelfState":"listed","routingState":"enabled","source":{"sourceUrl":"https://openrouter.ai","observedAt":"2026-06-02"}}"#,
    )
    .expect("model");
    fs::write(
        root.join("pricing/anthropic/claude-3-opus.json"),
        r#"{"catalogKey":"openrouter/anthropic/claude-3-opus","vendorCode":"openrouter","regionCode":"global","modelId":"anthropic/claude-3-opus","currency":"USD","prices":[{"priceId":"openrouter-claude-opus-input","priceSide":"input","meterCode":"llm_input_token","unitSize":"1000000","unitPrice":"15.000000","minimumQuantity":"0","effectiveFrom":"2026-06-02","source":{"sourceUrl":"https://openrouter.ai","observedAt":"2026-06-02"}}]}"#,
    )
    .expect("pricing");

    let catalog = load_vendor_catalog(root).expect("vendor catalog");

    assert_eq!(
        Some("anthropic/claude-3-opus"),
        catalog.models.first().map(|model| model.model_id.as_str())
    );
    assert_eq!(
        Some("anthropic/claude-3-opus"),
        catalog
            .pricing
            .first()
            .map(|pricing| pricing.model_id.as_str())
    );
}
