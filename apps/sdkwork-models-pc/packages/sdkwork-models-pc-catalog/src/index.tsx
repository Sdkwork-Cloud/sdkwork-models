import { useEffect, useMemo, useState } from "react";
import { listAvailableModels, listVendors, type ModelCatalog, type ModelInfo } from "@sdkwork/models-sdk";
import { loadRepositoryCatalog, summarizeCatalog } from "@sdkwork/models-pc-core";

type VendorCard = {
  vendorCode: string;
  displayName: string;
  modelCount: number;
};

function normalizeSearch(value: string): string {
  return value.trim().toLowerCase();
}

function matchesSearch(value: string, query: string): boolean {
  if (!query) {
    return true;
  }
  return value.toLowerCase().includes(query);
}

function filterVendorCards(cards: VendorCard[], query: string): VendorCard[] {
  const normalizedQuery = normalizeSearch(query);
  if (!normalizedQuery) {
    return cards;
  }
  return cards.filter((vendor) =>
    matchesSearch(vendor.vendorCode, normalizedQuery)
    || matchesSearch(vendor.displayName, normalizedQuery),
  );
}

function filterModels(
  models: ModelInfo[],
  query: string,
  filters: { codingOnly: boolean; ideVisibleOnly: boolean },
): ModelInfo[] {
  const normalizedQuery = normalizeSearch(query);
  return models.filter((model) => {
    if (normalizedQuery) {
      const matches =
        matchesSearch(model.modelId, normalizedQuery)
        || matchesSearch(model.displayName ?? "", normalizedQuery)
        || matchesSearch(model.catalogKey, normalizedQuery);
      if (!matches) {
        return false;
      }
    }
    if (filters.codingOnly && !(model.usageScopes ?? []).includes("coding")) {
      return false;
    }
    if (filters.ideVisibleOnly && model.codingVisible === false) {
      return false;
    }
    return true;
  });
}

export function ModelsCatalogApp() {
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [catalog, setCatalog] = useState<ModelCatalog | null>(null);
  const [summary, setSummary] = useState<{ catalogVersion: string; vendorCount: number; availableModelCount: number } | null>(null);
  const [vendorCards, setVendorCards] = useState<VendorCard[]>([]);
  const [search, setSearch] = useState("");
  const [selectedVendorCode, setSelectedVendorCode] = useState<string | null>(null);
  const [codingOnly, setCodingOnly] = useState(false);
  const [ideVisibleOnly, setIdeVisibleOnly] = useState(false);

  useEffect(() => {
    let active = true;
    setLoading(true);
    loadRepositoryCatalog()
      .then((loadedCatalog) => {
        if (!active) {
          return;
        }
        setCatalog(loadedCatalog);
        setSummary(summarizeCatalog(loadedCatalog));
        setVendorCards(
          listVendors(loadedCatalog).map((vendor) => ({
            vendorCode: vendor.vendorCode,
            displayName: vendor.displayName,
            modelCount: listAvailableModels(loadedCatalog, { vendorCode: vendor.vendorCode }).length,
          })),
        );
        setError(null);
      })
      .catch((cause: unknown) => {
        if (!active) {
          return;
        }
        setError(cause instanceof Error ? cause.message : String(cause));
      })
      .finally(() => {
        if (active) {
          setLoading(false);
        }
      });
    return () => {
      active = false;
    };
  }, []);

  const visibleVendors = useMemo(
    () => filterVendorCards(vendorCards, search),
    [vendorCards, search],
  );

  const selectedVendorModels = useMemo(() => {
    if (!catalog || !selectedVendorCode) {
      return [] as ModelInfo[];
    }
    return filterModels(
      listAvailableModels(catalog, { vendorCode: selectedVendorCode }),
      search,
      { codingOnly, ideVisibleOnly },
    );
  }, [catalog, search, selectedVendorCode, codingOnly, ideVisibleOnly]);

  return (
    <main>
      <h1>SDKWork Models</h1>
      <p className="muted">Portable model catalog explorer backed by `@sdkwork/models-sdk`.</p>
      {error ? <p role="alert">{error}</p> : null}
      {loading ? <p className="muted" aria-live="polite">Loading catalog…</p> : null}
      {summary ? (
        <p className="muted">
          Catalog {summary.catalogVersion} · {summary.vendorCount} vendors · {summary.availableModelCount} listed models
        </p>
      ) : null}
      <section className="toolbar" aria-label="Catalog search">
        <label className="search-label" htmlFor="catalog-search">
          Search vendors and models
        </label>
        <input
          id="catalog-search"
          type="search"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder="Vendor code, display name, or model id"
          className="search-input"
        />
        <div className="filter-row" role="group" aria-label="Model applicability filters">
          <button
            type="button"
            className={`filter-chip${codingOnly ? " filter-chip-active" : ""}`}
            aria-pressed={codingOnly}
            onClick={() => setCodingOnly((value) => !value)}
          >
            Coding
          </button>
          <button
            type="button"
            className={`filter-chip${ideVisibleOnly ? " filter-chip-active" : ""}`}
            aria-pressed={ideVisibleOnly}
            onClick={() => setIdeVisibleOnly((value) => !value)}
          >
            IDE visible
          </button>
        </div>
        <p className="muted search-meta">
          Showing {visibleVendors.length} of {vendorCards.length} vendors
        </p>
      </section>
      <section className="grid" aria-label="Model vendors">
        {visibleVendors.map((vendor) => {
          const selected = selectedVendorCode === vendor.vendorCode;
          return (
            <article className={`card${selected ? " card-selected" : ""}`} key={vendor.vendorCode}>
              <button
                type="button"
                className="card-button"
                aria-pressed={selected}
                onClick={() => setSelectedVendorCode(selected ? null : vendor.vendorCode)}
              >
                <h2>{vendor.displayName}</h2>
                <p>{vendor.vendorCode}</p>
                <p>{vendor.modelCount} listed models</p>
              </button>
            </article>
          );
        })}
      </section>
      {selectedVendorCode && catalog ? (
        <section className="model-panel" aria-label={`Models for ${selectedVendorCode}`}>
          <h2>{selectedVendorCode}</h2>
          <p className="muted">{selectedVendorModels.length} matching listed models</p>
          <ul className="model-list">
            {selectedVendorModels.map((model) => (
              <li key={model.catalogKey}>
                <strong>
                  {model.displayName || model.modelId}
                  {(model.usageScopes ?? []).includes("coding")
                    ? <span className="model-badge">coding</span>
                    : null}
                  {model.codingVisible === false
                    ? <span className="model-badge model-badge-hidden">IDE hidden</span>
                    : null}
                </strong>
                <span>{model.modelId}</span>
                <span>{model.regionCode}</span>
              </li>
            ))}
          </ul>
        </section>
      ) : null}
    </main>
  );
}
