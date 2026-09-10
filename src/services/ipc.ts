import { invoke } from "@tauri-apps/api/core";
import { measure, type MetricName } from "@/lib/perf";

/*
 * IPC sınırı — Anayasa madde 12, 19.3, 34.2.
 *
 * Webview'deki HİÇBİR bileşen `invoke` çağırmaz. Yalnız bu dosya çağırır,
 * servisler bu dosyayı çağırır, bileşenler servisleri çağırır. Böylece:
 *  - Tauri bağımlılığı tek dosyada kalır
 *  - Hermes token'ı JavaScript'e HİÇ inmez (madde 19.3)
 *  - HER çağrı otomatik ölçülür (madde 34.2) — ölçümü unutmak imkânsız
 */

/** Tauri içinde mi çalışıyoruz, yoksa tarayıcıda mı (saf vite dev)? */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Rust çekirdeğine tipli komut çağrısı. Süre `perf` sistemine yazılır.
 *
 * Tauri dışında (saf `pnpm dev`) çekirdek yoktur; `fallback` döner ve
 * arayüz geliştirmesi çekirdeği beklemez.
 */
export async function ipc<T>(
  command: string,
  fallback: T,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauri()) return fallback;
  return measure(`ipc:${command}` as MetricName, () => invoke<T>(command, args));
}

/**
 * Hata yutmayan çağrı — kullanıcının BİLİNÇLİ eylemleri için (vault seçimi,
 * index yeniden kurulumu). Madde 25.4: yalnız gerçek arıza akışı keser;
 * ama kullanıcının başlattığı bir iş sessizce başarısız olamaz.
 */
export async function ipcStrict<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauri()) {
    throw new Error("Bu işlem yalnız masaüstü uygulamasında çalışır.");
  }
  return measure(`ipc:${command}` as MetricName, () => invoke<T>(command, args));
}
