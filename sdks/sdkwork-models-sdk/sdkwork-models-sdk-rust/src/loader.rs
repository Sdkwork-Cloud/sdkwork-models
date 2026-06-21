use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::types::{
    BillingMeter, CatalogIndex, CatalogIndexVendor, CatalogManifest, FamilyFile, ModelCatalog,
    ModelInfo, ModelPricing, ModelVendor, ProtocolStandard, RankingFile, VendorCatalog,
};
use crate::CatalogError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MeterFile {
    meters: Vec<BillingMeter>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolFile {
    protocols: Vec<ProtocolStandard>,
}

pub fn load_catalog(root: impl AsRef<Path>) -> Result<ModelCatalog, CatalogError> {
    let root = root.as_ref();
    let manifest: CatalogManifest = read_json(root.join("sdkwork-models.json"))?;
    let meters_file: MeterFile = read_json(root.join(&manifest.models_root).join("meters.json"))?;
    let protocols_file: ProtocolFile =
        read_json(root.join(&manifest.models_root).join("protocols.json"))?;
    let index: CatalogIndex = read_json(root.join(&manifest.models_root).join("index.json"))?;

    let vendors = index
        .vendors
        .into_iter()
        .map(|entry| load_vendor_catalog_from_index(root, &manifest.models_root, &entry))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ModelCatalog {
        manifest,
        meters: meters_file.meters,
        protocols: protocols_file.protocols,
        vendors,
    })
}

pub fn load_bundled_catalog() -> Result<ModelCatalog, CatalogError> {
    if let Ok(configured_root) = std::env::var("SDKWORK_MODELS_CATALOG_ROOT") {
        let configured_root = configured_root.trim();
        if !configured_root.is_empty() {
            return load_catalog(PathBuf::from(configured_root));
        }
    }
    if let Some(root) = discover_catalog_root(Path::new(env!("CARGO_MANIFEST_DIR"))) {
        return load_catalog(root);
    }
    load_catalog(PathBuf::from("data").join("sdkwork-models"))
}

fn discover_catalog_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    for _ in 0..8 {
        if current.join("sdkwork-models.json").is_file() {
            return Some(current);
        }
        current = current.parent()?.to_path_buf();
    }
    None
}

pub fn load_vendor_catalog(vendor_root: impl AsRef<Path>) -> Result<VendorCatalog, CatalogError> {
    let vendor_root = vendor_root.as_ref();
    let vendor: ModelVendor = read_json(vendor_root.join("vendor.json"))?;
    let families_file: FamilyFile = read_json(vendor_root.join("families.json"))?;
    let rankings_file: RankingFile = read_json(vendor_root.join("rankings.json"))?;
    let models = read_json_dir::<ModelInfo>(vendor_root.join("models"))?;
    let pricing = read_json_dir::<ModelPricing>(vendor_root.join("pricing"))?;
    Ok(VendorCatalog {
        vendor_code: vendor.vendor_code.clone(),
        region_code: vendor.region_code.clone(),
        vendor,
        families: families_file.families,
        models,
        pricing,
        rankings: rankings_file.snapshots,
    })
}

fn load_vendor_catalog_from_index(
    root: &Path,
    models_root: &str,
    index: &CatalogIndexVendor,
) -> Result<VendorCatalog, CatalogError> {
    let models_root = root.join(models_root);
    let vendor: ModelVendor = read_json(models_root.join(&index.path))?;
    let families_file: FamilyFile = read_json(models_root.join(&index.families_path))?;
    let rankings_file: RankingFile = match &index.rankings_path {
        Some(path) => read_json(models_root.join(path))?,
        None => RankingFile {
            vendor_code: index.vendor_code.clone(),
            region_code: index.region_code.clone(),
            snapshots: Vec::new(),
        },
    };
    let models = index
        .model_files
        .iter()
        .map(|path| read_json::<ModelInfo>(models_root.join(path)))
        .collect::<Result<Vec<_>, _>>()?;
    let pricing = index
        .pricing_files
        .iter()
        .map(|path| read_json::<ModelPricing>(models_root.join(path)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(VendorCatalog {
        vendor_code: vendor.vendor_code.clone(),
        region_code: vendor.region_code.clone(),
        vendor,
        families: families_file.families,
        models,
        pricing,
        rankings: rankings_file.snapshots,
    })
}

fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, CatalogError> {
    let path = path.as_ref();
    let body = fs::read_to_string(path).map_err(CatalogError::Io)?;
    serde_json::from_str(&body).map_err(|source| CatalogError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn read_json_dir<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<Vec<T>, CatalogError> {
    let mut files = Vec::new();
    collect_json_files(path.as_ref(), &mut files)?;
    files.sort();
    files.into_iter().map(read_json).collect()
}

fn collect_json_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), CatalogError> {
    let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("json") {
            files.push(path);
        }
    }
    Ok(())
}
