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
            "supportedProtocols": list(vendor.get("supportedProtocols", [])),
            "clientApiCompatibility": dict(vendor.get("clientApiCompatibility", {})),
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
    result = [
        {"model": model, "has_region_pricing": _has_region_pricing(catalog, model)}
        for model in _regional_models(catalog)
    ]
    if vendor_code := _filter_value(standard_filters, "vendorCode", "vendor_code"):
        result = [item for item in result if item["model"].get("vendorCode") == vendor_code]
    if region_code := _filter_value(standard_filters, "regionCode", "region_code"):
        result = [item for item in result if item["model"].get("regionCode") == region_code]
    if family_code := _filter_value(standard_filters, "familyCode", "family_code"):
        result = [item for item in result if item["model"].get("familyCode") == family_code]
    if capability := _filter_value(standard_filters, "capability"):
        result = [item for item in result if capability in item["model"].get("capabilities", [])]
    if input_modality := _filter_value(standard_filters, "inputModality", "input_modality"):
        result = [item for item in result if input_modality in item["model"].get("inputModalities", [])]
    if output_modality := _filter_value(standard_filters, "outputModality", "output_modality"):
        result = [item for item in result if output_modality in item["model"].get("outputModalities", [])]
    if release_stage := _filter_value(standard_filters, "releaseStage", "release_stage"):
        result = [item for item in result if item["model"].get("releaseStage") == release_stage]
    if shelf_state := _filter_value(standard_filters, "shelfState", "shelf_state"):
        result = [item for item in result if item["model"].get("shelfState") == shelf_state]
    if routing_state := _filter_value(standard_filters, "routingState", "routing_state"):
        result = [item for item in result if item["model"].get("routingState") == routing_state]
    if api_format := _filter_value(standard_filters, "apiFormat", "api_format"):
        result = [item for item in result if item["model"].get("apiFormat") == api_format]
    if not _filter_value(standard_filters, "regionCode", "region_code"):
        result = _dedupe_model_identity_items(result)
    return [item["model"] for item in result]


def list_available_models(catalog: ModelCatalog, filter: dict | None = None, **filters: str) -> list[dict]:
    standard_filters = dict(filter or {})
    standard_filters.update(filters)
    standard_filters["routingState"] = "enabled"
    standard_filters["shelfState"] = "listed"
    return [
        model
        for model in list_models(catalog, standard_filters)
        if get_model_region_prices(catalog, model.get("catalogKey", ""), model.get("regionCode", ""))
    ]


def _filter_value(filters: dict, *keys: str) -> str | None:
    for key in keys:
        value = filters.get(key)
        if isinstance(value, str) and value:
            return value
    return None


def catalog_key(vendor_code: str, model_id: str) -> str:
    return f"{vendor_code}/{model_id}"


def list_meters(catalog: ModelCatalog) -> list[dict]:
    return catalog.meters


def find_meter(catalog: ModelCatalog, meter_code: str) -> dict | None:
    return next((meter for meter in catalog.meters if meter.get("meterCode") == meter_code), None)


def find_model(catalog: ModelCatalog, catalog_key_value: str) -> dict | None:
    parsed = _split_catalog_key(catalog_key_value)
    if parsed is None:
        return None
    vendor_code, model_id = parsed
    return next(
        (
            model
            for model in list_models(catalog)
            if model.get("vendorCode") == vendor_code
            and model.get("modelId") == model_id
        ),
        None,
    )


def find_model_by_vendor_region(catalog: ModelCatalog, vendor_code: str, region_code: str, model_id: str) -> dict | None:
    return next(
        (
            model
            for model in list_models(catalog, vendor_code=vendor_code, region_code=region_code)
            if model.get("vendorCode") == vendor_code
            and model.get("regionCode") == region_code
            and model.get("modelId") == model_id
        ),
        None,
    )


def get_model_prices(catalog: ModelCatalog, catalog_key_value: str) -> list[dict]:
    parsed = _split_catalog_key(catalog_key_value)
    if parsed is None:
        return []
    vendor_code, model_id = parsed
    pricing = next(
        (
            item
            for item in catalog.pricing
            if item.get("vendorCode") == vendor_code
            and item.get("modelId") == model_id
        ),
        None,
    )
    return [] if pricing is None else pricing.get("prices", [])


