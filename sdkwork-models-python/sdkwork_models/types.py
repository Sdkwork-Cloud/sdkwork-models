from __future__ import annotations

from dataclasses import dataclass
from typing import Any


JsonObject = dict[str, Any]


@dataclass(frozen=True)
class ModelCatalog:
    catalog_version: str
    schema_version: str
    meters: list[JsonObject]
    # Unique vendor identities. A vendor can appear in multiple region catalogs.
    vendors: list[JsonObject]
    vendor_catalogs: list[JsonObject]
    # Flattened model and pricing facts keyed by vendorCode/regionCode/modelId.
    models: list[JsonObject]
    pricing: list[JsonObject]
