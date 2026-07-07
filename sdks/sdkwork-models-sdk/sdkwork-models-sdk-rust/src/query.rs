use std::collections::{BTreeMap, BTreeSet};

use crate::capabilities::model_supports_feature;
use crate::types::{
    BillingMeter, ClientApiCompatibility, ModelCatalog, ModelInfo, ModelPrice, ModelVendorIdentity,
    ProtocolStandard, TtsVoice, VendorRegionRef, VideoGenerationProfile,
};

#[derive(Debug, Clone, Default)]
pub struct ModelFilter<'a> {
    pub vendor_code: Option<&'a str>,
    pub region_code: Option<&'a str>,
    pub family_code: Option<&'a str>,
    pub capability: Option<&'a str>,
    pub input_modality: Option<&'a str>,
    pub output_modality: Option<&'a str>,
    pub release_stage: Option<&'a str>,
    pub shelf_state: Option<&'a str>,
    pub routing_state: Option<&'a str>,
    pub api_format: Option<&'a str>,
}

pub fn list_vendors(catalog: &ModelCatalog) -> Vec<ModelVendorIdentity> {
    let mut seen = BTreeSet::new();
    let mut vendors = Vec::new();
    for region_catalog in &catalog.vendors {
        let vendor = &region_catalog.vendor;
        if !seen.insert(vendor.vendor_code.clone()) {
            continue;
        }
        vendors.push(ModelVendorIdentity {
            vendor_code: vendor.vendor_code.clone(),
            display_name: vendor.display_name.clone(),
            legal_name: vendor.legal_name.clone(),
            vendor_type: vendor.vendor_type.clone(),
            capabilities: vendor.capabilities.clone(),
            supported_protocols: vendor.supported_protocols.clone(),
            client_api_compatibility: vendor.client_api_compatibility.clone(),
            open_source: vendor.open_source.unwrap_or(false),
        });
    }
    vendors
}

pub fn list_vendor_regions(catalog: &ModelCatalog) -> Vec<VendorRegionRef> {
    catalog
        .vendors
        .iter()
        .map(|vendor| VendorRegionRef {
            vendor_code: vendor.vendor_code.clone(),
            region_code: vendor.region_code.clone(),
        })
        .collect()
}

pub fn catalog_key(vendor_code: &str, model_id: &str) -> String {
    format!("{vendor_code}/{model_id}")
}

pub fn list_meters(catalog: &ModelCatalog) -> Vec<&BillingMeter> {
    catalog.meters.iter().collect()
}

pub fn find_meter<'a>(catalog: &'a ModelCatalog, meter_code: &str) -> Option<&'a BillingMeter> {
    catalog
        .meters
        .iter()
        .find(|meter| meter.meter_code == meter_code)
}

pub fn list_models<'a>(catalog: &'a ModelCatalog, filter: ModelFilter<'_>) -> Vec<&'a ModelInfo> {
    let mut models = catalog
        .vendors
        .iter()
        .flat_map(|vendor| {
            vendor.models.iter().map(move |model| {
                (
                    vendor.pricing.iter().any(|pricing| {
                        pricing.model_id == model.model_id && !pricing.prices.is_empty()
                    }),
                    model,
                )
            })
        })
        .filter(|(_, model)| {
            filter
                .vendor_code
                .map(|value| model.vendor_code == value)
                .unwrap_or(true)
        })
        .filter(|model| {
            filter
                .region_code
                .map(|value| model.1.region_code == value)
                .unwrap_or(true)
        })
        .filter(|model| {
            filter
                .family_code
                .map(|value| model.1.family_code == value)
                .unwrap_or(true)
        })
        .filter(|model| {
            filter
                .capability
                .map(|value| model.1.capabilities.iter().any(|item| item == value))
                .unwrap_or(true)
        })
        .filter(|model| {
            filter
                .input_modality
                .map(|value| model.1.input_modalities.iter().any(|item| item == value))
                .unwrap_or(true)
        })
        .filter(|model| {
            filter
                .output_modality
                .map(|value| model.1.output_modalities.iter().any(|item| item == value))
                .unwrap_or(true)
        })
        .filter(|model| {
            filter
                .release_stage
                .map(|value| model.1.release_stage == value)
                .unwrap_or(true)
        })
        .filter(|model| {
            filter
                .shelf_state
                .map(|value| model.1.shelf_state == value)
                .unwrap_or(true)
        })
        .filter(|model| {
            filter
                .routing_state
                .map(|value| model.1.routing_state == value)
                .unwrap_or(true)
        })
        .filter(|model| {
            filter
                .api_format
                .map(|value| model.1.api_format == value)
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    if filter.region_code.is_none() {
        let mut deduped: BTreeMap<String, (bool, &ModelInfo)> = BTreeMap::new();
        for (has_region_pricing, model) in models {
            let replace = deduped
                .get(&model.catalog_key)
                .map(|(existing_has_pricing, existing_model)| {
                    model_identity_score(has_region_pricing, model)
                        > model_identity_score(*existing_has_pricing, existing_model)
                })
                .unwrap_or(true);
            if replace {
                deduped.insert(model.catalog_key.clone(), (has_region_pricing, model));
            }
        }
        return deduped.into_values().map(|(_, model)| model).collect();
    }
    models.drain(..).map(|(_, model)| model).collect()
}

pub fn list_available_models<'a>(
    catalog: &'a ModelCatalog,
    filter: ModelFilter<'_>,
) -> Vec<&'a ModelInfo> {
    list_models(
        catalog,
        ModelFilter {
            routing_state: Some("enabled"),
            shelf_state: Some("listed"),
            ..filter
        },
    )
    .into_iter()
    .filter(|model| {
        !get_model_region_prices(catalog, &model.catalog_key, &model.region_code).is_empty()
    })
    .collect()
}

