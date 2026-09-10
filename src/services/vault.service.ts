import { ipc, ipcStrict } from "./ipc";
import type { DashboardView, SearchHit, TodayView, VaultStatus } from "@/lib/generated";

/*
 * Vault Service — Anayasa madde 7.2, 15.1, 17.3.
 *
 * Madde 15.1: Markdown dosyaları arayüz tarafından DOĞRUDAN TARANMAZ.
 * Bu servisin hiçbir fonksiyonu dosya yolu almaz, dosya okumaz,
 * markdown ayrıştırmaz — hepsi index üzerinden geçer.
 *
 * Tipler `ts-rs` ile üretilir (madde 14.2). Elle tip tanımı YOK.
 */

const UNCONFIGURED: VaultStatus = { path: null, noteCount: 0, indexedAt: null };
const EMPTY_TODAY: TodayView = { overdue: [], due: [], touchedNotes: [] };
const EMPTY_DASHBOARD: DashboardView = {
  lifeScore: null,
  workspaces: [],
  criticalTasks: [],
};

export async function getVaultStatus(): Promise<VaultStatus> {
  return ipc<VaultStatus>("vault_status", UNCONFIGURED);
}

/**
 * Vault'u seçer. Anayasa madde 17.3.
 *
 * `ipcStrict`: kullanıcının bilinçli eylemi, sessizce başarısız olamaz.
 * Çekirdek yolu doğrular; okunamayan klasör reddedilir.
 */
export async function selectVault(path: string): Promise<VaultStatus> {
  return ipcStrict<VaultStatus>("select_vault", { path });
}

/** Anayasa madde 36.2 — bütçe < 10 ms (madde 34.1). */
export async function getToday(): Promise<TodayView> {
  return ipc<TodayView>("list_today", EMPTY_TODAY);
}

/** Anayasa madde 24.2. */
export async function getDashboard(): Promise<DashboardView> {
  return ipc<DashboardView>("dashboard_view", EMPTY_DASHBOARD);
}

/** Anayasa madde 27.4 — Cmd+K içinden not arama. */
export async function searchNotes(query: string): Promise<SearchHit[]> {
  if (query.trim().length < 2) return [];
  return ipc<SearchHit[]>("search_notes", [], { query });
}
