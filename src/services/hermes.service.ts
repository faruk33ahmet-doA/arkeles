import { ipc, ipcProbe, ipcStrict } from "./ipc";
import type {
  AvailableAction,
  HermesHealth,
  HermesSummary,
  Job,
} from "@/lib/generated";

/*
 * Hermes Service — Anayasa madde 18, 19; Sprint 4.
 *
 * ARKELÉS Hermes'e ASLA doğrudan bağlanmaz (madde 19.3). Bu servis yalnız
 * Rust komutlarını çağırır; adres, token ve WebSocket işi orada.
 *
 * Aksiyon adları çekirdekteki ALLOWLIST'ten doğrulanır — buradan gönderilen
 * keyfi bir dizge Hermes'e ulaşamaz (Sprint 4 madde 19).
 */

const OFFLINE: HermesHealth = { reachable: false, version: null, capabilities: [] };

const EMPTY_SUMMARY: HermesSummary = {
  reachable: false,
  version: null,
  running: 0,
  queued: 0,
  completedToday: 0,
  failedToday: 0,
  lastCompleted: null,
};

/** Ağ yoklaması — bütçe DIŞI (Sprint 1 borcu #2). */
export async function getHermesHealth(): Promise<HermesHealth> {
  return ipcProbe<HermesHealth>("hermes_health", OFFLINE);
}

/** Dashboard özeti. Hermes kapalıysa sayımlar yerel defterden gelir. */
export async function getHermesSummary(): Promise<HermesSummary> {
  return ipcProbe<HermesSummary>("hermes_summary", EMPTY_SUMMARY);
}

/**
 * Kullanılabilir semantik aksiyonlar — Anayasa madde 18.2.
 *
 * Hermes'in İLAN ETTİĞİ yeteneklerle kesişim. Boş dizi = arayüzde hiçbir
 * semantik aksiyon görünmez. Bu SAKİN bir durumdur, hata değil.
 */
export async function getHermesActions(): Promise<AvailableAction[]> {
  return ipcProbe<AvailableAction[]>("hermes_actions", []);
}

/**
 * Semantik iş gönderir — Sprint 4 madde 2, 6.
 *
 * `ipcStrict`: kullanıcının bilinçli eylemi, sessizce başarısız olamaz.
 * Hemen döner; Hermes'e iletim arka planda (madde 6: cevap beklenmez).
 */
export async function submitJob(
  action: string,
  workspace: string | null,
  input: string,
): Promise<Job> {
  return ipcStrict<Job>("submit_job", { action, workspace, input });
}

export async function getActiveJobs(): Promise<Job[]> {
  return ipc<Job[]>("active_jobs", []);
}

/** Aktivite geçmişi — Sprint 4 madde 14. `since`: YYYY-MM-DD. */
export async function getJobHistory(
  since: string,
  workspace: string | null,
  status: string | null,
): Promise<Job[]> {
  return ipc<Job[]>("job_history", [], { since, workspace, status });
}

/** Madde 15: OTOMATİK DEĞİL — yalnız kullanıcı isteyince. */
export async function retryJob(jobId: string): Promise<void> {
  return ipcStrict<void>("retry_job", { jobId });
}

export async function cancelJob(jobId: string): Promise<void> {
  return ipcStrict<void>("cancel_job", { jobId });
}