pub fn find_model<'a>(catalog: &'a ModelCatalog, catalog_key: &str) -> Option<&'a ModelInfo> {
    let (vendor_code, model_id) = split_catalog_key(catalog_key)?;
    list_models(catalog, ModelFilter::default())
        .into_iter()
        .find(|model| model.vendor_code == vendor_code && model.model_id == model_id)
}

pub fn find_model_by_vendor_region<'a>(
    catalog: &'a ModelCatalog,
    vendor_code: &str,
    region_code: &str,
    model_id: &str,
) -> Option<&'a ModelInfo> {
    catalog
        .vendors
        .iter()
        .filter(|vendor| vendor.vendor_code == vendor_code && vendor.region_code == region_code)
        .flat_map(|vendor| vendor.models.iter())
        .find(|model| model.model_id == model_id && model.region_code == region_code)
}

pub fn get_model_prices<'a>(catalog: &'a ModelCatalog, catalog_key: &str) -> Vec<&'a ModelPrice> {
    let Some((vendor_code, model_id)) = split_catalog_key(catalog_key) else {
        return Vec::new();
    };
    catalog
        .vendors
        .iter()
        .filter(|vendor| vendor.vendor_code == vendor_code)
        .flat_map(|vendor| vendor.pricing.iter())
        .find(|pricing| pricing.model_id == model_id)
        .map(|pricing| pricing.prices.iter().collect())
        .unwrap_or_default()
}

pub fn get_model_region_prices<'a>(
    catalog: &'a ModelCatalog,
    catalog_key: &str,
    region_code: &str,
) -> Vec<&'a ModelPrice> {
    let Some((vendor_code, model_id)) = split_catalog_key(catalog_key) else {
        return Vec::new();
    };
    catalog
        .vendors
        .iter()
        .filter(|vendor| vendor.vendor_code == vendor_code && vendor.region_code == region_code)
        .flat_map(|vendor| vendor.pricing.iter())
        .find(|pricing| pricing.model_id == model_id)
        .map(|pricing| pricing.prices.iter().collect())
        .unwrap_or_default()
}

pub fn get_best_reference_price<'a>(
    catalog: &'a ModelCatalog,
    catalog_key: &str,
    meter_code: &str,
) -> Option<&'a ModelPrice> {
    get_model_prices(catalog, catalog_key)
        .into_iter()
        .find(|price| price.meter_code == meter_code)
}

pub fn list_models_by_capability<'a>(
    catalog: &'a ModelCatalog,
    capability: &'a str,
) -> Vec<&'a ModelInfo> {
    list_models(
        catalog,
        ModelFilter {
            capability: Some(capability),
            ..ModelFilter::default()
        },
    )
}

