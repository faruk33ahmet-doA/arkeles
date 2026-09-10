import { useQuery } from "@tanstack/react-query";
import { getHermesHealth } from "@/services/hermes.service";
import { queryKeys } from "@/data/queryKeys";

/*
 * Anayasa madde 18.3: Hermes durumu dock'ta gösterilir.
 * Madde 18.4: kapalı olması hata durumu değil — bu yüzden `retry` yok,
 * hata fırlatılmaz, `reachable: false` normal bir sonuçtur.
 *
 * Yoklama aralığı 60 sn: sessiz (madde 5), gereksiz iş üretmez (madde 35.1).
 */
export function useHermesHealth() {
  return useQuery({
    queryKey: queryKeys.hermes.health,
    queryFn: getHermesHealth,
    refetchInterval: 60_000,
    retry: false,
  });
}
