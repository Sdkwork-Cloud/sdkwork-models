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
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    load_catalog(root)
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
    let mut files = fs::read_dir(path.as_ref())?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("json"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    files.sort();
    files.into_iter().map(read_json).collect()
}
