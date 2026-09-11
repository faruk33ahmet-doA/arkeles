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
  inbox: { status: ["inbox", "status"] as const },
  today: { view: ["today", "view"] as const },
  dashboard: { view: ["dashboard", "view"] as const },
  search: (query: string) => ["search", query] as const,

  /*
   * İş modülü — Sprint 3. Anahtarlar kuruma göre parçalıdır ki bir kurumun
   * tazelenmesi diğerini yeniden sorgulatmasın (madde 35.1).
   */
  work: {
    workspaces: ["work", "workspaces"] as const,
    overview: (id: string) => ["work", id, "overview"] as const,
    tasks: (id: string) => ["work", id, "tasks"] as const,
    notes: (id: string, kind: string) => ["work", id, "notes", kind] as const,
    meetings: (id: string) => ["work", id, "meetings"] as const,
    documents: (id: string) => ["work", id, "documents"] as const,
    noteDetail: (noteId: string) => ["work", "note", noteId] as const,
  },
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
  // Gelen kutusu dosyası vault içinde: oluşturulunca Hızlı Yakalama açılmalı.
  queryKeys.inbox.status,
  ["search"],
  // Tek önek: bütün iş modülü sorguları vault değişiminde tazelenir.
  ["work"],
] as const;
