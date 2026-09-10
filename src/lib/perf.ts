/*
 * Performans ölçüm sistemi — Anayasa madde 34.2.
 *
 * "Bütçe her sprint sonunda ÖLÇÜLÜR ve RAPORLANIR. Bütçeyi aşan sprint kapanmaz."
 *
 * Bu modül ölçer, YARGILAMAZ: bir metriği "kötü" ilan etmez, sadece sayıyı
 * ve bütçeyi yan yana koyar. Karar insanın.
 *
 * Maliyet: `performance.now()` ve bir dizi push. Ölçüm, ölçtüğü şeyi
 * yavaşlatmamalı (madde 35.1) — bu yüzden örnek sayısı sınırlı ve
 * hiçbir ölçüm render tetiklemez.
 */

/** Anayasa madde 34.1 bütçeleri. Tek kaynak. */
export const BUDGETS = {
  coldStartMs: 800,
  /** 120 Hz'de bir kare. */
  frameMs: 8.3,
  interactionMs: 100,
  todayQueryMs: 10,
  fullScanMs: 1000,
} as const;

export type MetricName =
  | "coldStart"
  | "layerTransition"
  | "interaction"
  | "ipc:list_today"
  | "ipc:dashboard_view"
  | "ipc:vault_status"
  | "ipc:index_status"
  | "ipc:hermes_health"
  | "ipc:search_notes"
  | "ipc:select_vault"
  | "ipc:rebuild_index";

export interface Sample {
  name: MetricName;
  ms: number;
  at: number;
}

export interface Summary {
  name: MetricName;
  count: number;
  last: number;
  p50: number;
  p95: number;
  max: number;
}

/** Metrik başına tutulan en fazla örnek. Bellek sınırı (madde 34.1). */
const MAX_SAMPLES = 120;

const samples = new Map<MetricName, number[]>();
let coldStartOrigin: number | null = null;
let coldStartMs: number | null = null;

export function record(name: MetricName, ms: number): void {
  let list = samples.get(name);
  if (!list) {
    list = [];
    samples.set(name, list);
  }
  list.push(ms);
  if (list.length > MAX_SAMPLES) list.shift();
}

/** Bir işi ölçer ve sonucunu olduğu gibi döndürür. */
export async function measure<T>(name: MetricName, work: () => Promise<T>): Promise<T> {
  const started = performance.now();
  try {
    return await work();
  } finally {
    record(name, performance.now() - started);
  }
}

/*
 * Soğuk açılış — Anayasa madde 34.1: < 800 ms.
 *
 * Başlangıç noktası olarak `performance.timeOrigin` kullanılır: bu, sayfanın
 * yüklenmeye başladığı andır, JS'in çalıştığı an değil. Modül yükleme
 * süresini de kapsar, yani gerçek beklemeyi ölçer.
 */
export function markColdStartOrigin(): void {
  coldStartOrigin = 0; // performance.now() zaten timeOrigin'e göredir
}

/**
 * İlk anlamlı kare — panelin gerçek veriyle boyandığı an.
 * İlk çağrı kazanır; sonraki çağrılar yok sayılır.
 */
export function markFirstMeaningfulPaint(): void {
  if (coldStartMs !== null || coldStartOrigin === null) return;
  coldStartMs = performance.now();
  record("coldStart", coldStartMs);
}

export function getColdStartMs(): number | null {
  return coldStartMs;
}

/*
 * Katman geçişi kare süresi — Anayasa madde 34.1: p95 < 8,3 ms.
 *
 * Geçiş boyunca rAF ile kare aralıkları örneklenir. Örnekleme geçiş
 * bitince DURUR — sürekli çalışan bir rAF döngüsü boşta pil yakar
 * ve madde 35.1'i ihlal eder.
 */
export function sampleTransitionFrames(durationMs: number): void {
  if (typeof requestAnimationFrame !== "function") return;

  let previous = performance.now();
  const deadline = previous + durationMs;

  function tick(now: number) {
    record("layerTransition", now - previous);
    previous = now;
    if (now < deadline) requestAnimationFrame(tick);
  }

  requestAnimationFrame(tick);
}

/** Etkileşim → görsel tepki. Çağıran, tepkinin göründüğü karede kapatır. */
export function startInteraction(): () => void {
  const started = performance.now();
  let closed = false;
  return () => {
    if (closed) return;
    closed = true;
    record("interaction", performance.now() - started);
  };
}

function percentile(sorted: number[], p: number): number {
  if (sorted.length === 0) return 0;
  const index = Math.min(sorted.length - 1, Math.floor((p / 100) * sorted.length));
  return sorted[index]!;
}

export function summarize(): Summary[] {
  const out: Summary[] = [];
  for (const [name, list] of samples) {
    if (list.length === 0) continue;
    const sorted = [...list].sort((a, b) => a - b);
    out.push({
      name,
      count: list.length,
      last: list[list.length - 1]!,
      p50: percentile(sorted, 50),
      p95: percentile(sorted, 95),
      max: sorted[sorted.length - 1]!,
    });
  }
  return out.sort((a, b) => a.name.localeCompare(b.name));
}

/** Bir metriğin bütçesi varsa döner. Yoksa null — yargı yok, bilgi yok. */
export function budgetFor(name: MetricName): number | null {
  if (name === "coldStart") return BUDGETS.coldStartMs;
  if (name === "layerTransition") return BUDGETS.frameMs;
  if (name === "interaction") return BUDGETS.interactionMs;
  if (name === "ipc:list_today") return BUDGETS.todayQueryMs;
  if (name === "ipc:rebuild_index") return BUDGETS.fullScanMs;
  // Diğer IPC çağrıları için anayasada ayrı bütçe yok; etkileşim bütçesine tabidirler.
  if (name.startsWith("ipc:")) return BUDGETS.interactionMs;
  return null;
}

export function reset(): void {
  samples.clear();
}
