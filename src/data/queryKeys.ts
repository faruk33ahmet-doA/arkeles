/*
 * Query anahtarları — tek kaynak.
 *
 * Dağınık string literal'ler cache invalidation'ı kırar. Dosya izleyici
 * `vault:changed` yayınladığında bu anahtarlar geçersiz kılınır (madde 21.3).
 */

export const queryKeys = {
  hermes: { health: ["hermes", "health"] as const },
  vault: { status: ["vault", "status"] as const },
  index: { status: ["index", "status"] as const },
  today: { view: ["today", "view"] as const },
  dashboard: { view: ["dashboard", "view"] as const },
  search: (query: string) => ["search", query] as const,
} as const;

/**
 * Vault değişince geçersiz kılınacak anahtar önekleri.
 *
 * `hermes` ve `index.status` BURADA DEĞİL: biri ağ durumu, diğeri şema
 * durumu — vault içeriğinden bağımsız (madde 35.1: gereksiz iş yapmayız).
 */
export const VAULT_DEPENDENT_KEYS = [
  queryKeys.today.view,
  queryKeys.dashboard.view,
  queryKeys.vault.status,
  ["search"],
] as const;
