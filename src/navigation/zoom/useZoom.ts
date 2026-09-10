import { useCallback } from "react";
import { useNavigationStore } from "@/state/navigationStore";
import type { LayerId } from "@/navigation/layers/types";

/*
 * Zoom Engine'in genel API'si — Anayasa madde 22.4.
 *
 * Bileşenler navigasyonu doğrudan store'a dokunarak değil, bu hook üzerinden
 * yapar. Böylece prefetch (madde 23.6) ve trail yönetimi tek noktadan geçer.
 *
 * Sprint 0: prefetch kancası hazır ama gerçek veri yok — no-op.
 * Sprint 1: zoomTo çağrısı, animasyon BAŞLAMADAN hedef katmanın query'sini
 *           ısıtacak (queryClient.prefetchQuery).
 */

export function useZoom() {
  const zoomTo = useNavigationStore((s) => s.zoomTo);
  const zoomOut = useNavigationStore((s) => s.zoomOut);
  const zoomToRoot = useNavigationStore((s) => s.zoomToRoot);

  const enterLayer = useCallback(
    (layer: LayerId) => {
      // TODO(Sprint 1): prefetch(layer) — animasyondan önce veriyi ısıt (madde 23.6)
      zoomTo(layer);
    },
    [zoomTo],
  );

  return { enterLayer, zoomOut, zoomToRoot };
}
