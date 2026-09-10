import { ipc } from "./ipc";

/*
 * Hermes Service — Anayasa madde 18, 19.
 *
 * ARKELÉS Hermes'e ASLA doğrudan HTTP atmaz (madde 19.3). Bu servis yalnız
 * Rust çekirdeğindeki `hermes_health` komutunu çağırır; ağ isteği, token
 * okuma ve Origin kontrolü orada olur.
 *
 * Madde 18.2: Hermes'in bildirmediği yetenek arayüzde HİÇ görünmez.
 * `capabilities` listesi bu filtrelemenin kaynağıdır.
 */

// GEÇİCİ: ts-rs Sprint 1 (madde 14.2)
export interface HermesHealth {
  /** Hermes'e ulaşılabildi mi? Anayasa madde 18.4: false bir HATA DEĞİLDİR. */
  reachable: boolean;
  version: string | null;
  /** Madde 18.1 — yetenek anahtarları. Ulaşılamıyorsa boş dizi. */
  capabilities: string[];
}

const OFFLINE: HermesHealth = {
  reachable: false,
  version: null,
  capabilities: [],
};

/**
 * Sprint 0: Hermes bağlanmadı (teslim kriteri). Çekirdek mock döner.
 * Sprint 1: gerçek `GET /health` çağrısı Rust tarafında.
 */
export async function getHermesHealth(): Promise<HermesHealth> {
  return ipc<HermesHealth>("hermes_health", OFFLINE);
}

/** Madde 18.2 filtresi — çağıran tarafın yetenek sorgusu. */
export function hasCapability(health: HermesHealth | undefined, capability: string): boolean {
  return health?.reachable === true && health.capabilities.includes(capability);
}
