use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

pub mod bundled;
pub mod loader;
pub mod query;
pub mod types;
pub mod validation;

pub use loader::{load_bundled_catalog, load_catalog, load_vendor_catalog};
pub use query::{
    catalog_key, find_meter, find_model, find_model_by_vendor_region, find_protocol,
    get_best_reference_price, get_model_prices, get_model_region_prices, list_available_models,
    list_client_api_compatibility_by_vendor, list_meters, list_models, list_models_by_capability,
    list_models_by_modality, list_models_by_protocol, list_protocols, list_protocols_by_vendor,
    list_vendor_regions, list_vendors, ModelFilter,
};
pub use types::*;
pub use validation::{validate_catalog, CatalogIssue};

#[derive(Debug)]
pub enum CatalogError {
    Io(io::Error),
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl Display for CatalogError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "catalog IO error: {error}"),
            Self::Json { path, source } => {
                write!(
                    formatter,
                    "catalog JSON error in {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl Error for CatalogError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json { source, .. } => Some(source),
        }
    }
}

impl From<io::Error> for CatalogError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
