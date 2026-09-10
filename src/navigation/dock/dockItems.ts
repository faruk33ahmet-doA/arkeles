import type { LayerId } from "@/navigation/layers/types";
import { listLayers } from "@/navigation/layers/layerRegistry";

/*
 * Bottom Dock içeriği — Anayasa madde 22.2, 36.1.
 *
 * Dock BÜTÜN ana modülleri gösterir (dashboard dahil değil — o kök/ev).
 * Sıra sabit ve anlamlıdır: anayasa madde 36.2 tablosundaki sırayı izler.
 * "placeholder" modüller de görünür ama görsel olarak sönük (madde 26.4).
 */

export interface DockItem {
  id: LayerId;
  title: string;
  status: "live" | "placeholder";
}

// Anayasa Bölüm VII sırası. dashboard kök olduğu için dock'ta yer almaz;
// dock'tan "eve dönüş" ayrı bir jesttir (Trail'in kök öğesi + Esc).
const DOCK_ORDER: LayerId[] = [
  "today",
  "work",
  "personal",
  "health",
  "finance",
  "learning",
  "content",
  "social",
  "system",
];

export function getDockItems(): DockItem[] {
  const byId = new Map(listLayers().map((l) => [l.id, l]));
  return DOCK_ORDER.map((id) => {
    const layer = byId.get(id)!;
    return { id, title: layer.title, status: layer.status };
  });
}
