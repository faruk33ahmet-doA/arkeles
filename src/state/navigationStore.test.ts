import { beforeEach, describe, expect, it } from "vitest";
import { useNavigationStore } from "./navigationStore";
import { ROOT_LAYER_ID } from "@/navigation/layers/layerRegistry";

/*
 * Navigasyon değişmezleri — Anayasa madde 22.5, 22.8.
 *
 * Bu testler bir davranışı değil bir ANAYASA KURALINI korur:
 *   22.5  "En fazla 2 zoom seviyesi."
 *   22.8  "Her gezinme klavyeyle geri alınabilir."
 * Kural ihlali sessizce sızarsa navigasyon hissi bozulur; test bunu yakalar.
 */

describe("navigationStore", () => {
  beforeEach(() => {
    useNavigationStore.setState({ trail: [ROOT_LAYER_ID], isTransitioning: false });
  });

  it("kökten başlar", () => {
    expect(useNavigationStore.getState().trail).toEqual([ROOT_LAYER_ID]);
    expect(useNavigationStore.getState().activeLayer()).toBe(ROOT_LAYER_ID);
  });

  it("modüle girince trail iki seviye olur — madde 22.5", () => {
    useNavigationStore.getState().zoomTo("today");
    expect(useNavigationStore.getState().trail).toEqual([ROOT_LAYER_ID, "today"]);
  });

  it("modülden modüle geçişte trail derinleşmez — madde 22.5", () => {
    const { zoomTo } = useNavigationStore.getState();
    zoomTo("today");
    zoomTo("work");
    zoomTo("finance");
    // Üç geçişten sonra bile derinlik 2. Yığın büyümez.
    expect(useNavigationStore.getState().trail).toEqual([ROOT_LAYER_ID, "finance"]);
  });

  it("aynı modüle tekrar girmek durumu değiştirmez", () => {
    useNavigationStore.getState().zoomTo("today");
    const before = useNavigationStore.getState().trail;
    useNavigationStore.getState().zoomTo("today");
    expect(useNavigationStore.getState().trail).toBe(before);
  });

  it("zoomOut bir seviye geri alır — madde 22.8", () => {
    useNavigationStore.getState().zoomTo("health");
    useNavigationStore.getState().zoomOut();
    expect(useNavigationStore.getState().trail).toEqual([ROOT_LAYER_ID]);
  });

  it("kökte zoomOut hiçbir şey yapmaz — kökün altı yok", () => {
    useNavigationStore.getState().zoomOut();
    expect(useNavigationStore.getState().trail).toEqual([ROOT_LAYER_ID]);
  });

  it("zoomToRoot doğrudan köke döner", () => {
    useNavigationStore.getState().zoomTo("social");
    useNavigationStore.getState().zoomToRoot();
    expect(useNavigationStore.getState().trail).toEqual([ROOT_LAYER_ID]);
  });
});

describe("navigationStore — kök katman özel durumu", () => {
  beforeEach(() => {
    useNavigationStore.setState({ trail: [ROOT_LAYER_ID], isTransitioning: false });
  });

  it("köke zoom etmek köke DÖNER, trail'i tekrarlamaz", () => {
    // Komut paletinden "Panel" seçilince oluşan durum.
    useNavigationStore.getState().zoomTo("work");
    useNavigationStore.getState().zoomTo(ROOT_LAYER_ID);
    expect(useNavigationStore.getState().trail).toEqual([ROOT_LAYER_ID]);
  });

  it("kökteyken köke zoom etmek durumu değiştirmez", () => {
    const before = useNavigationStore.getState().trail;
    useNavigationStore.getState().zoomTo(ROOT_LAYER_ID);
    expect(useNavigationStore.getState().trail).toBe(before);
  });

  it("trail hiçbir zaman aynı katmanı iki kez içermez", () => {
    const { zoomTo } = useNavigationStore.getState();
    for (const id of [ROOT_LAYER_ID, "work", ROOT_LAYER_ID, "health", "health"] as const) {
      zoomTo(id);
      const { trail } = useNavigationStore.getState();
      expect(new Set(trail).size).toBe(trail.length);
    }
  });
});
