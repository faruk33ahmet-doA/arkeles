import { useQuery } from "@tanstack/react-query";
import { searchNotes } from "@/services/vault.service";
import { queryKeys } from "@/data/queryKeys";

/*
 * Not arama — Anayasa madde 27.4.
 *
 * `enabled`: 2 karakterden kısa sorgu çekirdeğe GİTMEZ. Her tuş vuruşunda
 * FTS sorgusu atmak madde 35.1 ihlali olurdu.
 * `staleTime`: aynı sorgu tekrar yazılırsa önbellekten gelir — anlık (madde 21.3).
 */
export function useSearch(query: string) {
  const trimmed = query.trim();
  return useQuery({
    queryKey: queryKeys.search(trimmed),
    queryFn: () => searchNotes(trimmed),
    enabled: trimmed.length >= 2,
    staleTime: 15_000,
    retry: false,
  });
}
