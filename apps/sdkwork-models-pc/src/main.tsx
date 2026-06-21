import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { ModelsCatalogApp } from "@sdkwork/models-pc-catalog";
import "./styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ModelsCatalogApp />
  </StrictMode>,
);
