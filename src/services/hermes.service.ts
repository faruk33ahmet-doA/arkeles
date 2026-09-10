import { ipcProbe } from "./ipc";
import type { HermesHealth } from "@/lib/generated";

/*
 * Hermes Service — Anayasa madde 18, 19.
 *
 * ARKELÉS Hermes'e ASLA doğrudan HTTP atmaz (madde 19.3). Bu servis yalnız
 * Rust çekirdeğindeki `hermes_health` komutunu çağırır; ağ isteği, token
 * okuma ve Origin kontrolü orada olur.
 */

const OFFLINE: HermesHealth = { reachable: false, version: null, capabilities: [] };

/*
 * Sprint 1 borcu #2: bu bir AĞ YOKLAMASIDIR, performans metriği değil.
 *
 * Hermes kapalıyken çekirdek 500 ms TCP zaman aşımı bekler — tasarım gereği
 * (madde 18.4). `ipcProbe` bu çağrıyı `probe:` öneki ile ölçer ve bütçe
 * dışında tutar.
 */
export async function getHermesHealth(): Promise<HermesHealth> {
  return ipcProbe<HermesHealth>("hermes_health", OFFLINE);
}

/** Madde 18.2 filtresi — bildirilmeyen yetenek arayüzde HİÇ görünmez. */
export function hasCapability(
  health: HermesHealth | undefined,
  capability: string,
): boolean {
  return health?.reachable === true && health.capabilities.includes(capability);
}
