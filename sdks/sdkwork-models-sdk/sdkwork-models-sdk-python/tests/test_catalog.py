from pathlib import Path
import json

from sdkwork_models import (
    JsonObject,
    ModelCatalog,
    ProtocolStandard,
    catalog_key,
    find_model,
    find_model_by_vendor_region,
    find_meter,
    find_protocol,
    get_best_reference_price,
    get_model_prices,
    get_model_region_prices,
    list_available_models,
    list_meters,
    list_models,
    list_models_by_capability,
    list_models_by_modality,
    list_models_by_protocol,
    list_models_for_voice,
    list_protocols,
    list_protocols_by_vendor,
    list_vendor_regions,
    list_vendors,
    list_voices,
    list_voices_for_model,
    list_video_profiles,
    list_video_profiles_for_model,
    find_video_profile,
    load_catalog,
    load_bundled_catalog,
)


def read_repository_catalog_version() -> str:
    index_path = Path(__file__).resolve().parents[4] / "models" / "index.json"
    with index_path.open(encoding="utf-8") as handle:
        payload = json.load(handle)
    return payload["catalogVersion"]


def test_package_root_exports_catalog_types() -> None:
    assert ModelCatalog.__name__ == "ModelCatalog"
    assert JsonObject.__args__[1].__name__ == "Any"


def test_load_catalog() -> None:
    catalog = load_catalog(Path(__file__).resolve().parents[4])
    assert catalog.catalog_version == read_repository_catalog_version()
    assert find_model(catalog, "openai/gpt-5.5")["vendorCode"] == "openai"
    assert find_model(catalog, "openai/gpt-5.5")["regionCode"] == "global"
    assert find_model_by_vendor_region(catalog, "openai", "global", "gpt-5.5")["vendorCode"] == "openai"
    assert catalog_key("openai", "gpt-5.5") == "openai/gpt-5.5"
    assert find_model(catalog, "openai/global/gpt-5.5") is None
    assert all("regionCode" not in vendor for vendor in list_vendors(catalog))
    assert ("minimax", "cn") in {
        (item["vendorCode"], item["regionCode"]) for item in list_vendor_regions(catalog)
    }
    assert [vendor["vendorCode"] for vendor in list_vendors(catalog)].count("minimax") == 1
    assert any(meter["meterCode"] == "llm_input_token" for meter in list_meters(catalog))
    assert find_meter(catalog, "llm_input_token")["defaultUnitSize"] == "1000000"
    assert find_meter(catalog, "missing_meter") is None
    assert list_models(catalog, {"vendorCode": "openai", "regionCode": "global", "familyCode": "gpt-5"})
    assert list_models(catalog, vendor_code="openai", region_code="global", family_code="gpt-5")
    assert list_models(catalog, {"releaseStage": "active", "shelfState": "listed", "routingState": "enabled"})
    assert list_models(catalog, release_stage="active", shelf_state="listed", routing_state="enabled")
    assert list_models(catalog, {"apiFormat": "openai_compatible"})
    assert list_models(catalog, api_format="openai_compatible")
    assert list_models_by_capability(catalog, "chat")
    assert list_models_by_modality(catalog, "text", "text")
    available_models = list_available_models(catalog)
    assert available_models
    model_keys = [model["catalogKey"] for model in list_models(catalog)]
    assert len(model_keys) == len(set(model_keys))
    assert all(get_model_prices(catalog, model["catalogKey"]) for model in available_models)
    assert all(
        model["routingState"] == "enabled" and model["shelfState"] == "listed"
        for model in available_models
    )
    assert any(
        model["catalogKey"] == "kuaishou/kling-v3-0-preview"
        and model["regionCode"] == "global"
        for model in available_models
    )
    assert find_model(catalog, "kuaishou/kling-v3-0-preview")["regionCode"] == "global"
    assert find_model_by_vendor_region(catalog, "kuaishou", "cn", "kling-v3-0-preview")["regionCode"] == "cn"
    assert not any(
        model["catalogKey"] == "kuaishou/kling-v3-0-preview"
        for model in list_available_models(catalog, region_code="cn")
    )
    assert any(
        model["catalogKey"] == "kuaishou/kling-v3-0-preview"
        for model in list_available_models(catalog, region_code="global")
    )
    prices = get_model_prices(catalog, "openai/gpt-5.5")
    assert prices
    assert get_model_region_prices(catalog, "openai/gpt-5.5", "global")
    assert get_model_region_prices(catalog, "openai/gpt-5.5", "cn") == []
    assert get_model_prices(catalog, "openai/global/gpt-5.5") == []
    assert get_best_reference_price(catalog, "openai/gpt-5.5", "llm_input_token")["unitPrice"] == "5.000000"


def test_voice_catalog_queries() -> None:
    catalog = load_bundled_catalog()
    openai_voices = list_voices(catalog, vendor_code="openai", region_code="global")
    assert len(openai_voices) >= 11
    assert len(list_voices_for_model(catalog, "openai/gpt-4o-mini-tts")) >= 11
    assert len(list_voices(catalog)) >= 23
    assert list_models_for_voice(catalog, openai_voices[0]["catalogKey"])


def test_video_generation_profile_queries() -> None:
    catalog = load_bundled_catalog()
    assert len(list_video_profiles(catalog, vendor_code="kuaishou", region_code="global")) >= 3
    assert len(list_video_profiles_for_model(catalog, "openai/sora-2")) >= 2
    assert len(list_video_profiles(catalog)) >= 80
    assert find_video_profile(catalog, "vidu/viduq3-pro/t2v_5s_720p")