pub fn list_models_by_modality<'a>(
    catalog: &'a ModelCatalog,
    input_modality: &'a str,
    output_modality: &'a str,
) -> Vec<&'a ModelInfo> {
    list_models(
        catalog,
        ModelFilter {
            input_modality: Some(input_modality),
            output_modality: Some(output_modality),
            ..ModelFilter::default()
        },
    )
}

pub fn list_models_with_feature<'a>(
    catalog: &'a ModelCatalog,
    feature: &'a str,
) -> Vec<&'a ModelInfo> {
    list_models(catalog, ModelFilter::default())
        .into_iter()
        .filter(|model| model_supports_feature(model, feature))
        .collect()
}

pub fn list_protocols(catalog: &ModelCatalog) -> Vec<&ProtocolStandard> {
    catalog.protocols.iter().collect()
}

pub fn find_protocol<'a>(
    catalog: &'a ModelCatalog,
    protocol_code: &str,
) -> Option<&'a ProtocolStandard> {
    catalog
        .protocols
        .iter()
        .find(|protocol| protocol.protocol_code == protocol_code)
}

pub fn list_protocols_by_vendor<'a>(
    catalog: &'a ModelCatalog,
    vendor_code: &str,
) -> Vec<&'a ProtocolStandard> {
    let Some(vendor) = catalog
        .vendors
        .iter()
        .map(|region_catalog| &region_catalog.vendor)
        .find(|vendor| vendor.vendor_code == vendor_code)
    else {
        return Vec::new();
    };
    catalog
        .protocols
        .iter()
        .filter(|protocol| {
            vendor
                .supported_protocols
                .iter()
                .any(|supported| supported == &protocol.protocol_code)
        })
        .collect()
}

pub fn list_client_api_compatibility_by_vendor<'a>(
    catalog: &'a ModelCatalog,
    vendor_code: &str,
) -> Vec<&'a ClientApiCompatibility> {
    let Some(vendor) = catalog
        .vendors
        .iter()
        .map(|region_catalog| &region_catalog.vendor)
        .find(|vendor| vendor.vendor_code == vendor_code)
    else {
        return Vec::new();
    };
    vendor.client_api_compatibility.values().collect()
}

pub fn list_models_by_protocol<'a>(
    catalog: &'a ModelCatalog,
    protocol_code: &'a str,
) -> Vec<&'a ModelInfo> {
    list_models(
        catalog,
        ModelFilter {
            api_format: Some(protocol_code),
            ..ModelFilter::default()
        },
    )
}

fn model_identity_score(has_region_pricing: bool, model: &ModelInfo) -> i32 {
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
    if model.region_code == "global" {
        score += 1;
    }
    score
}

#[derive(Debug, Clone, Default)]
pub struct VoiceFilter<'a> {
    pub vendor_code: Option<&'a str>,
    pub region_code: Option<&'a str>,
    pub locale: Option<&'a str>,
    pub model_catalog_key: Option<&'a str>,
    pub search_query: Option<&'a str>,
}

pub fn voice_catalog_key(vendor_code: &str, voice_id: &str) -> String {
    format!("{vendor_code}/{voice_id}")
}

pub fn list_voices<'a>(catalog: &'a ModelCatalog, filter: VoiceFilter<'_>) -> Vec<&'a TtsVoice> {
    catalog
        .vendors
        .iter()
        .flat_map(|vendor| vendor.voices.iter())
        .filter(|voice| {
            filter
                .vendor_code
                .map(|code| voice.vendor_code == code)
                .unwrap_or(true)
        })
        .filter(|voice| {
            filter
                .region_code
                .map(|code| voice.region_code == code)
                .unwrap_or(true)
        })
        .filter(|voice| match filter.locale {
            Some(locale) => {
                voice.primary_locale == locale
                    || voice.supported_locales.iter().any(|entry| entry == locale)
            }
            None => true,
        })
        .filter(|voice| match filter.search_query {
            Some(query) => {
                let query = query.to_ascii_lowercase();
                voice.display_name.to_ascii_lowercase().contains(&query)
                    || voice.voice_id.to_ascii_lowercase().contains(&query)
            }
            None => true,
        })
        .filter(|voice| match filter.model_catalog_key {
            Some(model_key) => catalog
                .vendors
                .iter()
                .flat_map(|vendor| vendor.model_voice_bindings.iter())
                .any(|binding| {
                    binding.catalog_key == model_key
                        && binding
                            .bindings
                            .iter()
                            .any(|entry| entry.voice_key == voice.catalog_key)
                }),
            None => true,
        })
        .collect()
}

