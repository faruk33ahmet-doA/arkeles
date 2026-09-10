import { describe, expect, it } from "vitest";
import { LAYER_DURATION_S, layerVariants, type ZoomDirection } from "./motion";

/*
 * Hareket Anayasası değişmezleri — Anayasa madde 23.
 *
 * NEDEN BU TESTLER VAR: Zoom yönü bir kez ters yazıldı ve derleyici,
 * linter, tip sistemi hiçbiri fark etmedi — çünkü ters yön de geçerli
 * bir animasyondur, sadece YANLIŞ hissettirir. Mekânsal metafor (madde 23.1)
 * uygulamanın navigasyon hissinin tamamı olduğu için testle sabitlenir.
 */

type Target = { opacity: number; scale: number };

function resolve(
  variant: (typeof layerVariants)[string],
  direction: ZoomDirection,
): Target {
  // framer-motion varyant fonksiyonu imzası: (custom, current, velocity)
  const value = typeof variant === "function" ? variant(direction, {}, {}) : variant;
  return value as Target;
}

const enter = (d: ZoomDirection) => resolve(layerVariants.enter!, d);
const exit = (d: ZoomDirection) => resolve(layerVariants.exit!, d);
const center = resolve(layerVariants.center!, "in");

describe("Hareket Anayasası — süre sınırı (madde 23.4)", () => {
  it("katman geçişi 300 ms üst sınırını aşmaz", () => {
    expect(LAYER_DURATION_S).toBeLessThanOrEqual(0.3);
  });
});

describe("Hareket Anayasası — yalnız transform ve opacity (madde 23.3)", () => {
  it("hiçbir varyant layout veya paint tetikleyen özellik taşımaz", () => {
    const yasak = ["width", "height", "top", "left", "right", "bottom", "filter", "blur", "backdropFilter"];
    for (const target of [enter("in"), enter("out"), exit("in"), exit("out"), center]) {
      for (const key of Object.keys(target)) {
        expect(yasak).not.toContain(key);
        expect(["opacity", "scale"]).toContain(key);
      }
    }
  });
});

describe("Hareket Anayasası — mekânsal model (madde 23.1)", () => {
  it("dinlenme durumu tam görünür ve ölçeksizdir", () => {
    expect(center).toEqual({ opacity: 1, scale: 1 });
  });

  it("bütün giriş ve çıkış durumları saydamdır", () => {
    for (const target of [enter("in"), enter("out"), exit("in"), exit("out")]) {
      expect(target.opacity).toBe(0);
    }
  });

  /*
   * KAMERA İLERİ (panel → modül):
   *   panel BÜYÜYEREK geçilir, modül İLERİDEN yaklaşır.
   * Yani içeri girişte çıkan yüzey gelen yüzeyden DAHA BÜYÜK olmalı.
   */
  it("içeri girişte: çıkan yüzey büyür, gelen yüzey küçükten gelir", () => {
    expect(exit("in").scale).toBeGreaterThan(1);
    expect(enter("in").scale).toBeLessThan(1);
  });

  /*
   * KAMERA GERİ (modül → panel):
   *   modül UZAKLAŞARAK küçülür, panel BÜYÜKTEN yerine oturur.
   */
  it("dışarı çıkışta: çıkan yüzey küçülür, gelen yüzey büyükten gelir", () => {
    expect(exit("out").scale).toBeLessThan(1);
    expect(enter("out").scale).toBeGreaterThan(1);
  });

  it("yön tersine çevrilince ölçekler simetrik yer değiştirir", () => {
    // Aynı eksende ileri-geri: içeri girişin çıkışı, dışarı çıkışın girişiyle
    // aynı ölçekte olmalı. Simetri bozulursa metafor tutarsızlaşır.
    expect(exit("in").scale).toBe(enter("out").scale);
    expect(enter("in").scale).toBe(exit("out").scale);
  });

  it("ölçek farkı gösterişsizdir — en fazla %5 (madde 5)", () => {
    for (const target of [enter("in"), enter("out"), exit("in"), exit("out")]) {
      expect(Math.abs(target.scale - 1)).toBeLessThanOrEqual(0.05);
    }
  });
});
