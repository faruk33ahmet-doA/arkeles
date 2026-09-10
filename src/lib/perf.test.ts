import { beforeEach, describe, expect, it } from "vitest";
import { BUDGETS, budgetFor, record, reset, summarize } from "./perf";

/*
 * Performans ölçüm değişmezleri — Sprint 1 borçları #2 ve #3.
 *
 * Bu testler iki KARARI korur:
 *   #2  Ağ yoklamaları (`probe:`) performans bütçesine TABİ DEĞİLDİR.
 *       Hermes kapalıyken 500 ms beklemek tasarımdır (madde 18.4), ihlal değil.
 *   #3  Katman geçişi artık sabit kare aralığına değil, DÜŞEN KARE ORANINA
 *       göre ölçülür; böylece 60 Hz ve 120 Hz'de aynı anlamı taşır.
 */

describe("perf — probe ayrımı (Sprint 1 borcu #2)", () => {
  it("Hermes yoklamasının bütçesi YOKTUR", () => {
    expect(budgetFor("probe:hermes_health")).toBeNull();
  });

  it("gerçek IPC çağrılarının bütçesi vardır", () => {
    expect(budgetFor("ipc:list_today")).toBe(BUDGETS.todayQueryMs);
    expect(budgetFor("ipc:set_task_status")).toBe(BUDGETS.interactionMs);
  });

  it("probe metrikleri özetde işaretlenir", () => {
    reset();
    record("probe:hermes_health", 500);
    record("ipc:list_today", 4);

    const rows = summarize();
    const probe = rows.find((r) => r.name === "probe:hermes_health");
    const query = rows.find((r) => r.name === "ipc:list_today");

    expect(probe?.probe).toBe(true);
    expect(query?.probe).toBe(false);
  });
});

describe("perf — jank metriği (Sprint 1 borcu #3)", () => {
  it("sabit kare aralığı bütçesi KALDIRILDI", () => {
    // Eski `frameMs: 8.3` bütçesi 120 Hz varsayıyordu ve 60 Hz ekranda
    // mükemmel animasyonu bile ihlal gösteriyordu.
    expect("frameMs" in BUDGETS).toBe(false);
  });

  it("jank bütçesi YÜZDE cinsindendir", () => {
    expect(budgetFor("layerJank")).toBe(BUDGETS.jankPercent);
    expect(BUDGETS.jankPercent).toBeGreaterThan(0);
    expect(BUDGETS.jankPercent).toBeLessThanOrEqual(100);
  });

  it("jank metriği yüzde birimiyle raporlanır", () => {
    reset();
    record("layerJank", 12.5);
    const row = summarize().find((r) => r.name === "layerJank");
    expect(row?.unit).toBe("%");
  });

  it("ms metrikleri ms birimiyle raporlanır", () => {
    reset();
    record("coldStart", 143);
    expect(summarize().find((r) => r.name === "coldStart")?.unit).toBe("ms");
  });
});

describe("perf — özet istatistikleri", () => {
  beforeEach(reset);

  it("p50, p95 ve en yüksek doğru hesaplanır", () => {
    for (const value of [1, 2, 3, 4, 5, 6, 7, 8, 9, 100]) {
      record("ipc:list_today", value);
    }
    const row = summarize().find((r) => r.name === "ipc:list_today");
    expect(row?.count).toBe(10);
    expect(row?.max).toBe(100);
    expect(row?.last).toBe(100);
    expect(row?.p50).toBe(6);
    expect(row?.p95).toBe(100);
  });

  it("örnek yoksa satır üretilmez", () => {
    expect(summarize()).toEqual([]);
  });

  it("örnek sayısı sınırlıdır — bellek bütçesi (madde 34.1)", () => {
    for (let i = 0; i < 500; i += 1) record("interaction", i);
    expect(summarize().find((r) => r.name === "interaction")?.count).toBeLessThanOrEqual(120);
  });
});
