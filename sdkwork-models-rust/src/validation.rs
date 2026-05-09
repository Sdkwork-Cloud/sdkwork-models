use std::collections::BTreeSet;

use crate::types::ModelCatalog;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogIssue {
    pub code: String,
    pub path: String,
    pub message: String,
}

pub fn validate_catalog(catalog: &ModelCatalog) -> Vec<CatalogIssue> {
    let meters = catalog
        .meters
        .iter()
        .map(|meter| meter.meter_code.as_str())
        .collect::<BTreeSet<_>>();
    let models = catalog
        .vendors
        .iter()
        .flat_map(|vendor| vendor.models.iter().map(|model| model.catalog_key.as_str()))
        .collect::<BTreeSet<_>>();
    let mut issues = Vec::new();
    for vendor in &catalog.vendors {
        for pricing in &vendor.pricing {
            if !models.contains(pricing.catalog_key.as_str()) {
                issues.push(CatalogIssue {
                    code: "pricing.model.missing".to_owned(),
                    path: pricing.catalog_key.clone(),
                    message: "pricing references an unknown model".to_owned(),
                });
            }
            for price in &pricing.prices {
                if !meters.contains(price.meter_code.as_str()) {
                    issues.push(CatalogIssue {
                        code: "pricing.meter.missing".to_owned(),
                        path: format!("{}/{}", pricing.catalog_key, price.meter_code),
                        message: "pricing references an unknown meter".to_owned(),
                    });
                }
                for (field, value) in [
                    ("unitSize", price.unit_size.as_str()),
                    ("unitPrice", price.unit_price.as_str()),
                    ("minimumQuantity", price.minimum_quantity.as_str()),
                ] {
                    if !is_decimal_string(value) {
                        issues.push(CatalogIssue {
                            code: "pricing.decimal.invalid".to_owned(),
                            path: format!("{}/{}", pricing.catalog_key, field),
                            message: "price quantity fields must be decimal strings".to_owned(),
                        });
                    }
                }
            }
        }
    }
    issues
}

pub fn is_decimal_string(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let (integer, fraction) = match value.split_once('.') {
        Some((integer, fraction)) if !fraction.is_empty() => (integer, Some(fraction)),
        Some(_) => return false,
        None => (value, None),
    };
    if integer != "0" && integer.starts_with('0') {
        return false;
    }
    if integer.is_empty() || !integer.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }
    fraction
        .map(|value| value.chars().all(|ch| ch.is_ascii_digit()))
        .unwrap_or(true)
}
