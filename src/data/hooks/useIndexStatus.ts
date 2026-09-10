import { useQuery } from "@tanstack/react-query";
import { getIndexStatus } from "@/services/index.service";
import { queryKeys } from "@/data/queryKeys";

/*
 * Index durumu — Anayasa madde 9.3.
 *
 * Tarama sürerken (`rebuilding`) daha sık yoklanır ki kullanıcı bitişi
 * görsün; boştayken yoklama YOK — madde 35.1.
 */
export function useIndexStatus() {
  return useQuery({
    queryKey: queryKeys.index.status,
    queryFn: getIndexStatus,
    retry: false,
    refetchInterval: (query) => (query.state.data?.rebuilding ? 400 : false),
  });
}
