import { useQuery } from "@tanstack/react-query";
import { TODAY_FIXTURE, type TodayView } from "@/modules/today/fixtures";
import { queryKeys } from "@/data/queryKeys";

/*
 * Bugün görünümü — Anayasa madde 36.2 ("live" modül).
 *
 * SPRINT 0: sahte veri (fixture). Sprint 0 teslim kriteri "henüz gerçek veri
 * olmayacak" der; fixture bilinçli ve geçicidir.
 *
 * SPRINT 1: queryFn → vault.service.listToday() olacak, index üzerinden
 * (madde 15.2), bütçe < 10 ms (madde 34.1). Bileşenler DEĞİŞMEYECEK —
 * bu yüzden fixture bugünden gerçek şemayla aynı şekli taşıyor.
 */
export function useToday() {
  return useQuery<TodayView>({
    queryKey: queryKeys.today.view,
    queryFn: async () => TODAY_FIXTURE,
    retry: false,
  });
}
