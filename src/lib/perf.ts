/*
 * Performans ölçüm sistemi — Anayasa madde 34.2.
 *
 * "Bütçe her sprint sonunda ÖLÇÜLÜR ve RAPORLANIR."
 *
 * Bu modül ölçer, YARGILAMAZ: bir metriği "kötü" ilan etmez, sadece sayıyı
 * ve bütçeyi yan yana koyar. Karar insanın.
 *
 * Maliyet: `performance.now()` ve bir dizi push. Ölçüm, ölçtüğü şeyi
 * yavaşlatmamalı (madde 35.1) — örnek sayısı sınırlı, hiçbir ölçüm render
 * tetiklemez.
 */

/** Anayasa madde 34.1 bütçeleri. Tek kaynak. */
export const BUDGETS = {
  coldStartMs: 800,
  interactionMs: 100,
  todayQueryMs: 10,
  fullScanMs: 1000,
  /*
   * Sprint 1 borcu #3: SABİT 8,3 ms hedefi kaldırıldı.
   *
   * Eski metrik kare ARALIĞINI 8,3 ms'e (120 Hz) karşı ölçüyordu. 60 Hz
   * ekranda mükemmel akıcı bir animasyon bile 16,7 ms okur ve "ihlal"
   * görünürdü — metrik ekrana bağımlıydı, jank'e değil.
   *
   * Yeni metrik: DÜŞEN KARE ORANI. Ekranın kendi tazeleme aralığı ölçülür,
   * o aralığın 1,5 katından uzun süren kareler "düşmüş" sayılır. Böylece
   * 60 Hz ve 120 Hz'de aynı anlamı taşır.
   */
  jankPercent: 10,
} as const;

export type MetricName =
  | "coldStart"
  | "layerJank"
  | "interaction"
  | "ipc:list_today"
  | "ipc:dashboard_view"
  | "ipc:vault_status"
  | "ipc:index_status"
  | "ipc:search_notes"
  | "ipc:select_vault"
  | "ipc:rebuild_index"
  | "ipc:inbox_status"
  | "ipc:set_task_status"
  | "ipc:set_frontmatter_field"
  | "ipc:toggle_tag"
  | "ipc:quick_capture"
  // Ağ yoklaması — bütçe DIŞI (aşağıya bak).
  | "probe:hermes_health";

export interface Summary {
  name: MetricName;
  count: number;
  last: number;
  p50: number;
  p95: number;
  max: number;
  /** Metriğin birimi — tablo bunu gösterir. */
  unit: "ms" | "%";
  /** Bütçe dışı ölçüm (ağ yoklaması gibi). Tabloda "—" gösterilir. */
  probe: boolean;
}

/** Metrik başına tutulan en fazla örnek. Bellek sınırı (madde 34.1). */
const MAX_SAMPLES = 120;

const samples = new Map<MetricName, number[]>();
let coldStartMs: number | null = null;
let coldStartArmed = false;

/*
 * Sprint 1 borcu #2: Hermes yoklaması bir PERFORMANS metriği değildir.
 *
 * Hermes kapalıyken çekirdek 500 ms TCP zaman aşımı bekler — bu TASARIM
 * GEREĞİDİR (madde 18.4: kapalı olmak hata değil). Bunu 100 ms etkileşim
 * bütçesine karşı raporlamak yanlış alarm üretiyordu.
 *
 * Bu yüzden `probe:` önekli metrikler ölçülür ama BÜTÇEYE TABİ DEĞİLDİR.
 */
const PROBE_PREFIX = "probe:";

function isProbe(name: MetricName): boolean {
  return name.startsWith(PROBE_PREFIX);
}

export function record(name: MetricName, value: number): void {
  let list = samples.get(name);
  if (!list) {
    list = [];
    samples.set(name, list);
  }
  list.push(value);
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
 * `performance.now()` zaten `timeOrigin`'e (sayfanın yüklenmeye başladığı
 * an) göredir; modül yükleme süresini de kapsar, yani gerçek beklemeyi ölçer.
 */
export function markColdStartOrigin(): void {
  coldStartArmed = true;
}

/** İlk anlamlı kare. İlk çağrı kazanır. */
export function markFirstMeaningfulPaint(): void {
  if (coldStartMs !== null || !coldStartArmed) return;
  coldStartMs = performance.now();
  record("coldStart", coldStartMs);
}

