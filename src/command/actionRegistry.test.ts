import { describe, expect, it } from "vitest";
import { getActions } from "./actionRegistry";
import { listLayers } from "@/navigation/layers/layerRegistry";

/*
 * Aksiyon kaydı değişmezleri — Anayasa madde 18.2, 27.3.
 *
 *   27.3  "Kayıtta olmayan aksiyon yoktur."
 *   18.2  "Hermes'in bildirmediği yetenek arayüzde HİÇ görünmez."
 *         → Sprint 0'da yetenek gerektiren aksiyon TANIMSIZ olmalı;
 *           sahte komut koymak bu maddeyi ihlal ederdi.
 */

describe("actionRegistry", () => {
  it("her katman için bir navigasyon aksiyonu üretir — madde 27.4", () => {
    const actions = getActions();
    const navigateIds = actions.filter((a) => a.kind === "navigate").map((a) => a.id);
    for (const layer of listLayers()) {
      expect(navigateIds).toContain(`navigate:${layer.id}`);
    }
  });

  it("Sprint 0'da yetenek gerektiren aksiyon yok — madde 18.2", () => {
    const semantic = getActions().filter((a) => a.kind === "semantic");
    expect(semantic).toEqual([]);
  });

  it("her aksiyonun çalıştırılabilir bir gövdesi var — madde 18.2", () => {
    // Tıklandığında hata veren buton yoktur.
    for (const action of getActions()) {
      expect(action.run).toBeTypeOf("function");
    }
  });

  it("aksiyon kimlikleri tekil", () => {
    const ids = getActions().map((a) => a.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});
