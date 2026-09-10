import type { LayerDefinition, LayerId } from "./types";

/*
 * Katman kaydı — tek kaynak. Dock (madde 22.2), Zoom Engine (madde 22.4)
 * ve Command Palette (madde 27.3) hepsi buradan okur.
 *
 * Anayasa madde 36.1: Dock BÜTÜN ana modülleri gösterir. Zihinsel model
 * baştan bütündür. "live" olmayanlar PlaceholderLayer'a düşer.
 */

export const LAYERS: Record<LayerId, LayerDefinition> = {
  dashboard: {
    id: "dashboard",
    title: "Panel",
    level: 0,
    status: "live",
    component: () => import("@/modules/dashboard/DashboardLayer"),
  },
  today: {
    id: "today",
    title: "Bugün",
    level: 1,
    status: "live",
    component: () => import("@/modules/today/TodayLayer"),
  },
  work: {
    id: "work",
    title: "İş",
    level: 1,
    status: "placeholder",
    component: () => import("@/modules/_placeholder/PlaceholderLayer"),
  },
  personal: {
    id: "personal",
    title: "Kişisel",
    level: 1,
    status: "placeholder",
    component: () => import("@/modules/_placeholder/PlaceholderLayer"),
  },
  health: {
    id: "health",
    title: "Sağlık",
    level: 1,
    status: "placeholder",
    component: () => import("@/modules/_placeholder/PlaceholderLayer"),
  },
  finance: {
    id: "finance",
    title: "Finans",
    level: 1,
    status: "placeholder",
    component: () => import("@/modules/_placeholder/PlaceholderLayer"),
  },
  learning: {
    id: "learning",
    title: "Öğrenme",
    level: 1,
    status: "placeholder",
    component: () => import("@/modules/_placeholder/PlaceholderLayer"),
  },
  content: {
    id: "content",
    title: "İçerik",
    level: 1,
    status: "placeholder",
    component: () => import("@/modules/_placeholder/PlaceholderLayer"),
  },
  social: {
    id: "social",
    title: "Sosyal",
    level: 1,
    status: "placeholder",
    component: () => import("@/modules/_placeholder/PlaceholderLayer"),
  },
  system: {
    id: "system",
    title: "Sistem",
    level: 1,
    status: "placeholder",
    component: () => import("@/modules/_placeholder/PlaceholderLayer"),
  },
};

/** Anayasa madde 22: dashboard, seviye 0 — evrenin merkezi. Her açılışta buradayız. */
export const ROOT_LAYER_ID: LayerId = "dashboard";

export function getLayer(id: LayerId): LayerDefinition {
  return LAYERS[id];
}

export function listLayers(): LayerDefinition[] {
  return Object.values(LAYERS);
}
