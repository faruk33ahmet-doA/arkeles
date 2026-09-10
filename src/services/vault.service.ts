import { ipc } from "./ipc";

/*
 * Vault Service — Anayasa madde 7.2, 15.1, 17.3.
 *
 * Madde 15.1: Markdown dosyaları arayüz tarafından DOĞRUDAN TARANMAZ.
 * Bütün okuma Rust çekirdeğindeki index üzerinden yapılır. Bu servisin
 * hiçbir fonksiyonu dosya yolu almaz, dosya okumaz, markdown ayrıştırmaz.
 *
 * Sprint 0: yalnız durum sorgusu (vault bağlanmadı — teslim kriteri).
 * Sprint 1: list_today, list_notes, get_note, search buraya eklenir.
 */

// GEÇİCİ: ts-rs Sprint 1 (madde 14.2)
export interface VaultStatus {
  /** Yapılandırılmış vault yolu. Madde 17.3: sabit kodlanamaz, null olabilir. */
  path: string | null;
  /** Index'lenmiş not sayısı. Madde 9.4: türetilmiş veri. */
  noteCount: number;
  /** Son index zamanı (ISO 8601). Hiç index'lenmediyse null. */
  indexedAt: string | null;
}

const UNCONFIGURED: VaultStatus = {
  path: null,
  noteCount: 0,
  indexedAt: null,
};

export async function getVaultStatus(): Promise<VaultStatus> {
  return ipc<VaultStatus>("vault_status", UNCONFIGURED);
}
