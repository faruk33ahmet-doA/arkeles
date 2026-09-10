import { describe, expect, it } from "vitest";
import { getActions } from "./actionRegistry";
import { listLayers } from "@/navigation/layers/layerRegistry";

/*
 * Aksiyon kaydı değişmezleri — Anayasa madde 10.2, 18.2, 27.3.
 *
 *   27.3  "Kayıtta olmayan aksiyon yoktur."
 *   18.2  "Tıklandığında hata veren buton yoktur."
 *   10.2  ARKELÉS gelen kutusu dosyasını OLUŞTURMAZ → dosya yoksa
 *         Hızlı Yakalama GÖRÜNMEZ.
 */

const WITH_INBOX = { inboxAvailable: true };
const WITHOUT_INBOX = { inboxAvailable: false };

describe("actionRegistry", () => {
  it("her katman için bir navigasyon aksiyonu üretir — madde 27.4", () => {
    const navigateIds = getActions(WITH_INBOX)
      .filter((a) => a.kind === "navigate")
      .map((a) => a.id);
    for (const layer of listLayers()) {
      expect(navigateIds).toContain(`navigate:${layer.id}`);
    }
  });

  it("yetenek gerektiren semantik aksiyon yok — madde 8.2", () => {
    // Semantik mutasyon Hermes'in alanı; ARKELÉS bunları sunmaz.
    expect(getActions(WITH_INBOX).filter((a) => a.kind === "semantic")).toEqual([]);
  });

  it("her aksiyonun çalıştırılabilir gövdesi var — madde 18.2", () => {
    for (const action of getActions(WITH_INBOX)) {
      expect(action.run).toBeTypeOf("function");
    }
  });

  it("aksiyon kimlikleri tekil", () => {
    const ids = getActions(WITH_INBOX).map((a) => a.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("gelen kutusu VARSA Hızlı Yakalama listelenir — madde 10.1", () => {
    const ids = getActions(WITH_INBOX).map((a) => a.id);
    expect(ids).toContain("capture:quick");
  });

  it("gelen kutusu YOKSA Hızlı Yakalama listelenmez — madde 10.2 + 18.2", () => {
    // ARKELÉS dosyayı oluşturamaz; hata verecek bir buton göstermeyiz.
    const ids = getActions(WITHOUT_INBOX).map((a) => a.id);
    expect(ids).not.toContain("capture:quick");
  });

  it("Hızlı Yakalama MEKANİK sınıftadır — madde 10.3", () => {
    // Semantik olsaydı Hermes yeteneği gerektirirdi ve Hermes kapalıyken
    // çalışmazdı; madde 10 tam bunu engellemek için var.
    const capture = getActions(WITH_INBOX).find((a) => a.id === "capture:quick");
    expect(capture?.kind).toBe("mechanic");
    expect(capture?.capability).toBeUndefined();
  });
});
