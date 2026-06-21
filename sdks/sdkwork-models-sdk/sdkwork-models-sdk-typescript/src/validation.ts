import type { ModelCatalog } from "./types.js";

const DECIMAL_PATTERN = /^(0|[1-9][0-9]*)(\.[0-9]+)?$/;

export interface CatalogValidationIssue {
  code: string;
  path: string;
  message: string;
  severity: "error" | "warning";
}

export function validateCatalog(catalog: ModelCatalog): CatalogValidationIssue[] {
  const issues: CatalogValidationIssue[] = [];
  const meters = new Set(catalog.meters.map((meter) => meter.meterCode));
  const models = new Set(catalog.vendors.flatMap((vendor) => vendor.models.map((model) => model.catalogKey)));
  for (const vendor of catalog.vendors) {
    for (const pricing of vendor.pricing) {
      if (!models.has(pricing.catalogKey)) {
        issues.push({
          code: "pricing.model.missing",
          path: pricing.catalogKey,
          message: `${pricing.modelId} is not defined`,
          severity: "error",
        });
      }
      for (const price of pricing.prices) {
        if (!meters.has(price.meterCode)) {
          issues.push({
            code: "pricing.meter.missing",
            path: `${pricing.catalogKey}/${price.meterCode}`,
            message: `${price.meterCode} is not defined`,
            severity: "error",
          });
        }
        for (const field of ["unitSize", "unitPrice", "minimumQuantity"] as const) {
          if (!DECIMAL_PATTERN.test(price[field])) {
            issues.push({
              code: "pricing.decimal.invalid",
              path: `${pricing.catalogKey}/${field}`,
              message: `${field} must be a decimal string`,
              severity: "error",
            });
          }
        }
      }
    }
  }
  return issues;
}
