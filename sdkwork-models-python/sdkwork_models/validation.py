from __future__ import annotations

import re

from .types import ModelCatalog


DECIMAL_PATTERN = re.compile(r"^(0|[1-9][0-9]*)(\.[0-9]+)?$")


def validate_catalog(catalog: ModelCatalog) -> list[dict]:
    meters = {meter["meterCode"] for meter in catalog.meters}
    catalog_keys = {model["catalogKey"] for model in catalog.models}
    issues: list[dict] = []
    for pricing in catalog.pricing:
        if pricing.get("catalogKey") not in catalog_keys:
            issues.append({"code": "pricing.model.missing", "severity": "error", "message": pricing.get("catalogKey")})
        for price in pricing.get("prices", []):
            if price.get("meterCode") not in meters:
                issues.append({"code": "pricing.meter.missing", "severity": "error", "message": price.get("meterCode")})
            for field in ("unitSize", "unitPrice", "minimumQuantity"):
                if not isinstance(price.get(field), str) or not DECIMAL_PATTERN.match(price[field]):
                    issues.append({"code": "pricing.decimal.invalid", "severity": "error", "message": field})
    return issues
