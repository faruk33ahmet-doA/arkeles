import { useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useNavigationStore } from "@/state/navigationStore";
import { queryKeys } from "@/data/queryKeys";
import { getToday, getDashboard } from "@/services/vault.service";
import { sampleTransitionFrames } from "@/lib/perf";
import { LAYER_DURATION_S } from "./motion";
import type { LayerId } from "@/navigation/layers/types";

/*
 * Zoom Engine'in genel API'si — Anayasa madde 22.4, 23.6, 34.2.
 *
 * 23.6  "Veri, animasyon BAŞLAMADAN ÖNCE prefetch edilir.
 *        Zoom sırasında spinner görülmez."
 * 34.2  Geçiş kare süreleri ölçülür (bütçe p95 < 8,3 ms).
 *
 * Prefetch senkron BEKLEMEZ: veri hazır değilse geçiş yine anında başlar
 * (madde 23.5: animasyon kapı değildir). Prefetch bir GARANTİ değil,
 * bir ısınmadır — önbellek doluysa geçiş sonunda içerik zaten yerindedir.
 */

/** Katman → o katmanın ihtiyaç duyduğu veriyi ısıtan fonksiyon. */
const PREFETCH: Partial<Record<LayerId, (qc: ReturnType<typeof useQueryClient>) => void>> = {
  dashboard: (qc) => {
    qc.prefetchQuery({ queryKey: queryKeys.dashboard.view, queryFn: getDashboard });
  },
  today: (qc) => {
    qc.prefetchQuery({ queryKey: queryKeys.today.view, queryFn: getToday });
  },
};

export function useZoom() {
  const queryClient = useQueryClient();
  const zoomTo = useNavigationStore((s) => s.zoomTo);
  const zoomOut = useNavigationStore((s) => s.zoomOut);
  const zoomToRoot = useNavigationStore((s) => s.zoomToRoot);

  const enterLayer = useCallback(
    (layer: LayerId) => {
      // Madde 23.6: ÖNCE ısıt, SONRA geçişi başlat.
      PREFETCH[layer]?.(queryClient);
      // Madde 34.2: geçiş boyunca kare süreleri örneklenir.
      sampleTransitionFrames(LAYER_DURATION_S * 1000);
      zoomTo(layer);
    },
    [queryClient, zoomTo],
  );

  const goBack = useCallback(() => {
    PREFETCH.dashboard?.(queryClient);
    sampleTransitionFrames(LAYER_DURATION_S * 1000);
    zoomOut();
  }, [queryClient, zoomOut]);

  const goRoot = useCallback(() => {
    PREFETCH.dashboard?.(queryClient);
    sampleTransitionFrames(LAYER_DURATION_S * 1000);
    zoomToRoot();
  }, [queryClient, zoomToRoot]);

  return { enterLayer, zoomOut: goBack, zoomToRoot: goRoot };
}
