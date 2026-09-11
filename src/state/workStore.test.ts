import { beforeEach, describe, expect, it } from "vitest";
import { useWorkStore, WORK_PANELS } from "./workStore";
import { useNavigationStore } from "./navigationStore";
import { ROOT_LAYER_ID } from "@/navigation/layers/layerRegistry";

/*
 * İş modülü gezinme değişmezleri — Sprint 3 madde 12.
 *
 * KORUNAN KURAL: alt paneller Zoom Trail'e SEVİYE EKLEMEZ.
 * Bu kural sessizce bozulabilir (biri panel geçişini zoomTo'ya bağlayabilir)
 * ve bozulduğunda anayasa madde 22.5 ihlal edilir. Test onu tutuyor.
 */

describe("workStore — panel geçişi Zoom Trail'i bozmaz (madde 12)", () => {
  beforeEach(() => {
    useWorkStore.setState({ workspaceId: null, panel: "overview", noteId: null });
    useNavigationStore.setState({ trail: [ROOT_LAYER_ID] });
  });

  it("çalışma alanı açmak zoom seviyesi EKLEMEZ", () => {
    useNavigationStore.getState().zoomTo("work");
    const before = useNavigationStore.getState().trail;

    useWorkStore.getState().openWorkspace("wif");

    expect(useNavigationStore.getState().trail).toEqual(before);
    expect(useNavigationStore.getState().trail).toHaveLength(2);
  });

  it("panel değiştirmek zoom seviyesi EKLEMEZ", () => {
    useNavigationStore.getState().zoomTo("work");
    useWorkStore.getState().openWorkspace("wif");

    for (const panel of WORK_PANELS) {
      useWorkStore.getState().setPanel(panel.id);
      expect(useNavigationStore.getState().trail).toEqual([ROOT_LAYER_ID, "work"]);
    }
  });

  it("not açmak zoom seviyesi EKLEMEZ", () => {
    useNavigationStore.getState().zoomTo("work");
    useWorkStore.getState().openWorkspace("wif");
    useWorkStore.getState().openNote("n1");

    expect(useNavigationStore.getState().trail).toEqual([ROOT_LAYER_ID, "work"]);
  });
});

describe("workStore — durum geçişleri", () => {
  beforeEach(() => {
    useWorkStore.setState({ workspaceId: null, panel: "overview", noteId: null });
  });

  it("yeni kurum her zaman Genel Bakış'tan başlar", () => {
    const store = useWorkStore.getState();
    store.openWorkspace("wif");
    store.setPanel("tasks");
    expect(useWorkStore.getState().panel).toBe("tasks");

    // Başka kuruma geçiş: önceki panel TAŞINMAZ.
    useWorkStore.getState().openWorkspace("gen");
    expect(useWorkStore.getState().panel).toBe("overview");
    expect(useWorkStore.getState().workspaceId).toBe("gen");
  });

  it("kurum değişince açık not kapanır", () => {
    const store = useWorkStore.getState();
    store.openWorkspace("wif");
    store.openNote("n1");
    expect(useWorkStore.getState().noteId).toBe("n1");

    useWorkStore.getState().openWorkspace("gen");
    expect(useWorkStore.getState().noteId).toBeNull();
  });

  it("panel değişince açık not kapanır", () => {
    const store = useWorkStore.getState();
    store.openWorkspace("wif");
    store.openNote("n1");

    useWorkStore.getState().setPanel("notes");
    expect(useWorkStore.getState().noteId).toBeNull();
  });

  it("aynı kuruma tekrar girmek durumu sıfırlamaz", () => {
    const store = useWorkStore.getState();
    store.openWorkspace("wif");
    store.setPanel("documents");

    useWorkStore.getState().openWorkspace("wif");
    expect(useWorkStore.getState().panel).toBe("documents");
  });

  it("kurumu kapatmak her şeyi sıfırlar", () => {
    const store = useWorkStore.getState();
    store.openWorkspace("wif");
    store.setPanel("tasks");
    store.openNote("n1");

    useWorkStore.getState().closeWorkspace();
    const state = useWorkStore.getState();
    expect(state.workspaceId).toBeNull();
    expect(state.panel).toBe("overview");
    expect(state.noteId).toBeNull();
  });

  it("altı panel tanımlı — Sprint 3 madde 4", () => {
    expect(WORK_PANELS.map((p) => p.id)).toEqual([
      "overview",
      "tasks",
      "projects",
      "documents",
      "meetings",
      "notes",
    ]);
  });
});
