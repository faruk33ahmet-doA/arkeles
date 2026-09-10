import { useQuery } from "@tanstack/react-query";
import { getInboxStatus } from "@/services/mutation.service";
import { queryKeys } from "@/data/queryKeys";

/*
 * Gelen kutusu durumu — Anayasa madde 10.2, 18.2.
 *
 * 10.2  ARKELÉS gelen kutusu dosyasını OLUŞTURMAZ.
 * 18.2  "Hermes'in bildirmediği yeteneği arayüzde HİÇ göstermez" ilkesinin
 *       aynısı burada da geçerli: dosya yoksa Hızlı Yakalama GÖRÜNMEZ.
 *       Tıklandığında hata veren buton yoktur.
 */
export function useInboxStatus() {
  return useQuery({
    queryKey: queryKeys.inbox.status,
    queryFn: getInboxStatus,
    retry: false,
  });
}
