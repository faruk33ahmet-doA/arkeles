import { ipc } from "./ipc";

/*
 * Index Service — Anayasa madde 15, 9.
 *
 * SQLite index TÜRETİLMİŞ VERİDİR (madde 9.4). Bu servis index'in
 * durumunu sorgular ve yeniden kurulmasını tetikleyebilir — çünkü
 * madde 9.3 gereği index her an silinip yeniden kurulabilir olmalıdır.
 *
 * Sprint 0: iskelet. Parser Sprint 1'de yazılacak, o zaman `rebuild`
 * gerçek iş yapacak.
 */

// GEÇİCİ: ts-rs Sprint 1 (madde 14.2)
export interface IndexStatus {
  /** Index dosyası var ve şema sürümü güncel mi? */
  ready: boolean;
  /** Şema sürümü. Madde 15.5: değişince otomatik yeniden kurulur. */
  schemaVersion: number;
  /** Şu an yeniden kurulum sürüyor mu? */
  rebuilding: boolean;
}

const NOT_READY: IndexStatus = {
  ready: false,
  schemaVersion: 0,
  rebuilding: false,
};

export async function getIndexStatus(): Promise<IndexStatus> {
  return ipc<IndexStatus>("index_status", NOT_READY);
}
