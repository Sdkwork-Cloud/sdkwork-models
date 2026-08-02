use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use sha2::{Digest, Sha256};

use sdkwork_models::{ClientApiCompatibility, ModelCatalog, ModelInfo, TtsVoice, VendorCatalog};

use sdkwork_models_contract_service::{
    AdminAiModelItem, AdminAiModelRegionPriceCommand, AdminModelSubject, AdminModelVendorItem,
};

pub(crate) const SYSTEM_TENANT_ID: i64 = 0;
pub(crate) const SYSTEM_ORGANIZATION_ID: i64 = 0;
pub(crate) const SYSTEM_DATA_SCOPE: i32 = 1;
pub(crate) const ACTIVE_STATUS: i32 = 1;
pub(crate) const INACTIVE_STATUS: i32 = 0;
pub(crate) const SYNC_MODE_DRY_RUN: &str = "dry_run";
const AI_RESOURCE_DESCRIPTION_MAX_CHARS: usize = 512;

pub(crate) fn pricing_catalog_key(vendor_code: &str, model_id: &str) -> String {
    model_catalog_key(vendor_code, model_id)
}

pub(crate) fn model_catalog_key(vendor_code: &str, model_id: &str) -> String {
    format!("{vendor_code}/{model_id}")
}

pub(crate) fn voice_catalog_key(vendor_code: &str, voice_id: &str) -> String {
    format!("{vendor_code}/{voice_id}")
}

pub(crate) fn catalog_identity_models(
    catalog: &ModelCatalog,
) -> BTreeMap<String, (&VendorCatalog, &ModelInfo)> {
    let mut models: BTreeMap<String, (&VendorCatalog, &ModelInfo)> = BTreeMap::new();
    for vendor in &catalog.vendors {
        for model in &vendor.models {
            let key = model_catalog_key(&model.vendor_code, &model.model_id);
            let replace = models
                .get(&key)
                .map(|(existing_vendor, existing_model)| {
                    model_identity_score(vendor, model)
                        > model_identity_score(existing_vendor, existing_model)
                })
                .unwrap_or(true);
            if replace {
                models.insert(key, (vendor, model));
            }
        }
    }
    models
}

pub fn public_catalog_identity_models(
    catalog: &ModelCatalog,
) -> BTreeMap<String, (&VendorCatalog, &ModelInfo)> {
    catalog_identity_models(catalog)
        .into_iter()
        .filter(|(_, (_, model))| sdkwork_model_is_publicly_active(model))
        .collect()
}

fn model_identity_score(vendor: &VendorCatalog, model: &ModelInfo) -> i32 {
    let has_region_pricing = vendor
        .pricing
        .iter()
        .any(|pricing| pricing.model_id == model.model_id && !pricing.prices.is_empty());
    let mut score = 0;
    if has_region_pricing {
        score += 100;
    }
    if model.routing_state == "enabled" {
        score += 40;
    }
    if model.shelf_state == "listed" {
        score += 20;
    }
    if model.release_stage == "active" {
        score += 10;
    }
    if matches!(model.lifecycle.as_str(), "current" | "preview") {
        score += 5;
    }
    if vendor.region_code == "global" {
        score += 1;
    }
    score
}

pub(crate) fn sdkwork_model_is_publicly_active(model: &ModelInfo) -> bool {
    matches!(model.release_stage.as_str(), "active" | "preview")
        && model.shelf_state == "listed"
        && model.routing_state == "enabled"
        && !matches!(
            model.lifecycle.as_str(),
            "deprecated" | "catalog_only" | "retired"
        )
}

pub(crate) fn sdkwork_voice_is_publicly_active(voice: &TtsVoice) -> bool {
    matches!(voice.release_stage.as_str(), "active" | "preview")
        && voice.shelf_state == "listed"
        && voice.routing_state == "enabled"
        && !matches!(
            voice.lifecycle.as_str(),
            "deprecated" | "catalog_only" | "retired"
        )
}

pub(crate) fn voice_catalog_status(voice: &TtsVoice) -> i32 {
    if sdkwork_voice_is_publicly_active(voice) {
        ACTIVE_STATUS
    } else {
        INACTIVE_STATUS
    }
}

pub(crate) fn catalog_model_status(model: &ModelInfo) -> i32 {
    if sdkwork_model_is_publicly_active(model) {
        ACTIVE_STATUS
    } else {
        INACTIVE_STATUS
    }
}

#[derive(Debug)]
pub(crate) enum CatalogImportError {
    Catalog(sdkwork_models::CatalogError),
    CatalogVersionMismatch { expected: String, actual: String },
    UnknownVendors(Vec<String>),
}

impl Display for CatalogImportError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Catalog(error) => write!(formatter, "{error}"),
            Self::CatalogVersionMismatch { expected, actual } => write!(
                formatter,
                "sdkwork-models catalog version mismatch: expected {expected}, loaded {actual}"
            ),
            Self::UnknownVendors(vendors) => {
                write!(
                    formatter,
                    "sdkwork-models catalog does not define vendor(s): {}",
                    vendors.join(", ")
                )
            }
        }
    }
}

impl Error for CatalogImportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Catalog(error) => Some(error),
            Self::CatalogVersionMismatch { .. } | Self::UnknownVendors(_) => None,
        }
    }
}

impl From<sdkwork_models::CatalogError> for CatalogImportError {
    fn from(value: sdkwork_models::CatalogError) -> Self {
        Self::Catalog(value)
    }
}

pub(crate) fn load_catalog_root_with_pin(
    catalog_root: Option<&str>,
    catalog_version: Option<&str>,
) -> Result<ModelCatalog, CatalogImportError> {
    let catalog = match catalog_root
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(root) => sdkwork_models::load_catalog(root)?,
        None => sdkwork_models::load_bundled_catalog()?,
    };
    validate_catalog_version_pin(&catalog, catalog_version)?;
    Ok(catalog)
}

pub(crate) fn catalog_with_selected_vendors(
    catalog: &ModelCatalog,
    vendor_codes: &[String],
) -> Result<ModelCatalog, CatalogImportError> {
    let requested = normalized_vendor_set(vendor_codes);
    if requested.is_empty() {
        return Ok(catalog.clone());
    }

    let available = catalog
        .vendors
        .iter()
        .map(|vendor| vendor.vendor.vendor_code.clone())
        .collect::<BTreeSet<_>>();
    let missing = requested
        .iter()
        .filter(|vendor_code| !available.contains(*vendor_code))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(CatalogImportError::UnknownVendors(missing));
    }

    Ok(ModelCatalog {
        manifest: catalog.manifest.clone(),
        meters: catalog.meters.clone(),
        protocols: catalog.protocols.clone(),
        vendors: catalog
            .vendors
            .iter()
            .filter(|vendor| requested.contains(&vendor.vendor.vendor_code))
            .cloned()
            .collect(),
    })
}

