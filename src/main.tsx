import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { applyTheme, DEFAULT_THEME } from "@/design/theme";
import "@/design/tokens.css";
import "./index.css";

/*
 * Anayasa madde 29.2: karanlık tema varsayılan.
 * İlk boyamadan önce uygulanır — tema sıçraması (flash) olmaz.
 */
applyTheme(DEFAULT_THEME);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
