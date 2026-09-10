import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { applyTheme, DEFAULT_THEME } from "@/design/theme";
import { calibrateRefreshRate, markColdStartOrigin } from "@/lib/perf";
import "@/design/tokens.css";
import "./index.css";

/*
 * Anayasa madde 29.2: karanlık tema varsayılan.
 * İlk boyamadan önce uygulanır — tema sıçraması (flash) olmaz.
 */
applyTheme(DEFAULT_THEME);

/*
 * Soğuk açılış ölçümü başlar — Anayasa madde 34.2.
 * Bitiş noktası: panelin gerçek veriyle boyandığı an
 * (DashboardLayer → markFirstMeaningfulPaint).
 */
markColdStartOrigin();

/*
 * Ekranın tazeleme aralığını ölç — Sprint 1 borcu #3.
 * Jank ölçümü sabit 8,3 ms yerine BU değere göre yapılır; böylece 60 Hz ve
 * 120 Hz ekranlarda aynı anlamı taşır.
 */
calibrateRefreshRate();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