pub(crate) fn catalog_scope_vendor_codes(catalog: &ModelCatalog) -> Vec<String> {
    catalog_vendor_records(catalog)
        .into_iter()
        .map(|vendor| vendor.vendor_code)
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CatalogVendorRecord {
    pub vendor_code: String,
    pub display_name: String,
    pub legal_name: Option<String>,
    pub description: Option<String>,
    pub website_url: Option<String>,
    pub docs_url: Option<String>,
    pub country_region: Option<String>,
    pub vendor_type: String,
    pub model_families: Vec<String>,
    pub capabilities: Vec<String>,
    pub supported_protocols: Vec<String>,
    pub client_api_compatibility: BTreeMap<String, ClientApiCompatibility>,
    pub open_source: bool,
    pub sort_order: i32,
    pub source_url: String,
}

pub(crate) fn catalog_vendor_records(catalog: &ModelCatalog) -> Vec<CatalogVendorRecord> {
    let mut vendors = BTreeMap::<String, CatalogVendorRecord>::new();
    for region_catalog in &catalog.vendors {
        let vendor = &region_catalog.vendor;
        let record = vendors
            .entry(vendor.vendor_code.clone())
            .or_insert_with(|| CatalogVendorRecord {
                vendor_code: vendor.vendor_code.clone(),
                display_name: vendor.display_name.clone(),
                legal_name: vendor.legal_name.clone(),
                description: vendor.description.clone(),
                website_url: vendor.website_url.clone(),
                docs_url: vendor.docs_url.clone(),
                country_region: vendor.country_region.clone(),
                vendor_type: vendor.vendor_type.clone(),
                model_families: Vec::new(),
                capabilities: Vec::new(),
                supported_protocols: Vec::new(),
                client_api_compatibility: BTreeMap::new(),
                open_source: vendor.open_source.unwrap_or(false),
                sort_order: vendor.sort_order.unwrap_or(1_000_000),
                source_url: vendor.source.source_url.clone(),
            });
        append_unique(&mut record.model_families, vendor.model_families.iter());
        append_unique(&mut record.capabilities, vendor.capabilities.iter());
        append_unique(
            &mut record.supported_protocols,
            vendor.supported_protocols.iter(),
        );
        for (client_api_code, compatibility) in &vendor.client_api_compatibility {
            match record.client_api_compatibility.get(client_api_code) {
                Some(existing)
                    if client_api_support_rank(&existing.support_status)
                        >= client_api_support_rank(&compatibility.support_status) => {}
                _ => {
                    record
                        .client_api_compatibility
                        .insert(client_api_code.clone(), compatibility.clone());
                }
            }
        }
    }
    vendors.into_values().collect()
}

fn append_unique<'a>(target: &mut Vec<String>, values: impl IntoIterator<Item = &'a String>) {
    for value in values {
        if !target.contains(value) {
            target.push(value.clone());
        }
    }
}

fn client_api_support_rank(value: &str) -> i32 {
    match value {
        "supported" => 3,
        "compatible" => 2,
        "unsupported" => 1,
        _ => 0,
    }
}

