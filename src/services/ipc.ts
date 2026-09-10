import { invoke } from "@tauri-apps/api/core";

/*
 * IPC sınırı — Anayasa madde 12.
 *
 * Webview'deki HİÇBİR bileşen `invoke` çağırmaz. Yalnız bu dosya çağırır,
 * servisler bu dosyayı çağırır, bileşenler servisleri çağırır. Böylece:
 *  - Tauri bağımlılığı tek dosyada kalır (test edilebilirlik)
 *  - Sprint 0'da çekirdek hazır olmadan mock ile ilerlenebilir
 *  - Hermes token'ı JavaScript'e HİÇ inmez (madde 19.3): webview yalnız
 *    komut adı gönderir, ağ işini Rust yapar
 */

/** Tauri içinde mi çalışıyoruz, yoksa tarayıcıda mı (vite dev)? */
export const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/**
 * Rust çekirdeğine tipli komut çağrısı.
 *
 * Sprint 0: Tauri dışında (saf `pnpm dev`) çalışırken çekirdek yoktur.
 * Bu durumda `fallback` döner — arayüz geliştirmesi çekirdeği beklemez.
 * Sprint 1'de gerçek komutlar bağlandığında fallback'ler kaldırılır.
 */
export async function ipc<T>(command: string, fallback: T, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) return fallback;
  return invoke<T>(command, args);
}
