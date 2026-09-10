/*
 * Query anahtarları — tek kaynak.
 *
 * Dağınık string literal'ler cache invalidation'ı kırar. Sprint 2'de
 * mekanik mutasyonlar (madde 8.1) bu anahtarları invalidate edecek;
 * o yüzden baştan merkezî.
 */

export const queryKeys = {
  hermes: {
    health: ["hermes", "health"] as const,
  },
  vault: {
    status: ["vault", "status"] as const,
  },
  index: {
    status: ["index", "status"] as const,
  },
  today: {
    view: ["today", "view"] as const,
  },
} as const;