pub(crate) fn catalog_scope_model_count(catalog: &ModelCatalog) -> usize {
    catalog_identity_models(catalog).len()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CatalogScopeCounts {
    pub meter_count: usize,
    pub vendor_count: usize,
    pub family_count: usize,
    pub model_count: usize,
    pub capability_count: usize,
    pub price_count: usize,
    pub ranking_count: usize,
    pub voice_count: usize,
    pub voice_binding_count: usize,
    pub video_profile_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogAuthorityKeys {
    pub vendor_codes: Vec<String>,
    pub catalog_keys: Vec<String>,
    pub family_uuids: Vec<String>,
    pub capability_uuids: Vec<String>,
    pub price_uuids: Vec<String>,
    pub ranking_uuids: Vec<String>,
    pub voice_uuids: Vec<String>,
    pub voice_binding_uuids: Vec<String>,
    pub video_profile_uuids: Vec<String>,
    pub vendor_modality_uuids: Vec<String>,
    pub vendor_api_endpoint_uuids: Vec<String>,
    pub model_modality_uuids: Vec<String>,
    pub model_api_endpoint_uuids: Vec<String>,
    pub ai_resource_codes: Vec<String>,
}

impl CatalogScopeCounts {
    pub fn accepted_count(self) -> i64 {
        (self.meter_count
            + self.vendor_count
            + self.family_count
            + self.model_count
            + self.capability_count
            + self.price_count
            + self.ranking_count
            + self.voice_count
            + self.voice_binding_count
            + self.video_profile_count) as i64
    }
}

pub(crate) fn catalog_scope_counts(catalog: &ModelCatalog) -> CatalogScopeCounts {
    let identity_models = public_catalog_identity_models(catalog);
    let model_catalog_keys = identity_models.keys().cloned().collect::<BTreeSet<_>>();
    let capability_count = identity_models
        .values()
        .flat_map(|(_, model)| {
            let capabilities = if model.capabilities.is_empty() {
                vec![model.primary_capability.clone()]
            } else {
                model.capabilities.clone()
            };
            capabilities.into_iter().map(move |capability| {
                model_catalog_key(&model.vendor_code, &model.model_id) + "/" + &capability
            })
        })
        .collect::<BTreeSet<_>>()
        .len();
    let public_model_keys = public_catalog_identity_models(catalog)
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let price_count = catalog
        .vendors
        .iter()
        .flat_map(|vendor| vendor.pricing.iter())
        .filter(|pricing| {
            public_model_keys.contains(&model_catalog_key(&pricing.vendor_code, &pricing.model_id))
        })
        .map(|pricing| pricing.prices.len())
        .sum();
    let ranking_count = catalog
        .vendors
        .iter()
        .flat_map(|vendor| {
            let vendor_code = vendor.vendor.vendor_code.as_str();
            vendor.rankings.iter().flat_map(move |snapshot| {
                snapshot.items.iter().map(move |item| {
                    (
                        model_catalog_key(vendor_code, &item.model_id),
                        pricing_catalog_key(vendor_code, &item.model_id),
                    )
                })
            })
        })
        .filter(|(model_catalog_key, _)| model_catalog_keys.contains(model_catalog_key))
        .count();
    let voice_count = catalog
        .vendors
        .iter()
        .map(|vendor| vendor.voices.len())
        .sum();
    let voice_binding_count = catalog
        .vendors
        .iter()
        .flat_map(|vendor| vendor.model_voice_bindings.iter())
        .map(|binding| binding.bindings.len())
        .sum();
    CatalogScopeCounts {
        meter_count: catalog.meters.len(),
        vendor_count: catalog_scope_vendor_codes(catalog).len(),
        family_count: catalog
            .vendors
            .iter()
            .flat_map(|vendor| {
                vendor.families.iter().map(|family| {
                    (
                        vendor.vendor.vendor_code.clone(),
                        family.family_code.clone(),
                    )
                })
            })
            .collect::<BTreeSet<_>>()
            .len(),
        model_count: catalog_scope_model_count(catalog),
        capability_count,
        price_count,
        ranking_count,
        voice_count,
        voice_binding_count,
        video_profile_count: catalog
            .vendors
            .iter()
            .flat_map(|vendor| vendor.model_video_profiles.iter())
            .map(|file| file.profiles.len())
            .sum(),
    }
}

pub(crate) fn catalog_authority_keys(catalog: &ModelCatalog) -> CatalogAuthorityKeys {
    let vendor_codes = catalog
        .vendors
        .iter()
        .map(|vendor| vendor.vendor.vendor_code.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let catalog_keys = catalog_identity_models(catalog)
        .keys()
        .cloned()
        .into_iter()
        .collect::<Vec<_>>();
    let public_catalog_keys = public_catalog_identity_models(catalog)
        .keys()
        .cloned()
        .into_iter()
        .collect::<Vec<_>>();
    let model_catalog_key_set = public_catalog_keys.iter().cloned().collect::<BTreeSet<_>>();
    let family_uuids = catalog
        .vendors
        .iter()
        .flat_map(|vendor| {
            vendor.families.iter().map(|family| {
                stable_uuid(
                    "sdk-family",
                    &[&vendor.vendor.vendor_code, &family.family_code],
                )
            })
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let capability_uuids = public_catalog_identity_models(catalog)
        .values()
        .flat_map(|(_, model)| {
            let capabilities = if model.capabilities.is_empty() {
                vec![model.primary_capability.clone()]
            } else {
                model.capabilities.clone()
            };
            capabilities.into_iter().map(move |capability| {
                stable_uuid(
                    "sdk-cap",
                    &[&model.vendor_code, &model.model_id, &capability],
                )
            })
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let price_uuids = catalog
        .vendors
        .iter()
        .flat_map(|vendor| {
            vendor.pricing.iter().flat_map(|pricing| {
                pricing.prices.iter().map(|price| {
                    (
                        model_catalog_key(&pricing.vendor_code, &pricing.model_id),
                        stable_uuid(
                            "sdk-price",
                            &[
                                &pricing.vendor_code,
                                &pricing.region_code,
                                &pricing.model_id,
                                &price.price_id,
                            ],
                        ),
                    )
                })
            })
        })
        .filter_map(|(catalog_key, uuid)| {
            if model_catalog_key_set.contains(&catalog_key) {
                Some(uuid)
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let ranking_uuids = catalog
        .vendors
        .iter()
        .flat_map(|vendor| {
            let vendor_code = vendor.vendor.vendor_code.clone();
            let region_code = vendor.vendor.region_code.clone();
            let model_catalog_key_set = model_catalog_key_set.clone();
            vendor.rankings.iter().flat_map(move |snapshot| {
                let vendor_code = vendor_code.clone();
                let region_code = region_code.clone();
                let model_catalog_key_set = model_catalog_key_set.clone();
                snapshot.items.iter().filter_map(move |item| {
                    let model_catalog_key = model_catalog_key(&vendor_code, &item.model_id);
                    if model_catalog_key_set.contains(&model_catalog_key) {
                        Some(stable_uuid(
                            "sdk-rank",
                            &[
                                &snapshot.snapshot_date,
                                &snapshot.rank_scope,
                                &vendor_code,
                                &region_code,
                                &item.model_id,
                            ],
                        ))
                    } else {
                        None
                    }
                })
            })
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let voice_uuids = catalog
        .vendors
        .iter()
        .flat_map(|vendor| {
            vendor
                .voices
                .iter()
                .map(|voice| stable_uuid("sdk-voice", &[&voice.vendor_code, &voice.voice_id]))
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let voice_binding_uuids = catalog
        .vendors
        .iter()
        .flat_map(|vendor| {
            vendor.model_voice_bindings.iter().flat_map(|binding_file| {
                binding_file.bindings.iter().map(|binding| {
                    stable_uuid(
                        "sdk-voice-bind",
                        &[&binding_file.catalog_key, &binding.voice_key],
                    )
                })
            })
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let video_profile_uuids = catalog
        .vendors
        .iter()
        .flat_map(|vendor| {
            vendor.model_video_profiles.iter().flat_map(|profile_file| {
                profile_file.profiles.iter().map(|profile| {
                    stable_uuid(
                        "sdk-video-profile",
                        &[&profile_file.catalog_key, &profile.profile_code],
                    )
                })
            })
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    CatalogAuthorityKeys {
        vendor_codes,
        catalog_keys,
        family_uuids,
        capability_uuids,
        price_uuids,
        ranking_uuids,
        voice_uuids,
        voice_binding_uuids,
        video_profile_uuids,
        vendor_modality_uuids: catalog_vendor_modality_projections(catalog)
            .into_iter()
            .map(|item| item.uuid)
            .collect(),
        vendor_api_endpoint_uuids: catalog_vendor_api_endpoint_projections(catalog)
            .into_iter()
            .map(|item| item.uuid)
            .collect(),
        model_modality_uuids: catalog_model_modality_projections(catalog)
            .into_iter()
            .map(|item| item.uuid)
            .collect(),
        model_api_endpoint_uuids: catalog_model_api_endpoint_projections(catalog)
            .into_iter()
            .map(|item| item.uuid)
            .collect(),
        ai_resource_codes: catalog_ai_resource_projections(catalog)
            .into_iter()
            .map(|item| item.resource_code)
            .collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogModalityProjection {
    pub uuid: String,
    pub modality_code: String,
    pub display_name: String,
    pub modality_group: String,
    pub description: String,
    pub input_supported: bool,
    pub output_supported: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogApiEndpointProjection {
    pub uuid: String,
    pub endpoint_code: String,
    pub protocol_code: String,
    pub display_name: String,
    pub method: String,
    pub path_template: String,
    pub streaming_supported: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogVendorModalityProjection {
    pub uuid: String,
    pub vendor_code: String,
    pub modality_code: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogVendorApiEndpointProjection {
    pub uuid: String,
    pub vendor_code: String,
    pub endpoint_code: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogModalityApiEndpointProjection {
    pub uuid: String,
    pub modality_code: String,
    pub endpoint_code: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogModelModalityProjection {
    pub uuid: String,
    pub catalog_key: String,
    pub model: String,
    pub vendor_code: String,
    pub modality_code: String,
    pub direction: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogModelApiEndpointProjection {
    pub uuid: String,
    pub catalog_key: String,
    pub model: String,
    pub vendor_code: String,
    pub endpoint_code: String,
    pub provider_native_model: String,
    pub default_parameters: String,
    pub supports_streaming: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogAiResourceProjection {
    pub uuid: String,
    pub resource_code: String,
    pub resource_kind: String,
    pub display_name: String,
    pub vendor_code: Option<String>,
    pub modality_code: Option<String>,
    pub api_endpoint_code: Option<String>,
    pub catalog_key: Option<String>,
    pub model: Option<String>,
    pub provider_native_model: Option<String>,
    pub composition_mode: String,
    pub capability_schema: String,
    pub metadata_schema: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

pub(crate) fn catalog_modality_projections(
    catalog: &ModelCatalog,
) -> Vec<CatalogModalityProjection> {
    let mut usage = public_catalog_identity_models(catalog)
        .into_values()
        .map(|(_, model)| model)
        .fold(
            BTreeMap::<String, (bool, bool)>::new(),
            |mut usage, model| {
                for modality in &model.input_modalities {
                    let entry = usage.entry(modality.clone()).or_insert((false, false));
                    entry.0 = true;
                }
                for modality in &model.output_modalities {
                    let entry = usage.entry(modality.clone()).or_insert((false, false));
                    entry.1 = true;
                }
                usage
                    .entry(model.primary_capability.clone())
                    .or_insert((true, true));
                usage
            },
        );
    for meter in &catalog.meters {
        usage.entry(meter.modality.clone()).or_insert((true, true));
    }
    usage
        .into_iter()
        .enumerate()
        .map(
            |(index, (modality_code, (input_supported, output_supported)))| {
                CatalogModalityProjection {
                    uuid: stable_uuid("sdk-modality", &[&modality_code]),
                    display_name: modality_display_name(&modality_code),
                    modality_group: modality_group(&modality_code).to_owned(),
                    description: modality_description(&modality_code),
                    sort_order: modality_sort_order(&modality_code)
                        .unwrap_or((index as i32) + 1000),
                    modality_code,
                    input_supported,
                    output_supported,
                }
            },
        )
        .collect()
}

pub(crate) fn catalog_api_endpoint_projections(
    catalog: &ModelCatalog,
) -> Vec<CatalogApiEndpointProjection> {
    public_catalog_identity_models(catalog)
        .into_values()
        .map(|(_, model)| model)
        .map(model_endpoint_descriptor)
        .map(|descriptor| (descriptor.endpoint_code.to_owned(), descriptor))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .map(|descriptor| CatalogApiEndpointProjection {
            uuid: stable_uuid("sdk-api-endpoint", &[descriptor.endpoint_code]),
            endpoint_code: descriptor.endpoint_code.to_owned(),
            protocol_code: descriptor.protocol_code.to_owned(),
            display_name: descriptor.display_name.to_owned(),
            method: descriptor.method.to_owned(),
            path_template: descriptor.path_template.to_owned(),
            streaming_supported: descriptor.streaming_supported,
            sort_order: descriptor.sort_order,
        })
        .collect()
}

pub(crate) fn catalog_vendor_modality_projections(
    catalog: &ModelCatalog,
) -> Vec<CatalogVendorModalityProjection> {
    public_catalog_identity_models(catalog)
        .into_values()
        .flat_map(|(_, model)| {
            model_modality_codes(model)
                .into_iter()
                .map(move |modality_code| (model.vendor_code.clone(), modality_code))
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .map(
            |(index, (vendor_code, modality_code))| CatalogVendorModalityProjection {
                uuid: stable_uuid("sdk-vendor-modality", &[&vendor_code, &modality_code]),
                vendor_code,
                modality_code,
                sort_order: (index as i32) + 1,
            },
        )
        .collect()
}

pub(crate) fn catalog_vendor_api_endpoint_projections(
    catalog: &ModelCatalog,
) -> Vec<CatalogVendorApiEndpointProjection> {
    public_catalog_identity_models(catalog)
        .into_values()
        .map(|(_, model)| {
            (
                model.vendor_code.clone(),
                model_endpoint_descriptor(model).endpoint_code.to_owned(),
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .map(
            |(index, (vendor_code, endpoint_code))| CatalogVendorApiEndpointProjection {
                uuid: stable_uuid("sdk-vendor-endpoint", &[&vendor_code, &endpoint_code]),
                vendor_code,
                endpoint_code,
                sort_order: (index as i32) + 1,
            },
        )
        .collect()
}

pub(crate) fn catalog_modality_api_endpoint_projections(
    catalog: &ModelCatalog,
) -> Vec<CatalogModalityApiEndpointProjection> {
    public_catalog_identity_models(catalog)
        .into_values()
        .map(|(_, model)| model)
        .flat_map(|model| {
            let endpoint_code = model_endpoint_descriptor(model).endpoint_code.to_owned();
            model_endpoint_modalities(model)
                .into_iter()
                .map(move |modality_code| (modality_code, endpoint_code.clone()))
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .map(
            |(index, (modality_code, endpoint_code))| CatalogModalityApiEndpointProjection {
                uuid: stable_uuid("sdk-modality-endpoint", &[&modality_code, &endpoint_code]),
                modality_code,
                endpoint_code,
                sort_order: (index as i32) + 1,
            },
        )
        .collect()
}

pub(crate) fn catalog_model_modality_projections(
    catalog: &ModelCatalog,
) -> Vec<CatalogModelModalityProjection> {
    public_catalog_identity_models(catalog)
        .into_iter()
        .flat_map(|(catalog_key, (_, model))| {
            model_modality_directions(model)
                .into_iter()
                .map(
                    move |(modality_code, direction)| CatalogModelModalityProjection {
                        uuid: stable_uuid(
                            "sdk-model-modality",
                            &[&catalog_key, &modality_code, &direction],
                        ),
                        catalog_key: catalog_key.clone(),
                        model: model.model_id.clone(),
                        vendor_code: model.vendor_code.clone(),
                        modality_code,
                        direction,
                        sort_order: 1,
                    },
                )
        })
        .collect()
}

pub(crate) fn catalog_model_api_endpoint_projections(
    catalog: &ModelCatalog,
) -> Vec<CatalogModelApiEndpointProjection> {
    public_catalog_identity_models(catalog)
        .into_iter()
        .enumerate()
        .map(|(index, (catalog_key, (_, model)))| {
            let endpoint = model_endpoint_descriptor(model);
            CatalogModelApiEndpointProjection {
                uuid: stable_uuid(
                    "sdk-model-endpoint",
                    &[&catalog_key, endpoint.endpoint_code],
                ),
                catalog_key,
                model: model.model_id.clone(),
                vendor_code: model.vendor_code.clone(),
                endpoint_code: endpoint.endpoint_code.to_owned(),
                provider_native_model: model.model_id.clone(),
                default_parameters: "{}".to_owned(),
                supports_streaming: model.supports_streaming,
                sort_order: (index as i32) + 1,
            }
        })
        .collect()
}

pub(crate) fn catalog_ai_resource_projections(
    catalog: &ModelCatalog,
) -> Vec<CatalogAiResourceProjection> {
    let mut resources = BTreeMap::<String, CatalogAiResourceProjection>::new();
    for (index, vendor) in catalog
        .vendors
        .iter()
        .map(|vendor| &vendor.vendor)
        .map(|vendor| (vendor.vendor_code.clone(), vendor))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .enumerate()
    {
        let resource_code = format!("vendor.{}", vendor.vendor_code);
        resources.insert(
            resource_code.clone(),
            CatalogAiResourceProjection {
                uuid: stable_uuid("sdk-cap-resource", &[&resource_code]),
                resource_code,
                resource_kind: "vendor".to_owned(),
                display_name: vendor.display_name.clone(),
                vendor_code: Some(vendor.vendor_code.clone()),
                modality_code: None,
                api_endpoint_code: None,
                catalog_key: None,
                model: None,
                provider_native_model: None,
                composition_mode: "single".to_owned(),
                capability_schema: "{}".to_owned(),
                metadata_schema: "{}".to_owned(),
                description: ai_resource_description(vendor.description.clone()),
                sort_order: (index as i32) + 1,
            },
        );
    }

    for (index, modality) in catalog_modality_resource_projections(catalog)
        .into_iter()
        .enumerate()
    {
        let resource_code = format!("modality.{}", modality.modality_code);
        resources.insert(
            resource_code.clone(),
            CatalogAiResourceProjection {
                uuid: stable_uuid("sdk-cap-resource", &[&resource_code]),
                resource_code,
                resource_kind: "modality".to_owned(),
                display_name: modality.display_name,
                vendor_code: None,
                modality_code: Some(modality.modality_code),
                api_endpoint_code: None,
                catalog_key: None,
                model: None,
                provider_native_model: None,
                composition_mode: "single".to_owned(),
                capability_schema: serde_json::json!({
                    "modalityGroup": modality.modality_group,
                    "inputSupported": modality.input_supported,
                    "outputSupported": modality.output_supported
                })
                .to_string(),
                metadata_schema: "{}".to_owned(),
                description: ai_resource_description(Some(modality.description)),
                sort_order: 5_000 + modality.sort_order + (index as i32),
            },
        );
    }

    for endpoint in catalog_api_endpoint_projections(catalog) {
        let resource_code = format!("api.{}", endpoint.endpoint_code);
        resources.insert(
            resource_code.clone(),
            CatalogAiResourceProjection {
                uuid: stable_uuid("sdk-cap-resource", &[&resource_code]),
                resource_code,
                resource_kind: "api_endpoint".to_owned(),
                display_name: endpoint.display_name,
                vendor_code: endpoint_vendor_code(&endpoint.endpoint_code),
                modality_code: endpoint_modality_code(&endpoint.endpoint_code),
                api_endpoint_code: Some(endpoint.endpoint_code),
                catalog_key: None,
                model: None,
                provider_native_model: None,
                composition_mode: "single".to_owned(),
                capability_schema: "{}".to_owned(),
                metadata_schema: "{}".to_owned(),
                description: ai_resource_description(Some(
                    "Model catalog API endpoint capability".to_owned(),
                )),
                sort_order: 10_000 + endpoint.sort_order,
            },
        );
    }

    for (index, (catalog_key, (_, model))) in public_catalog_identity_models(catalog)
        .into_iter()
        .enumerate()
    {
        let endpoint = model_endpoint_descriptor(model);
        let modality_code = model_resource_suffix(model);
        let resource_code = format!(
            "model.{}.{}.{}",
            model.vendor_code, model.model_id, modality_code
        );
        resources.insert(
            resource_code.clone(),
            CatalogAiResourceProjection {
                uuid: stable_uuid("sdk-cap-resource", &[&resource_code]),
                resource_code,
                resource_kind: "model_api".to_owned(),
                display_name: if model.display_name.trim().is_empty() {
                    model.model_id.clone()
                } else {
                    model.display_name.clone()
                },
                vendor_code: Some(model.vendor_code.clone()),
                modality_code: Some(modality_code),
                api_endpoint_code: Some(endpoint.endpoint_code.to_owned()),
                catalog_key: Some(catalog_key),
                model: Some(model.model_id.clone()),
                provider_native_model: Some(model.model_id.clone()),
                composition_mode: "single".to_owned(),
                capability_schema: serde_json::json!({
                    "capability": &model.primary_capability,
                    "capabilities": if model.capabilities.is_empty() {
                        vec![model.primary_capability.clone()]
                    } else {
                        model.capabilities.clone()
                    },
                    "inputModalities": &model.input_modalities,
                    "outputModalities": &model.output_modalities,
                    "apiFormat": &model.api_format,
                    "supportsStreaming": model.supports_streaming,
                    "supportsTools": model.supports_tools,
                    "supportsJsonSchema": model.supports_json_schema
                })
                .to_string(),
                metadata_schema: "{}".to_owned(),
                description: ai_resource_description(model.description.clone()),
                sort_order: 20_000 + (index as i32) + 1,
            },
        );
    }

    resources.into_values().collect()
}

fn ai_resource_description(description: Option<String>) -> Option<String> {
    description.map(|mut value| {
        if let Some((byte_index, _)) = value.char_indices().nth(AI_RESOURCE_DESCRIPTION_MAX_CHARS) {
            value.truncate(byte_index);
        }
        value
    })
}

fn catalog_modality_resource_projections(catalog: &ModelCatalog) -> Vec<CatalogModalityProjection> {
    let mut modalities = catalog_modality_projections(catalog);
    if modalities.iter().any(|modality| {
        matches!(
            modality.modality_code.as_str(),
            "chat" | "text" | "embedding" | "rerank"
        )
    }) && !modalities
        .iter()
        .any(|modality| modality.modality_code == "llm")
    {
        modalities.push(CatalogModalityProjection {
            uuid: stable_uuid("sdk-modality", &["llm"]),
            modality_code: "llm".to_owned(),
            display_name: modality_display_name("llm"),
            modality_group: modality_group("llm").to_owned(),
            input_supported: true,
            output_supported: true,
            description: modality_description("llm"),
            sort_order: modality_sort_order("llm").unwrap_or(5),
        });
    }
    modalities
}

pub(crate) fn model_resource_suffix(model: &ModelInfo) -> String {
    if model.primary_capability == "chat" {
        "chat".to_owned()
    } else {
        model.primary_capability.clone()
    }
}

#[derive(Debug, Clone, Copy)]
struct EndpointDescriptor {
    endpoint_code: &'static str,
    protocol_code: &'static str,
    display_name: &'static str,
    method: &'static str,
    path_template: &'static str,
    streaming_supported: bool,
    sort_order: i32,
}

fn model_endpoint_descriptor(model: &ModelInfo) -> EndpointDescriptor {
    match model.primary_capability.as_str() {
        "image" => EndpointDescriptor {
            endpoint_code: "openai.images",
            protocol_code: "openai_compatible",
            display_name: "OpenAI Images",
            method: "POST",
            path_template: "/v1/images/generations",
            streaming_supported: false,
            sort_order: 30,
        },
        "audio" => EndpointDescriptor {
            endpoint_code: "openai.audio",
            protocol_code: "openai_compatible",
            display_name: "OpenAI Audio",
            method: "POST",
            path_template: "/v1/audio",
            streaming_supported: true,
            sort_order: 40,
        },
        "music" => EndpointDescriptor {
            endpoint_code: "suno.music",
            protocol_code: "vendor_native",
            display_name: "Suno Music",
            method: "POST",
            path_template: "/v1/music",
            streaming_supported: false,
            sort_order: 50,
        },
        "video" => EndpointDescriptor {
            endpoint_code: "openai.video",
            protocol_code: "openai_compatible",
            display_name: "Video Generation",
            method: "POST",
            path_template: "/v1/videos",
            streaming_supported: false,
            sort_order: 60,
        },
        "embedding" => EndpointDescriptor {
            endpoint_code: "openai.embeddings",
            protocol_code: "openai_compatible",
            display_name: "OpenAI Embeddings",
            method: "POST",
            path_template: "/v1/embeddings",
            streaming_supported: false,
            sort_order: 20,
        },
        "rerank" => EndpointDescriptor {
            endpoint_code: "rerank",
            protocol_code: "vendor_native",
            display_name: "Rerank",
            method: "POST",
            path_template: "/v1/rerank",
            streaming_supported: false,
            sort_order: 70,
        },
        _ if model.api_format == "openai_responses" => EndpointDescriptor {
            endpoint_code: "openai.chat_completions",
            protocol_code: "openai_compatible",
            display_name: "OpenAI Chat Completions",
            method: "POST",
            path_template: "/v1/chat/completions",
            streaming_supported: model.supports_streaming,
            sort_order: 10,
        },
        _ => EndpointDescriptor {
            endpoint_code: "openai.chat_completions",
            protocol_code: "openai_compatible",
            display_name: "OpenAI Chat Completions",
            method: "POST",
            path_template: "/v1/chat/completions",
            streaming_supported: model.supports_streaming,
            sort_order: 10,
        },
    }
}

fn model_modality_codes(model: &ModelInfo) -> BTreeSet<String> {
    model
        .input_modalities
        .iter()
        .chain(model.output_modalities.iter())
        .chain(std::iter::once(&model.primary_capability))
        .filter_map(|value| {
            let value = value.trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            }
        })
        .collect()
}

fn model_endpoint_modalities(model: &ModelInfo) -> BTreeSet<String> {
    model
        .input_modalities
        .iter()
        .chain(model.output_modalities.iter())
        .chain(std::iter::once(&model.primary_capability))
        .filter_map(|value| {
            let value = value.trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            }
        })
        .collect()
}

fn model_modality_directions(model: &ModelInfo) -> Vec<(String, String)> {
    let input = model
        .input_modalities
        .iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    let output = model
        .output_modalities
        .iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    input
        .union(&output)
        .map(|modality_code| {
            let direction = match (
                input.contains(modality_code),
                output.contains(modality_code),
            ) {
                (true, true) => "input_output",
                (true, false) => "input",
                (false, true) => "output",
                (false, false) => "input_output",
            };
            (modality_code.clone(), direction.to_owned())
        })
        .collect()
}

fn endpoint_vendor_code(endpoint_code: &str) -> Option<String> {
    endpoint_code
        .split_once('.')
        .map(|(vendor_code, _)| vendor_code.to_owned())
        .filter(|vendor_code| vendor_code != "rerank")
}

fn endpoint_modality_code(endpoint_code: &str) -> Option<String> {
    match endpoint_code {
        "openai.images" => Some("image"),
        "openai.audio" => Some("audio"),
        "suno.music" => Some("music"),
        "openai.video" => Some("video"),
        "openai.embeddings" => Some("embedding"),
        "rerank" => Some("rerank"),
        "openai.chat_completions" => Some("chat"),
        _ => None,
    }
    .map(str::to_owned)
}

fn modality_display_name(modality_code: &str) -> String {
    match modality_code {
        "llm" => "LLM",
        "text" => "Text",
        "chat" => "Chat",
        "image" => "Image",
        "audio" => "Audio",
        "music" => "Music",
        "video" => "Video",
        "embedding" => "Embedding",
        "rerank" => "Rerank",
        "tool" => "Tool",
        "storage" => "Storage",
        "network" => "Network",
        value => value,
    }
    .to_owned()
}

fn modality_group(modality_code: &str) -> &'static str {
    match modality_code {
        "llm" | "chat" | "text" | "embedding" | "rerank" => "language",
        "image" | "video" => "visual",
        "audio" | "music" => "audio",
        "tool" | "storage" | "network" => "tooling",
        _ => "custom",
    }
}

fn modality_description(modality_code: &str) -> String {
    format!("SDKWork model catalog {modality_code} modality")
}

fn modality_sort_order(modality_code: &str) -> Option<i32> {
    match modality_code {
        "llm" => Some(5),
        "chat" => Some(10),
        "text" => Some(20),
        "embedding" => Some(30),
        "image" => Some(40),
        "audio" => Some(50),
        "music" => Some(60),
        "video" => Some(70),
        "rerank" => Some(80),
        "tool" => Some(90),
        "storage" => Some(100),
        "network" => Some(110),
        _ => None,
    }
}

pub(crate) fn is_dry_run_mode(mode: &str) -> bool {
    mode == SYNC_MODE_DRY_RUN
}

pub(crate) fn catalog_preview_admin_items(
    catalog: &ModelCatalog,
    subject: AdminModelSubject,
) -> (Vec<AdminModelVendorItem>, Vec<AdminAiModelItem>) {
    let vendors = catalog_vendor_records(catalog)
        .into_iter()
        .map(|vendor| AdminModelVendorItem {
            id: 0,
            uuid: stable_uuid("sdk-vendor-preview", &[&vendor.vendor_code]),
            tenant_id: subject.tenant_id,
            organization_id: subject.organization_id,
            vendor_code: vendor.vendor_code,
            name: vendor.display_name,
            status: "active".to_owned(),
            color: "bg-slate-700".to_owned(),
            description: vendor.description.unwrap_or_default(),
            supported_protocols: json_array(&vendor.supported_protocols),
            client_api_compatibility: serde_json::to_string(&vendor.client_api_compatibility)
                .unwrap_or_else(|_| "{}".to_owned()),
            deleted_at: None,
        })
        .map(|item| (item.vendor_code.clone(), item))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .enumerate()
        .map(|(index, mut item)| {
            item.id = (index as i64) + 1;
            item
        })
        .collect::<Vec<_>>();
    let models = public_catalog_identity_models(catalog)
        .into_iter()
        .map(|(catalog_key, (vendor, model))| {
            let prices = vendor
                .pricing
                .iter()
                .find(|pricing| pricing.model_id == model.model_id);
            let item = AdminAiModelItem {
                id: 0,
                uuid: stable_uuid(
                    "sdk-model-preview",
                    &[&vendor.vendor.vendor_code, &model.model_id],
                ),
                tenant_id: subject.tenant_id,
                organization_id: subject.organization_id,
                vendor_id: vendor.vendor.vendor_code.clone(),
                vendor_code: vendor.vendor.vendor_code.clone(),
                vendor_name: vendor.vendor.display_name.clone(),
                catalog_key: catalog_key.clone(),
                model: model.model_id.clone(),
                display_name: model.display_name.clone(),
                name: if model.display_name.trim().is_empty() {
                    model.model_id.clone()
                } else {
                    model.display_name.clone()
                },
                model_type: preview_model_type(model),
                region_prices: vec![AdminAiModelRegionPriceCommand {
                    region_code: vendor.vendor.region_code.clone(),
                    currency: preview_currency(prices, &vendor.vendor.region_code),
                    price_in: preview_price(prices, true),
                    price_out: preview_price(prices, false),
                    cache_read_price: non_empty_preview_cache_price(prices, "llm_cache_read_token"),
                    cache_write_price: non_empty_preview_cache_price(
                        prices,
                        "llm_cache_write_token",
                    ),
                }],
                status: "active".to_owned(),
                calls: "0".to_owned(),
                description: model.description.clone(),
                modalities: preview_modalities(model),
                input_modalities: model.input_modalities.clone(),
                output_modalities: model.output_modalities.clone(),
                api_format: Some(model.api_format.clone()),
                capability_intro: None,
                limitations: Vec::new(),
                supported_languages: Vec::new(),
                use_cases: model.strengths.clone(),
                training_data_cutoff: None,
                context_tokens: model.context_tokens,
                max_output_tokens: model.max_output_tokens,
                supports_streaming: model.supports_streaming,
                supports_tools: model.supports_tools,
                supports_json_schema: model.supports_json_schema,
                release_stage: Some(release_stage_code(&model.release_stage)),
                shelf_state: Some(shelf_state_code(&model.shelf_state)),
                routing_state: Some(routing_state_code(&model.routing_state)),
                replacement_model: model.replacement_model.clone(),
                deleted_at: None,
            };
            (catalog_key, item)
        })
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .enumerate()
        .map(|(index, mut item)| {
            item.id = (index as i64) + 1;
            item
        })
        .collect::<Vec<_>>();
    (vendors, models)
}

fn validate_catalog_version_pin(
    catalog: &ModelCatalog,
    catalog_version: Option<&str>,
) -> Result<(), CatalogImportError> {
    let Some(expected) = catalog_version
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    if expected != catalog.manifest.catalog_version {
        return Err(CatalogImportError::CatalogVersionMismatch {
            expected: expected.to_owned(),
            actual: catalog.manifest.catalog_version.clone(),
        });
    }
    Ok(())
}

fn normalized_vendor_set(vendor_codes: &[String]) -> BTreeSet<String> {
    vendor_codes
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}

pub(crate) fn catalog_scope_source_hash(source_code: &str, catalog: &ModelCatalog) -> String {
    let payload = serde_json::json!({
        "hashKind": "sdkwork-models.catalog-scope.v1",
        "sourceCode": source_code,
        "catalog": catalog,
    });
    let bytes = serde_json::to_vec(&payload).unwrap_or_else(|_| {
        format!(
            "{}:{}:{}:{}:{}",
            source_code,
            catalog.manifest.schema_version,
            catalog.manifest.catalog_version,
            catalog.manifest.generated_at,
            catalog.vendors.len()
        )
        .into_bytes()
    });
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hex::encode(hasher.finalize())
}

fn preview_model_type(model: &ModelInfo) -> String {
    if model
        .input_modalities
        .iter()
        .chain(model.output_modalities.iter())
        .any(|modality| modality == "embedding")
    {
        return "Embedding".to_owned();
    }
    match model.primary_capability.as_str() {
        "image" => "Image",
        "audio" => "Audio",
        "music" => "Music",
        "sfx" | "sound_effect" => "SoundEffect",
        "video" => "Video",
        "embedding" => "Embedding",
        _ => "Chat",
    }
    .to_owned()
}

fn preview_currency(pricing: Option<&sdkwork_models::ModelPricing>, region_code: &str) -> String {
    pricing
        .map(|pricing| pricing.currency.trim().to_ascii_uppercase())
        .filter(|currency| {
            currency.len() == 3 && currency.bytes().all(|byte| byte.is_ascii_uppercase())
        })
        .unwrap_or_else(|| match region_code {
            "cn" => "CNY".to_owned(),
            _ => "USD".to_owned(),
        })
}

fn preview_modalities(model: &ModelInfo) -> Vec<String> {
    let mut values = model.input_modalities.clone();
    for modality in &model.output_modalities {
        if !values.contains(modality) {
            values.push(modality.clone());
        }
    }
    values
}

fn preview_price(pricing: Option<&sdkwork_models::ModelPricing>, input: bool) -> String {
    let Some(pricing) = pricing else {
        return String::new();
    };
    let meters: &[&str] = if input {
        &[
            "llm_input_token",
            "embedding_input_token",
            "image_input_token",
            "image_megapixel",
            "audio_input_token",
            "audio_input_second",
            "audio_input_minute",
            "stt_audio_minute",
            "tts_input_character",
            "api_request",
            "video_input_token",
        ]
    } else {
        &[
            "llm_output_token",
            "image_output_token",
            "image_result",
            "image_megapixel",
            "audio_output_token",
            "audio_output_second",
            "music_output_second",
            "sfx_result",
            "video_output_token",
            "video_output_second",
            "video_result",
            "api_result",
        ]
    };
    pricing
        .prices
        .iter()
        .find(|price| meters.contains(&price.meter_code.as_str()))
        .map(|price| price.unit_price.clone())
        .unwrap_or_default()
}

fn preview_cache_price(pricing: Option<&sdkwork_models::ModelPricing>, meter_code: &str) -> String {
    pricing
        .and_then(|pricing| {
            pricing
                .prices
                .iter()
                .find(|price| price.meter_code == meter_code)
        })
        .map(|price| price.unit_price.clone())
        .unwrap_or_default()
}

fn non_empty_preview_cache_price(
    pricing: Option<&sdkwork_models::ModelPricing>,
    meter_code: &str,
) -> Option<String> {
    let value = preview_cache_price(pricing, meter_code);
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

pub(crate) fn stable_uuid(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    for part in parts {
        hasher.update(b":");
        hasher.update(part.as_bytes());
    }
    let digest = format!("{:x}", hasher.finalize());
    format!("{prefix}-{}", &digest[..40])
}

pub(crate) fn catalog_sync_run_uuid(snapshot_uuid: &str) -> String {
    const MAX_LEN: usize = 64;
    let prefixed = format!("catalog-sync-{snapshot_uuid}");
    if prefixed.len() <= MAX_LEN {
        prefixed
    } else {
        snapshot_uuid.to_owned()
    }
}

pub(crate) fn stable_catalog_id(prefix: &str, parts: &[&str]) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    for part in parts {
        hasher.update(b":");
        hasher.update(part.as_bytes());
    }
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    let value = u64::from_be_bytes(bytes) & 0x3fff_ffff_ffff_ffff;
    (value as i64) + 1
}

pub(crate) fn metadata_json(
    catalog: &ModelCatalog,
    source: &str,
    extra: serde_json::Value,
) -> String {
    serde_json::json!({
        "source": source,
        "catalogVersion": catalog.manifest.catalog_version,
        "schemaVersion": catalog.manifest.schema_version,
        "generatedAt": catalog.manifest.generated_at,
        "extra": extra,
    })
    .to_string()
}

pub(crate) fn modality_code(value: &str) -> i32 {
    match value {
        "text" => 1,
        "image" => 2,
        "audio" => 3,
        "music" => 4,
        "video" => 5,
        "embedding" => 6,
        "rerank" => 7,
        "tool" => 8,
        "storage" => 9,
        "network" => 10,
        _ => 0,
    }
}

pub(crate) fn capability_code(value: &str) -> i32 {
    match value {
        "image" => 2,
        "audio" => 3,
        "music" => 4,
        "video" => 5,
        "embedding" => 6,
        "rerank" => 7,
        "tool" => 8,
        _ => 1,
    }
}

pub(crate) fn family_type_code(value: &str) -> i32 {
    match value {
        "embedding" => 2,
        "image" => 3,
        "audio" => 4,
        "music" => 5,
        "video" => 6,
        "rerank" => 7,
        "multimodal" => 8,
        _ => 1,
    }
}

pub(crate) fn vendor_type_code(value: &str) -> i32 {
    match value {
        "open_source" => 2,
        "research" => 3,
        "community" => 4,
        _ => 1,
    }
}

pub(crate) fn release_stage_code(value: &str) -> i32 {
    match value {
        "preview" => 2,
        "deprecated" => 3,
        "retired" => 4,
        _ => 1,
    }
}

pub(crate) fn shelf_state_code(value: &str) -> i32 {
    match value {
        "hidden" => 2,
        "archived" => 3,
        _ => 1,
    }
}

pub(crate) fn routing_state_code(value: &str) -> i32 {
    match value {
        "enabled" => 1,
        _ => 0,
    }
}

pub(crate) fn lifecycle_code(value: &str) -> i32 {
    match value {
        "preview" => 2,
        "deprecated" => 3,
        "catalog_only" => 4,
        "retired" => 5,
        _ => 1,
    }
}

pub(crate) fn voice_gender_code(value: &str) -> i32 {
    match value {
        "male" => 1,
        "female" => 2,
        "neutral" => 3,
        _ => 0,
    }
}

pub(crate) fn price_side_code(value: &str) -> i32 {
    match value {
        "upstream" => 2,
        "customer" => 3,
        _ => 1,
    }
}

pub(crate) fn price_supplier_code(
    vendor_code: &str,
    _region_code: &str,
    price_side: &str,
    pricing_scope: Option<&str>,
) -> Option<String> {
    if price_side == "upstream" || matches!(pricing_scope, Some("provider" | "channel")) {
        Some(format!("{vendor_code}_direct"))
    } else {
        None
    }
}

pub(crate) fn pricing_scope_code(value: Option<&str>) -> i32 {
    match value {
        Some("provider") => 2,
        Some("channel") => 3,
        Some("plan") => 4,
        _ => 1,
    }
}

pub(crate) fn primary_modality(model: &ModelInfo) -> i32 {
    model
        .output_modalities
        .first()
        .or_else(|| model.input_modalities.first())
        .map(|value| modality_code(value))
        .unwrap_or_else(|| capability_code(&model.primary_capability))
}

pub(crate) fn model_modalities_json(model: &ModelInfo) -> String {
    let mut values = model.input_modalities.clone();
    for output in &model.output_modalities {
        if !values.contains(output) {
            values.push(output.clone());
        }
    }
    serde_json::to_string(&values).unwrap_or_else(|_| "[]".to_owned())
}

pub(crate) fn model_capabilities_json(model: &ModelInfo) -> String {
    let capabilities;
    let values = if model.capabilities.is_empty() {
        capabilities = vec![model.primary_capability.clone()];
        capabilities.as_slice()
    } else {
        model.capabilities.as_slice()
    };
    json_array(values)
}

pub(crate) fn json_array(values: &[String]) -> String {
    serde_json::to_string(values).unwrap_or_else(|_| "[]".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{ai_resource_description, catalog_sync_run_uuid, price_supplier_code};

    #[test]
    fn ai_resource_description_preserves_none_and_short_values() {
        assert_eq!(None, ai_resource_description(None));
        assert_eq!(
            Some("short description".to_owned()),
            ai_resource_description(Some("short description".to_owned()))
        );
    }

    #[test]
    fn ai_resource_description_preserves_exact_character_limit() {
        let description = "a".repeat(512);
        assert_eq!(
            Some(description.clone()),
            ai_resource_description(Some(description))
        );
    }

    #[test]
    fn ai_resource_description_truncates_values_over_character_limit() {
        let description = format!("{}b", "a".repeat(512));
        assert_eq!(
            Some("a".repeat(512)),
            ai_resource_description(Some(description))
        );
    }

    #[test]
    fn ai_resource_description_truncates_multibyte_values_at_utf8_boundary() {
        let description = format!("{}{}", "界".repeat(512), "文".repeat(48));
        let projected = ai_resource_description(Some(description)).expect("description");
        assert_eq!(512, projected.chars().count());
        assert_eq!("界".repeat(512), projected);
    }

    #[test]
    fn catalog_sync_run_uuid_fits_varchar_64_for_installer_style_snapshot_uuid() {
        let snapshot_uuid = format!("catalog-refresh-{}", "11111111-1111-4111-8111-111111111111");
        let sync_run_uuid = catalog_sync_run_uuid(&snapshot_uuid);
        assert!(
            sync_run_uuid.len() <= 64,
            "sync run uuid must fit VARCHAR(64): len={} value={sync_run_uuid}",
            sync_run_uuid.len()
        );
        assert_eq!(snapshot_uuid, sync_run_uuid);
    }

    #[test]
    fn catalog_sync_run_uuid_keeps_prefix_for_standard_snapshot_uuid() {
        let snapshot_uuid = "11111111-1111-4111-8111-111111111111".to_owned();
        assert_eq!(
            "catalog-sync-11111111-1111-4111-8111-111111111111",
            catalog_sync_run_uuid(&snapshot_uuid)
        );
    }

    #[test]
    fn price_supplier_code_keeps_vendor_identity_separate_from_region() {
        assert_eq!(
            Some("minimax_direct".to_owned()),
            price_supplier_code("minimax", "cn", "upstream", None)
        );
        assert_eq!(
            Some("minimax_direct".to_owned()),
            price_supplier_code("minimax", "global", "official", Some("provider"))
        );
        assert_eq!(
            Some("kuaishou_direct".to_owned()),
            price_supplier_code("kuaishou", "global", "official", Some("channel"))
        );
        assert_eq!(
            None,
            price_supplier_code("minimax", "cn", "official", Some("model"))
        );
    }
}
