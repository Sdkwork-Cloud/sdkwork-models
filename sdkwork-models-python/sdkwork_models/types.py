from __future__ import annotations

from dataclasses import dataclass
from typing import Any


JsonObject = dict[str, Any]


@dataclass(frozen=True)
class ProtocolStandard:
    protocol_code: str
    vendor_origin: str
    display_name: str
    family: str
    docs_url: str
    maturity: str


@dataclass(frozen=True)
class ModelCatalog:
    catalog_version: str
    schema_version: str
    meters: list[JsonObject]
    protocols: list[JsonObject]
    vendors: list[JsonObject]
    vendor_catalogs: list[JsonObject]
    models: list[JsonObject]
    pricing: list[JsonObject]
