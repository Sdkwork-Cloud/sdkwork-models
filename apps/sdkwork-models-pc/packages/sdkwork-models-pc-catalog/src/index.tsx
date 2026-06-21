import { useEffect, useState } from "react";
import { listAvailableModels, listVendors } from "@sdkwork/models";
import { loadRepositoryCatalog, summarizeCatalog } from "@sdkwork/models-pc-core";

export function ModelsCatalogApp() {
  const [error, setError] = useState<string | null>(null);
  const [summary, setSummary] = useState<{ catalogVersion: string; vendorCount: number; availableModelCount: number } | null>(null);
  const [vendorCards, setVendorCards] = useState<Array<{ vendorCode: string; displayName: string; modelCount: number }>>([]);

  useEffect(() => {
    let active = true;
    loadRepositoryCatalog()
      .then((catalog) => {
        if (!active) {
          return;
        }
        setSummary(summarizeCatalog(catalog));
        setVendorCards(
          listVendors(catalog).map((vendor) => ({
            vendorCode: vendor.vendorCode,
            displayName: vendor.displayName,
            modelCount: listAvailableModels(catalog, { vendorCode: vendor.vendorCode }).length,
          })),
        );
        setError(null);
      })
      .catch((cause: unknown) => {
        if (!active) {
          return;
        }
        setError(cause instanceof Error ? cause.message : String(cause));
      });
    return () => {
      active = false;
    };
  }, []);

  return (
    <main>
      <h1>SDKWork Models</h1>
      <p className="muted">Portable model catalog explorer backed by `@sdkwork/models`.</p>
      {error ? <p>{error}</p> : null}
      {summary ? (
        <p className="muted">
          Catalog {summary.catalogVersion} · {summary.vendorCount} vendors · {summary.availableModelCount} listed models
        </p>
      ) : null}
      <section className="grid" aria-label="Model vendors">
        {vendorCards.map((vendor) => (
          <article className="card" key={vendor.vendorCode}>
            <h2>{vendor.displayName}</h2>
            <p>{vendor.vendorCode}</p>
            <p>{vendor.modelCount} listed models</p>
          </article>
        ))}
      </section>
    </main>
  );
}