export function getColdStartMs(): number | null {
  return coldStartMs;
}

// ---------------------------------------------------------------------------
// JANK ÖLÇÜMÜ — Sprint 1 borcu #3
// ---------------------------------------------------------------------------

/**
 * Ekranın ölçülmüş kare aralığı (ms). 60 Hz → ~16,7 · 120 Hz → ~8,3.
 * Kalibrasyon yapılana kadar `null`.
 */
let refreshIntervalMs: number | null = null;

/** Bir karenin "düşmüş" sayılması için aralığın kaç katı olması gerekir. */
const DROP_FACTOR = 1.5;

/**
 * Ekranın tazeleme aralığını ölçer. Boşta, bir kez çalışır.
 *
 * Medyan alınır (ortalama değil): tek bir uzun kare ortalamayı bozar,
 * medyan tipik aralığı verir.
 */
export function calibrateRefreshRate(): void {
  if (typeof requestAnimationFrame !== "function" || refreshIntervalMs !== null) return;

  const intervals: number[] = [];
  let previous = performance.now();

  function tick(now: number) {
    intervals.push(now - previous);
    previous = now;
    if (intervals.length < 20) {
      requestAnimationFrame(tick);
      return;
    }
    // İlk kare ölçüm gürültüsü taşır, atılır.
    const sorted = intervals.slice(1).sort((a, b) => a - b);
    refreshIntervalMs = sorted[Math.floor(sorted.length / 2)] ?? null;
  }

  requestAnimationFrame(tick);
}

export function getRefreshIntervalMs(): number | null {
  return refreshIntervalMs;
}

/**
 * Bir katman geçişi boyunca DÜŞEN KARE ORANINI ölçer (madde 34.1).
 *
 * Ekran aralığı henüz kalibre edilmediyse ölçüm YAPILMAZ — yanlış bir
 * sayı üretmek, sayı üretmemekten kötüdür.
 *
 * Örnekleme geçiş bitince DURUR: sürekli çalışan bir rAF döngüsü boşta
 * pil yakar ve madde 35.1'i ihlal eder.
 */
export function sampleTransitionJank(durationMs: number): void {
  if (typeof requestAnimationFrame !== "function") return;

  const baseline = refreshIntervalMs;
  if (baseline === null) {
    // İlk geçişte kalibrasyon henüz bitmemiş olabilir; sessizce geç.
    return;
  }

  const threshold = baseline * DROP_FACTOR;
  let previous = performance.now();
  const deadline = previous + durationMs;
  let total = 0;
  let dropped = 0;

  function tick(now: number) {
    const delta = now - previous;
    previous = now;
    total += 1;
    if (delta > threshold) {
      // Kaç kare atlandığını da hesaba kat: 3 kare süren bir aralık
      // 2 düşen kare demektir.
      dropped += Math.max(1, Math.round(delta / baseline!) - 1);
    }

    if (now < deadline) {
      requestAnimationFrame(tick);
      return;
    }
    if (total > 0) {
      record("layerJank", (dropped / (total + dropped)) * 100);
    }
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
      unit: name === "layerJank" ? "%" : "ms",
      probe: isProbe(name),
    });
  }
  return out.sort((a, b) => a.name.localeCompare(b.name));
}

/**
 * Bir metriğin bütçesi. `null` = bütçeye tabi değil (yargı yok, bilgi yok).
 *
 * `probe:` metrikleri her zaman `null` döner — Sprint 1 borcu #2.
 */
export function budgetFor(name: MetricName): number | null {
  if (isProbe(name)) return null;
  if (name === "coldStart") return BUDGETS.coldStartMs;
  if (name === "layerJank") return BUDGETS.jankPercent;
  if (name === "interaction") return BUDGETS.interactionMs;
  if (name === "ipc:list_today") return BUDGETS.todayQueryMs;
  if (name === "ipc:rebuild_index") return BUDGETS.fullScanMs;
  // Diğer IPC çağrıları etkileşim bütçesine tabidir.
  if (name.startsWith("ipc:")) return BUDGETS.interactionMs;
  return null;
}

export function reset(): void {
  samples.clear();
}
