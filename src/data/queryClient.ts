import { QueryClient } from "@tanstack/react-query";

/*
 * TanStack Query yapılandırması — Anayasa madde 14, 21.3, 34.
 *
 * Madde 21.3: "Açılışta ilk görünen şey SON BİLİNEN VERİDİR; arkada tazelenir.
 *              Kullanıcı boş bir iskelet ekranı beklemez."
 * Bunun aracı stale-while-revalidate davranışıdır — bu yüzden:
 *  - staleTime > 0: veri hemen gösterilir, arkada tazelenir
 *  - retry düşük: madde 18.4, Hermes kapalı olması hata değildir;
 *    ısrarla yeniden denemek gereksiz iş üretir (madde 35.1)
 *  - refetchOnWindowFocus kapalı: sessizlik (madde 5) — pencereye her
 *    dönüşte ağ trafiği üretmeyiz. Tazeleme dosya izleyiciden gelir (Sprint 1).
 */

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      gcTime: 5 * 60_000,
      retry: 1,
      retryDelay: 500,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
    },
  },
});