def get_model_region_prices(catalog: ModelCatalog, catalog_key_value: str, region_code: str) -> list[dict]:
    parsed = _split_catalog_key(catalog_key_value)
    if parsed is None:
        return []
    vendor_code, model_id = parsed
    for vendor_catalog in catalog.vendor_catalogs:
        if vendor_catalog.get("vendorCode") != vendor_code or vendor_catalog.get("regionCode") != region_code:
            continue
        pricing = next(
            (item for item in vendor_catalog.get("pricing", []) if item.get("modelId") == model_id),
            None,
        )
        return [] if pricing is None else pricing.get("prices", [])
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


def list_models_with_feature(catalog: ModelCatalog, feature: str) -> list[dict]:
    from .capabilities import model_supports_feature

    return [model for model in list_models(catalog) if model_supports_feature(model, feature)]


def list_protocols(catalog: ModelCatalog) -> list[dict]:
    return catalog.protocols


def find_protocol(catalog: ModelCatalog, protocol_code: str) -> dict | None:
    return next((p for p in catalog.protocols if p.get("protocolCode") == protocol_code), None)


def list_protocols_by_vendor(catalog: ModelCatalog, vendor_code: str) -> list[dict]:
    vendor = next((v for v in catalog.vendors if v.get("vendorCode") == vendor_code), None)
    if vendor is None:
        return []
    supported = set(vendor.get("supportedProtocols", []))
    return [p for p in catalog.protocols if p.get("protocolCode") in supported]


def list_client_api_compatibility_by_vendor(catalog: ModelCatalog, vendor_code: str) -> list[dict]:
    vendor = next((v for v in catalog.vendors if v.get("vendorCode") == vendor_code), None)
    if vendor is None:
        return []
    compatibility = vendor.get("clientApiCompatibility", {})
    if not isinstance(compatibility, dict):
        return []
    return list(compatibility.values())


def list_models_by_protocol(catalog: ModelCatalog, protocol_code: str) -> list[dict]:
    return list_models(catalog, api_format=protocol_code)


def voice_catalog_key(vendor_code: str, voice_id: str) -> str:
    return f"{vendor_code}/{voice_id}"


def list_voices(catalog: ModelCatalog, filter: dict | None = None, **filters: str) -> list[dict]:
    standard_filters = dict(filter or {})
    standard_filters.update(filters)
    result = [
        voice
        for vendor_catalog in catalog.vendor_catalogs
        for voice in vendor_catalog.get("voices", [])
    ]
    if vendor_code := _filter_value(standard_filters, "vendorCode", "vendor_code"):
        result = [voice for voice in result if voice.get("vendorCode") == vendor_code]
    if region_code := _filter_value(standard_filters, "regionCode", "region_code"):
        result = [voice for voice in result if voice.get("regionCode") == region_code]
    if locale := _filter_value(standard_filters, "locale"):
        result = [
            voice
            for voice in result
            if voice.get("primaryLocale") == locale or locale in voice.get("supportedLocales", [])
        ]
    if query := _filter_value(standard_filters, "q"):
        query_lower = query.lower()
        result = [
            voice
            for voice in result
            if query_lower in str(voice.get("displayName", "")).lower()
            or query_lower in str(voice.get("voiceId", "")).lower()
        ]
    if model_catalog_key := _filter_value(standard_filters, "modelCatalogKey", "model_catalog_key"):
        result = [
            voice
            for voice in result
            if _voice_bound_to_model(catalog, voice.get("catalogKey", ""), model_catalog_key)
        ]
    return result


def list_voices_for_model(catalog: ModelCatalog, model_catalog_key: str) -> list[dict]:
    return list_voices(catalog, model_catalog_key=model_catalog_key)


def list_models_for_voice(catalog: ModelCatalog, voice_catalog_key_value: str) -> list[dict]:
    model_keys: set[str] = set()
    for vendor_catalog in catalog.vendor_catalogs:
        for binding in vendor_catalog.get("modelVoiceBindings", []):
            if not any(
                entry.get("voiceKey") == voice_catalog_key_value
                for entry in binding.get("bindings", [])
            ):
                continue
            catalog_key = binding.get("catalogKey")
            if isinstance(catalog_key, str) and catalog_key:
                model_keys.add(catalog_key)
    return [model for model in list_models(catalog) if model.get("catalogKey") in model_keys]


