/*
 * Katman (Layer) mimarisi — Anayasa madde 22.
 *
 * ARKELÉS'te "sayfa" yoktur. Her modül bir KATMAN'dır ve hepsi aynı evren
 * içindedir (madde 22.7). Kullanıcı katmanlar arasında ZOOM ile gezer
 * (madde 22.4). En fazla 2 zoom seviyesi vardır (madde 22.5); daha derini
 * panel geçişidir (madde 22.6).
 */

/** Zoom seviyesi. Anayasa madde 22.5: yalnız iki seviye. */
export type LayerLevel = 0 | 1;

/**
 * Katman kimliği. Dock item id'leriyle ve modül klasör adlarıyla hizalıdır.
 * Anayasa madde 36.2: MVP'de bir kısmı gerçek veri, bir kısmı hazır arayüz.
 */
export type LayerId =
  | "dashboard"
  | "today"
  | "work"
  | "personal"
  | "health"
  | "finance"
  | "learning"
  | "content"
  | "social"
  | "system";

export interface LayerDefinition {
  id: LayerId;
  /** Dock ve Zoom Trail'de görünen ad. */
  title: string;
  /** Zoom hedefi seviyesi. Seviye 0 = ana evren (dashboard), seviye 1 = modül. */
  level: LayerLevel;
  /**
   * MVP durumu — madde 36.2.
   * "live"        → gerçek veri (Sprint 0'da sahte fixture, Sprint 1+ gerçek)
   * "placeholder" → bilinçli boş durum (madde 26.4), kırık değil "henüz"
   */
  status: "live" | "placeholder";
  /** Lazy yüklenen katman bileşeni. */
  component: () => Promise<{ default: React.ComponentType }>;
}
