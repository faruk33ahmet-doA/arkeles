import { useQuery } from "@tanstack/react-query";
import { getDashboard } from "@/services/vault.service";
import { queryKeys } from "@/data/queryKeys";

/** Panel görünümü — Anayasa madde 24.2. Gerçek veri (Sprint 1). */
export function useDashboard() {
  return useQuery({
    queryKey: queryKeys.dashboard.view,
    queryFn: getDashboard,
    retry: false,
  });
}