pub fn list_voices_for_model<'a>(
    catalog: &'a ModelCatalog,
    model_catalog_key: &str,
) -> Vec<&'a TtsVoice> {
    list_voices(
        catalog,
        VoiceFilter {
            model_catalog_key: Some(model_catalog_key),
            ..VoiceFilter::default()
        },
    )
}

pub fn list_models_for_voice<'a>(
    catalog: &'a ModelCatalog,
    voice_catalog_key: &str,
) -> Vec<&'a ModelInfo> {
    let model_keys = catalog
        .vendors
        .iter()
        .flat_map(|vendor| vendor.model_voice_bindings.iter())
        .filter(|binding| {
            binding
                .bindings
                .iter()
                .any(|entry| entry.voice_key == voice_catalog_key)
        })
        .map(|binding| binding.catalog_key.as_str())
        .collect::<BTreeSet<_>>();
    catalog
        .vendors
        .iter()
        .flat_map(|vendor| vendor.models.iter())
        .filter(|model| model_keys.contains(model.catalog_key.as_str()))
        .collect()
}

#[derive(Debug, Clone, Default)]
pub struct VideoProfileFilter<'a> {
    pub vendor_code: Option<&'a str>,
    pub region_code: Option<&'a str>,
    pub model_catalog_key: Option<&'a str>,
    pub generation_mode: Option<&'a str>,
    pub duration_tier_code: Option<&'a str>,
    pub resolution: Option<&'a str>,
}

pub fn video_profile_catalog_key(vendor_code: &str, model_id: &str, profile_code: &str) -> String {
    format!("{vendor_code}/{model_id}/{profile_code}")
}

pub fn list_video_profiles<'a>(
    catalog: &'a ModelCatalog,
    filter: VideoProfileFilter<'_>,
) -> Vec<&'a VideoGenerationProfile> {
    let mut profiles = Vec::new();
    for vendor in &catalog.vendors {
        if filter
            .vendor_code
            .is_some_and(|code| vendor.vendor_code != code)
        {
            continue;
        }
        if filter
            .region_code
            .is_some_and(|code| vendor.region_code != code)
        {
            continue;
        }
        for file in &vendor.model_video_profiles {
            if filter
                .model_catalog_key
                .is_some_and(|model_key| file.catalog_key != model_key)
            {
                continue;
            }
            for profile in &file.profiles {
                if filter
                    .generation_mode
                    .is_some_and(|mode| profile.generation_mode != mode)
                {
                    continue;
                }
                if filter.duration_tier_code.is_some_and(|tier_code| {
                    profile.duration_tier_code.as_deref() != Some(tier_code)
                        && !profile
                            .duration_tier_codes
                            .iter()
                            .any(|entry| entry == tier_code)
                }) {
                    continue;
                }
                if filter
                    .resolution
                    .is_some_and(|resolution| profile.resolution != resolution)
                {
                    continue;
                }
                profiles.push(profile);
            }
        }
    }
    profiles
}

pub fn list_video_profiles_for_model<'a>(
    catalog: &'a ModelCatalog,
    model_catalog_key: &str,
) -> Vec<&'a VideoGenerationProfile> {
    list_video_profiles(
        catalog,
        VideoProfileFilter {
            model_catalog_key: Some(model_catalog_key),
            ..VideoProfileFilter::default()
        },
    )
}

pub fn find_video_profile<'a>(
    catalog: &'a ModelCatalog,
    profile_catalog_key: &str,
) -> Option<&'a VideoGenerationProfile> {
    catalog
        .vendors
        .iter()
        .flat_map(|vendor| vendor.model_video_profiles.iter())
        .flat_map(|file| file.profiles.iter())
        .find(|profile| profile.catalog_key == profile_catalog_key)
}

fn split_catalog_key(catalog_key: &str) -> Option<(&str, &str)> {
    let (vendor_code, model_id) = catalog_key.split_once('/')?;
    if vendor_code.is_empty() || model_id.is_empty() {
        return None;
    }
    Some((vendor_code, model_id))
}
