use std::collections::{BTreeMap, BTreeSet};

use crate::types::{
    BillingMeter, ModelCatalog, ModelInfo, ModelPrice, ModelVendorIdentity, ProtocolStandard,
    VendorRegionRef,
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

pub fn catalog_key(vendor_code: &str, _region_code: &str, model_id: &str) -> String {
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
    let mut parts = catalog_key.split('/');
    let vendor_code = parts.next()?;
    let model_id = parts.next()?;
    if parts.next().is_some() || vendor_code.is_empty() || model_id.is_empty() {
        return None;
    }
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
    let mut parts = catalog_key.split('/');
    let Some(vendor_code) = parts.next() else {
        return Vec::new();
    };
    let Some(model_id) = parts.next() else {
        return Vec::new();
    };
    if parts.next().is_some() || vendor_code.is_empty() || model_id.is_empty() {
        return Vec::new();
    }
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
    let mut parts = catalog_key.split('/');
    let Some(vendor_code) = parts.next() else {
        return Vec::new();
    };
    let Some(model_id) = parts.next() else {
        return Vec::new();
    };
    if parts.next().is_some() || vendor_code.is_empty() || model_id.is_empty() {
        return Vec::new();
    }
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