def video_profile_catalog_key(vendor_code: str, model_id: str, profile_code: str) -> str:
    return f"{vendor_code}/{model_id}/{profile_code}"


def list_video_profiles(catalog: ModelCatalog, filter: dict | None = None, **filters: str) -> list[dict]:
    merged = {**(filter or {}), **filters}
    vendor_code = merged.get("vendor_code") or merged.get("vendorCode")
    region_code = merged.get("region_code") or merged.get("regionCode")
    model_catalog_key = merged.get("model_catalog_key") or merged.get("modelCatalogKey")
    generation_mode = merged.get("generation_mode") or merged.get("generationMode")
    duration_tier_code = merged.get("duration_tier_code") or merged.get("durationTierCode")
    resolution = merged.get("resolution")
    result: list[dict] = []
    for vendor_catalog in catalog.vendor_catalogs:
        if vendor_code and vendor_catalog.get("vendorCode") != vendor_code:
            continue
        if region_code and vendor_catalog.get("regionCode") != region_code:
            continue
        for profile_file in vendor_catalog.get("modelVideoProfiles", []):
            if model_catalog_key and profile_file.get("catalogKey") != model_catalog_key:
                continue
            for profile in profile_file.get("profiles", []):
                if generation_mode and profile.get("generationMode") != generation_mode:
                    continue
                if duration_tier_code and profile.get("durationTierCode") != duration_tier_code:
                    tier_codes = profile.get("durationTierCodes") or []
                    if duration_tier_code not in tier_codes:
                        continue
                if resolution and profile.get("resolution") != resolution:
                    continue
                result.append(profile)
    return result


def list_video_profiles_for_model(catalog: ModelCatalog, model_catalog_key: str) -> list[dict]:
    return list_video_profiles(catalog, model_catalog_key=model_catalog_key)


def find_video_profile(catalog: ModelCatalog, profile_catalog_key: str) -> dict | None:
    for vendor_catalog in catalog.vendor_catalogs:
        for profile_file in vendor_catalog.get("modelVideoProfiles", []):
            for profile in profile_file.get("profiles", []):
                if profile.get("catalogKey") == profile_catalog_key:
                    return profile
    return None


def _voice_bound_to_model(catalog: ModelCatalog, voice_catalog_key_value: str, model_catalog_key: str) -> bool:
    for vendor_catalog in catalog.vendor_catalogs:
        for binding in vendor_catalog.get("modelVoiceBindings", []):
            if binding.get("catalogKey") != model_catalog_key:
                continue
            if any(
                entry.get("voiceKey") == voice_catalog_key_value
                for entry in binding.get("bindings", [])
            ):
                return True
    return False


def _split_catalog_key(catalog_key_value: str) -> tuple[str, str] | None:
    separator_index = catalog_key_value.find("/")
    if separator_index <= 0 or separator_index == len(catalog_key_value) - 1:
        return None
    return catalog_key_value[:separator_index], catalog_key_value[separator_index + 1:]


def _regional_models(catalog: ModelCatalog) -> list[dict]:
    if catalog.vendor_catalogs:
        return [
            model
            for vendor_catalog in catalog.vendor_catalogs
            for model in vendor_catalog.get("models", [])
        ]
    return catalog.models


def _has_region_pricing(catalog: ModelCatalog, model: dict) -> bool:
    return bool(
        get_model_region_prices(
            catalog,
            model.get("catalogKey", ""),
            model.get("regionCode", ""),
        )
    )


def _dedupe_model_identity_items(items: list[dict]) -> list[dict]:
    deduped: dict[str, dict] = {}
    for item in items:
        catalog_key = item["model"].get("catalogKey")
        if not catalog_key:
            continue
        existing = deduped.get(catalog_key)
        if existing is None or _model_identity_score(item) > _model_identity_score(existing):
            deduped[catalog_key] = item
    return list(deduped.values())


def _model_identity_score(item: dict) -> int:
    model = item["model"]
    score = 0
    if item["has_region_pricing"]:
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
