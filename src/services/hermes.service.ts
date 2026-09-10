import { ipc } from "./ipc";
import type { HermesHealth } from "@/lib/generated";

/*
 * Hermes Service — Anayasa madde 18, 19.
 *
 * ARKELÉS Hermes'e ASLA doğrudan HTTP atmaz (madde 19.3). Bu servis yalnız
 * Rust çekirdeğindeki `hermes_health` komutunu çağırır; ağ isteği, token
 * okuma ve Origin kontrolü orada olur.
 */

const OFFLINE: HermesHealth = { reachable: false, version: null, capabilities: [] };

export async function getHermesHealth(): Promise<HermesHealth> {
  return ipc<HermesHealth>("hermes_health", OFFLINE);
}

/** Madde 18.2 filtresi — bildirilmeyen yetenek arayüzde HİÇ görünmez. */
export function hasCapability(
  health: HermesHealth | undefined,
  capability: string,
): boolean {
  return health?.reachable === true && health.capabilities.includes(capability);
}
