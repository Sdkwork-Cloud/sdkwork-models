from __future__ import annotations

from .types import ModelCatalog


def list_vendors(catalog: ModelCatalog) -> list[dict]:
    vendors: dict[str, dict] = {}
    for vendor in catalog.vendors:
        vendor_code = vendor["vendorCode"]
        if vendor_code in vendors:
            continue
        vendors[vendor_code] = {
            "vendorCode": vendor_code,
            "displayName": vendor.get("displayName"),
            "legalName": vendor.get("legalName"),
            "vendorType": vendor.get("vendorType"),
            "capabilities": list(vendor.get("capabilities", [])),
            "openSource": vendor.get("openSource", False),
        }
    return list(vendors.values())


def list_vendor_regions(catalog: ModelCatalog) -> list[dict]:
    if catalog.vendor_catalogs:
        return [
            {"vendorCode": item["vendorCode"], "regionCode": item["regionCode"]}
            for item in catalog.vendor_catalogs
        ]
    return [
        {"vendorCode": vendor["vendorCode"], "regionCode": region["regionCode"]}
        for vendor in catalog.vendors
        for region in vendor.get("regions", [])
    ]


def list_models(catalog: ModelCatalog, filter: dict | None = None, **filters: str) -> list[dict]:
    standard_filters = dict(filter or {})
    standard_filters.update(filters)
    result = catalog.models
    if vendor_code := _filter_value(standard_filters, "vendorCode", "vendor_code"):
        result = [model for model in result if model.get("vendorCode") == vendor_code]
    if region_code := _filter_value(standard_filters, "regionCode", "region_code"):
        result = [model for model in result if model.get("regionCode") == region_code]
    if family_code := _filter_value(standard_filters, "familyCode", "family_code"):
        result = [model for model in result if model.get("familyCode") == family_code]
    if capability := _filter_value(standard_filters, "capability"):
        result = [model for model in result if capability in model.get("capabilities", [])]
    if input_modality := _filter_value(standard_filters, "inputModality", "input_modality"):
        result = [model for model in result if input_modality in model.get("inputModalities", [])]
    if output_modality := _filter_value(standard_filters, "outputModality", "output_modality"):
        result = [model for model in result if output_modality in model.get("outputModalities", [])]
    if release_stage := _filter_value(standard_filters, "releaseStage", "release_stage"):
        result = [model for model in result if model.get("releaseStage") == release_stage]
    if shelf_state := _filter_value(standard_filters, "shelfState", "shelf_state"):
        result = [model for model in result if model.get("shelfState") == shelf_state]
    if routing_state := _filter_value(standard_filters, "routingState", "routing_state"):
        result = [model for model in result if model.get("routingState") == routing_state]
    if api_format := _filter_value(standard_filters, "apiFormat", "api_format"):
        result = [model for model in result if model.get("apiFormat") == api_format]
    return result


def list_available_models(catalog: ModelCatalog, filter: dict | None = None, **filters: str) -> list[dict]:
    standard_filters = dict(filter or {})
    standard_filters.update(filters)
    standard_filters["routingState"] = "enabled"
    standard_filters["shelfState"] = "listed"
    return [
        model
        for model in list_models(catalog, standard_filters)
        if get_model_prices(catalog, model.get("catalogKey", ""))
    ]


def _filter_value(filters: dict, *keys: str) -> str | None:
    for key in keys:
        value = filters.get(key)
        if isinstance(value, str) and value:
            return value
    return None


def catalog_key(vendor_code: str, region_code: str, model_id: str) -> str:
    return f"{vendor_code}/{region_code}/{model_id}"


def list_meters(catalog: ModelCatalog) -> list[dict]:
    return catalog.meters


def find_meter(catalog: ModelCatalog, meter_code: str) -> dict | None:
    return next((meter for meter in catalog.meters if meter.get("meterCode") == meter_code), None)


def find_model(catalog: ModelCatalog, catalog_key_value: str) -> dict | None:
    parts = catalog_key_value.split("/")
    if len(parts) != 3 or not parts[0] or not parts[1] or not parts[2]:
        return None
    return find_model_by_vendor_region(catalog, parts[0], parts[1], parts[2])


def find_model_by_vendor_region(catalog: ModelCatalog, vendor_code: str, region_code: str, model_id: str) -> dict | None:
    return next(
        (
            model
            for model in catalog.models
            if model.get("vendorCode") == vendor_code
            and model.get("regionCode") == region_code
            and model.get("modelId") == model_id
        ),
        None,
    )


def get_model_prices(catalog: ModelCatalog, catalog_key_value: str) -> list[dict]:
    parts = catalog_key_value.split("/")
    if len(parts) != 3 or not parts[0] or not parts[1] or not parts[2]:
        return []
    vendor_code, region_code, model_id = parts
    pricing = next(
        (
            item
            for item in catalog.pricing
            if item.get("vendorCode") == vendor_code
            and item.get("regionCode") == region_code
            and item.get("modelId") == model_id
        ),
        None,
    )
    return [] if pricing is None else pricing.get("prices", [])


def get_best_reference_price(catalog: ModelCatalog, catalog_key_value: str, meter_code: str) -> dict | None:
    return next((price for price in get_model_prices(catalog, catalog_key_value) if price.get("meterCode") == meter_code), None)


def list_models_by_capability(catalog: ModelCatalog, capability: str) -> list[dict]:
    return list_models(catalog, capability=capability)


def list_models_by_modality(catalog: ModelCatalog, input_modality: str, output_modality: str) -> list[dict]:
    return list_models(catalog, input_modality=input_modality, output_modality=output_modality)