def test_protocol_queries() -> None:
    catalog = load_catalog(Path(__file__).resolve().parents[4])
    protocols = list_protocols(catalog)
    assert len(protocols) >= 4
    assert any(p["protocolCode"] == "openai_responses" for p in protocols)
    assert any(p["protocolCode"] == "openai_compatible" for p in protocols)
    assert any(p["protocolCode"] == "anthropic_messages" for p in protocols)
    assert any(p["protocolCode"] == "google_gemini" for p in protocols)

    assert find_protocol(catalog, "openai_responses")["displayName"] == "OpenAI Responses API"
    assert find_protocol(catalog, "nonexistent") is None

    openai_protocols = list_protocols_by_vendor(catalog, "openai")
    assert len(openai_protocols) >= 2
    assert any(p["protocolCode"] == "openai_responses" for p in openai_protocols)
    assert any(p["protocolCode"] == "openai_compatible" for p in openai_protocols)

    anthropic_protocols = list_protocols_by_vendor(catalog, "anthropic")
    assert any(p["protocolCode"] == "anthropic_messages" for p in anthropic_protocols)

    ds_protocols = list_protocols_by_vendor(catalog, "deepseek")
    assert any(p["protocolCode"] == "anthropic_messages" for p in ds_protocols), "deepseek supports anthropic_messages"

    responses_models = list_models_by_protocol(catalog, "openai_responses")
    assert len(responses_models) > 0
    assert all(m["apiFormat"] == "openai_responses" for m in responses_models)

    vendor = next(v for v in list_vendors(catalog) if v["vendorCode"] == "openai")
    assert "supportedProtocols" in vendor
    assert "openai_responses" in vendor["supportedProtocols"]


def test_load_bundled_catalog_from_environment(monkeypatch=None) -> None:
    catalog_root = Path(__file__).resolve().parents[4]
    previous_root = None
    if monkeypatch is None:
        import os

        previous_root = os.environ.get("SDKWORK_MODELS_CATALOG_ROOT")
        os.environ["SDKWORK_MODELS_CATALOG_ROOT"] = str(catalog_root)
        try:
            catalog = load_bundled_catalog()
        finally:
            if previous_root is None:
                os.environ.pop("SDKWORK_MODELS_CATALOG_ROOT", None)
            else:
                os.environ["SDKWORK_MODELS_CATALOG_ROOT"] = previous_root
    else:
        monkeypatch.setenv("SDKWORK_MODELS_CATALOG_ROOT", str(catalog_root))
        catalog = load_bundled_catalog()
    assert find_model(catalog, "openai/gpt-5.5")["vendorCode"] == "openai"
    assert catalog.catalog_version == read_repository_catalog_version()


def test_catalog_key_parser_keeps_slash_delimited_provider_model_ids_intact() -> None:
    catalog = ModelCatalog(
        catalog_version="fixture-1.0.0",
        schema_version="1.1.0",
        meters=[],
        protocols=[],
        vendors=[],
        vendor_catalogs=[
            {
                "vendorCode": "openrouter",
                "regionCode": "global",
                "models": [
                    {
                        "catalogKey": "openrouter/anthropic/claude-3-opus",
                        "modelId": "anthropic/claude-3-opus",
                        "displayName": "Claude 3 Opus through OpenRouter",
                        "vendorCode": "openrouter",
                        "regionCode": "global",
                        "familyCode": "anthropic",
                        "releaseStage": "active",
                        "shelfState": "listed",
                        "routingState": "enabled",
                        "apiFormat": "openai_compatible",
                        "capabilities": ["chat"],
                        "inputModalities": ["text"],
                        "outputModalities": ["text"],
                    }
                ],
                "pricing": [
                    {
                        "catalogKey": "openrouter/anthropic/claude-3-opus",
                        "vendorCode": "openrouter",
                        "regionCode": "global",
                        "modelId": "anthropic/claude-3-opus",
                        "currency": "USD",
                        "prices": [{"meterCode": "llm_input_token", "unitPrice": "15.000000"}],
                    }
                ],
            }
        ],
        models=[],
        pricing=[
            {
                "catalogKey": "openrouter/anthropic/claude-3-opus",
                "vendorCode": "openrouter",
                "regionCode": "global",
                "modelId": "anthropic/claude-3-opus",
                "currency": "USD",
                "prices": [{"meterCode": "llm_input_token", "unitPrice": "15.000000"}],
            }
        ],
    )

    assert catalog_key("openrouter", "anthropic/claude-3-opus") == "openrouter/anthropic/claude-3-opus"
    assert find_model(catalog, "openrouter/anthropic/claude-3-opus")["modelId"] == "anthropic/claude-3-opus"
    assert len(get_model_prices(catalog, "openrouter/anthropic/claude-3-opus")) == 1
    assert len(get_model_region_prices(catalog, "openrouter/anthropic/claude-3-opus", "global")) == 1
    assert find_model(catalog, "openrouter/global/anthropic/claude-3-opus") is None
    assert get_model_prices(catalog, "openrouter/global/anthropic/claude-3-opus") == []


def test_model_capability_predicates() -> None:
    from sdkwork_models import (
        get_model_capability_profile,
        list_models_with_feature,
        model_supports_audio_input,
        model_supports_feature,
        model_supports_tool_call,
        model_supports_vision,
    )

    catalog = load_catalog(Path(__file__).resolve().parents[4])
    claude = find_model(catalog, "anthropic/claude-opus-4-8")
    assert claude is not None
    assert model_supports_vision(claude)
    assert model_supports_tool_call(claude)
    assert not model_supports_audio_input(claude)
    assert model_supports_feature(claude, "tool_call")

    profile = get_model_capability_profile(claude)
    assert profile["catalogKey"] == "anthropic/claude-opus-4-8"
    assert "tool_call" in profile["features"]
    assert len(list_models_with_feature(catalog, "tool_call")) > 0
