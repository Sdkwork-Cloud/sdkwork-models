from __future__ import annotations

import json
import os
from pathlib import Path
from urllib.parse import urljoin, urlparse
from urllib.request import urlopen

from .types import ModelCatalog


def _is_remote_url(value: str | Path) -> bool:
    parsed = urlparse(str(value))
    return parsed.scheme in {"http", "https"}


def _read_json(root: str | Path, rel_path: str) -> dict:
    if _is_remote_url(root):
        with urlopen(urljoin(f"{str(root).rstrip('/')}/", rel_path), timeout=30) as response:
            return json.loads(response.read().decode("utf-8"))
    return json.loads((Path(root) / rel_path).read_text(encoding="utf-8"))


def load_catalog(path_or_url: str | Path) -> ModelCatalog:
    manifest = _read_json(path_or_url, "sdkwork-models.json")
    meters = _read_json(path_or_url, "models/meters.json")["meters"]
    protocols = _read_json(path_or_url, "models/protocols.json")["protocols"]
    index = _read_json(path_or_url, "models/index.json")
    vendors: list[dict] = []
    vendor_codes: set[str] = set()
    vendor_catalogs: list[dict] = []
    models: dict[str, dict] = {}
    model_scores: dict[str, int] = {}
    pricing: list[dict] = []
    for item in index.get("vendors", []):
        vendor_code = item["vendorCode"]
        region_code = item["regionCode"]
        vendor_catalog = load_vendor_catalog(path_or_url, vendor_code, region_code)
        vendor_catalogs.append(vendor_catalog)
        if vendor_code not in vendor_codes:
            vendor_codes.add(vendor_code)
            vendors.append(vendor_catalog["vendor"])
        for model in vendor_catalog["models"]:
            _put_best_model_identity(models, model_scores, vendor_catalog, model)
        pricing.extend(vendor_catalog["pricing"])
    return ModelCatalog(
        catalog_version=manifest["catalogVersion"],
        schema_version=manifest["schemaVersion"],
        meters=meters,
        protocols=protocols,
        vendors=vendors,
        vendor_catalogs=vendor_catalogs,
        models=list(models.values()),
        pricing=pricing,
    )


def load_bundled_catalog() -> ModelCatalog:
    configured_root = os.getenv("SDKWORK_MODELS_CATALOG_ROOT", "").strip()
    if configured_root:
        return load_catalog(configured_root)
    for ancestor in Path(__file__).resolve().parents:
        if (ancestor / "sdkwork-models.json").is_file():
            return load_catalog(ancestor)
    return load_catalog(Path("data") / "sdkwork-models")


def load_vendor_catalog(path_or_url: str | Path, vendor_code: str, region_code: str) -> dict:
    index = _read_json(path_or_url, "models/index.json")
    vendor_index = next(
        (
            item
            for item in index.get("vendors", [])
            if item.get("vendorCode") == vendor_code and item.get("regionCode") == region_code
        ),
        None,
    )
    if vendor_index is None:
        raise ValueError(f"vendor region {vendor_code}/{region_code} is not indexed")
    voices_path = vendor_index.get("voicesPath")
    voices: list[dict] = []
    if isinstance(voices_path, str) and voices_path:
        voices = _read_json(path_or_url, f"models/{voices_path}").get("voices", [])
    model_voice_bindings = [
        _read_json(path_or_url, f"models/{path}") for path in vendor_index.get("modelVoiceFiles", [])
    ]
    model_video_profiles = [
        _read_json(path_or_url, f"models/{path}") for path in vendor_index.get("modelVideoProfileFiles", [])
    ]
    return {
        "vendorCode": vendor_code,
        "regionCode": region_code,
        "vendor": _read_json(path_or_url, f"models/{vendor_index['path']}"),
        "families": _read_json(path_or_url, f"models/{vendor_index['familiesPath']}"),
        "models": [_read_json(path_or_url, f"models/{path}") for path in vendor_index.get("modelFiles", [])],
        "pricing": [_read_json(path_or_url, f"models/{path}") for path in vendor_index.get("pricingFiles", [])],
        "voices": voices,
        "modelVoiceBindings": model_voice_bindings,
        "modelVideoProfiles": model_video_profiles,
    }


def _put_best_model_identity(models: dict[str, dict], scores: dict[str, int], vendor_catalog: dict, model: dict) -> None:
    catalog_key = model.get("catalogKey")
    if not isinstance(catalog_key, str) or not catalog_key:
        return
    score = _model_identity_score(vendor_catalog, model)
    if catalog_key not in models or score > scores[catalog_key]:
        models[catalog_key] = model
        scores[catalog_key] = score


def _model_identity_score(vendor_catalog: dict, model: dict) -> int:
    score = 0
    if _has_region_pricing(vendor_catalog, model):
        score += 100
    if model.get("routingState") == "enabled":
        score += 40
    if model.get("shelfState") == "listed":
        score += 20
    if model.get("releaseStage") == "active":
        score += 10
    if model.get("lifecycle") in {"current", "preview"}:
        score += 5
    if model.get("regionCode") == "global":
        score += 1
    return score


def _has_region_pricing(vendor_catalog: dict, model: dict) -> bool:
    model_id = model.get("modelId")
    return isinstance(model_id, str) and any(
        item.get("modelId") == model_id and item.get("prices")
        for item in vendor_catalog.get("pricing", [])
    )
