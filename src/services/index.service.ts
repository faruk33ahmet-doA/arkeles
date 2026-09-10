import { ipc, ipcStrict } from "./ipc";
import type { IndexStatus, ScanReportDto } from "@/lib/generated";

/*
 * Index Service — Anayasa madde 9, 15.
 *
 * SQLite index TÜRETİLMİŞ VERİDİR (madde 9.4). Bu servis index'in durumunu
 * sorgular ve yeniden kurulmasını tetikleyebilir — çünkü madde 9.3 gereği
 * index her an silinip yeniden kurulabilir olmalıdır.
 */

const NOT_READY: IndexStatus = {
  ready: false,
  schemaVersion: 0,
  rebuilding: false,
  lastScanMs: null,
  rowErrors: 0,
};

export async function getIndexStatus(): Promise<IndexStatus> {
  return ipc<IndexStatus>("index_status", NOT_READY);
}

/** Madde 9.3'ün kullanıcıya açılan yüzü: yeniden kurmak kayıpsızdır. */
export async function rebuildIndex(): Promise<ScanReportDto> {
  return ipcStrict<ScanReportDto>("rebuild_index");
}
