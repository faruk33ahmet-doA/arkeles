import { useQuery } from "@tanstack/react-query";
import { getToday } from "@/services/vault.service";
import { queryKeys } from "@/data/queryKeys";

/*
 * Bugün görünümü — GERÇEK VERİ (Sprint 1).
 *
 * Fixture kaldırıldı. Veri Rust çekirdeğindeki SQLite index'ten gelir
 * (madde 15.2), bütçe < 10 ms (madde 34.1).
 *
 * Tazeleme dosya izleyiciden gelir (madde 20.4 → `vault:changed`), yoklamayla
 * değil — madde 35.1: gereksiz iş yapmayız.
 */
export function useToday() {
  return useQuery({
    queryKey: queryKeys.today.view,
    queryFn: getToday,
    retry: false,
  });
}
