/*
 * Bugün modülü — SAHTE VERİ. Sprint 0 kapsamı.
 *
 * Bu dosya Sprint 1'de SİLİNECEKTİR. Var olma sebebi tek: arayüzün gerçek
 * veri şemasıyla bugünden konuşması, böylece çekirdek bağlandığında
 * bileşenlerin değişmemesi.
 *
 * Şema, anayasa madde 16 (ULID kimlik) ve madde 8.3 (görev durumları)
 * ile hizalıdır. Alan adları Rust tarafında `ts-rs` ile üretilecek
 * tiplerle birebir eşleşecek şekilde seçildi (madde 14.2).
 */

/** Anayasa madde 8.3: durum değişimi mekanik mutasyondur (ARKELÉS yazar). */
export type TaskStatus = "open" | "done";

export interface Task {
  /** Anayasa madde 16.1: kalıcı ULID. Dosya adı kimlik DEĞİLDİR (16.2). */
  id: string;
  title: string;
  status: TaskStatus;
  /** Kaynak notun ULID'i — tıklanınca oraya gidilir. */
  noteId: string;
  /** Görevin ait olduğu çalışma alanı (madde 36.3). Yoksa null. */
  workspace: string | null;
  /** ISO 8601 tarih. Vadesi yoksa null. */
  due: string | null;
}

export interface NoteSummary {
  id: string;
  title: string;
  /** ISO 8601 — son değişiklik. */
  modifiedAt: string;
  /**
   * Anayasa madde 16.3: `arkeles_id` taşımayan notlar "yönetilmeyen"dir.
   * Okunur ve gösterilir ama mekanik mutasyon uygulanmaz.
   */
  managed: boolean;
}

export interface TodayView {
  /** Bugün vadesi olan açık görevler. */
  due: Task[];
  /** Vadesi GEÇMİŞ açık görevler — ayrı grup (anayasa Sprint 1 kapsamı). */
  overdue: Task[];
  /** Bugün oluşturulan veya değiştirilen notlar. */
  touchedNotes: NoteSummary[];
}

export const TODAY_FIXTURE: TodayView = {
  overdue: [
    {
      id: "01JQ8XKZ0000000000000001",
      title: "TüGA sponsorluk dosyasını gözden geçir",
      status: "open",
      noteId: "01JQ8XKZ0000000000000101",
      workspace: "TüGA",
      due: "2026-09-09",
    },
  ],
  due: [
    {
      id: "01JQ8XKZ0000000000000002",
      title: "WIF haftalık ekip notunu kapat",
      status: "open",
      noteId: "01JQ8XKZ0000000000000102",
      workspace: "WIF",
      due: "2026-09-11",
    },
    {
      id: "01JQ8XKZ0000000000000003",
      title: "KEPDER bütçe tablosunu güncelle",
      status: "open",
      noteId: "01JQ8XKZ0000000000000103",
      workspace: "KEPDER",
      due: "2026-09-11",
    },
    {
      id: "01JQ8XKZ0000000000000004",
      title: "Merci içerik takvimini onayla",
      status: "done",
      noteId: "01JQ8XKZ0000000000000104",
      workspace: "Merci",
      due: "2026-09-11",
    },
  ],
  touchedNotes: [
    {
      id: "01JQ8XKZ0000000000000102",
      title: "WIF — Haftalık Ekip Notu",
      modifiedAt: "2026-09-11T08:14:00Z",
      managed: true,
    },
    {
      id: "01JQ8XKZ0000000000000105",
      title: "Burkon — Tedarikçi görüşmesi",
      modifiedAt: "2026-09-11T07:02:00Z",
      managed: true,
    },
    {
      id: "01JQ8XKZ0000000000000106",
      title: "Okuma listesi",
      modifiedAt: "2026-09-11T06:40:00Z",
      managed: false,
    },
  ],
};

/*
 * Panel (dashboard) için sahte özet — Sprint 0.
 *
 * DİKKAT: `lifeScore` burada SAHTEDİR ve anayasa madde 11 gereği
 * ARKELÉS bunu ASLA HESAPLAMAZ. Sprint 4'te Hermes hesaplayıp vault'a
 * yazacak, ARKELÉS yalnız okuyacak. Fixture'da `computedAt` alanının
 * bulunması bilinçlidir: madde 11.3 tazelik göstermeyi zorunlu kılar.
 */
export interface LifeScore {
  value: number;
  /** Anayasa madde 11.3: skor tazeliğiyle birlikte saklanır. */
  computedAt: string;
}

export interface WorkspaceSummary {
  name: string;
  openTasks: number;
}

export interface DashboardFixture {
  /** Madde 11.1: Hermes hesaplar. Hesaplanmadıysa null. */
  lifeScore: LifeScore | null;
  /** Madde 11.5: SAYMAK analiz değildir — bu index'ten gelir. */
  workspaces: WorkspaceSummary[];
  criticalTasks: Task[];
}

export const DASHBOARD_FIXTURE: DashboardFixture = {
  lifeScore: { value: 72, computedAt: "2026-09-11T06:00:00Z" },
  workspaces: [
    { name: "WIF", openTasks: 4 },
    { name: "GEN", openTasks: 2 },
    { name: "TüGA", openTasks: 6 },
    { name: "Burkon", openTasks: 1 },
    { name: "Merci", openTasks: 3 },
    { name: "KEPDER", openTasks: 2 },
    { name: "Öğrenciyiz", openTasks: 5 },
  ],
  criticalTasks: [
    ...TODAY_FIXTURE.overdue,
    ...TODAY_FIXTURE.due.filter((t) => t.status === "open").slice(0, 2),
  ],
};
