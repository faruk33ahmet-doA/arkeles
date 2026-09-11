import { describe, expect, it } from "vitest";
import { getActions, type ActionContext } from "./actionRegistry";
import type { AvailableAction, Workspace } from "@/lib/generated";

/*
 * Cmd+K Hermes aksiyonları — Sprint 4 madde 18, anayasa madde 18.2.
 *
 * KORUNAN KURAL: "Hermes'in bildirmediği yeteneği arayüzde HİÇ göstermez."
 * Bu sessizce bozulabilir (biri aksiyonu elle listeye ekleyebilir) ve
 * bozulduğunda kullanıcıya çalışmayan bir buton gösterilir.
 */

const ws = (id: string, label: string, active: boolean): Workspace => ({
  id,
  label,
  active,
  outstandingTasks: 0,
  noteCount: 0,
});

const action = (id: string, label: string, capability: string): AvailableAction => ({
  id,
  label,
  capability,
});

const WORKSPACES = [ws("wif", "WIF", true), ws("gen", "GEN", true), ws("tuga", "TüGA", false)];
const BASE = { inboxAvailable: false, workspaces: WORKSPACES };

describe("Cmd+K — Hermes aksiyonları (madde 18.2)", () => {
  it("yetenek YOKSA hiçbir Hermes aksiyonu listelenmez", () => {
    const ids = getActions({ ...BASE, hermesActions: [] }).map((a) => a.id);
    expect(ids.filter((id) => id.startsWith("hermes:"))).toEqual([]);
  });

  it("hermesActions hiç verilmezse de listelenmez", () => {
    const ids = getActions(BASE).map((a) => a.id);
    expect(ids.filter((id) => id.startsWith("hermes:"))).toEqual([]);
  });

  it("bildirilen yetenek için genel aksiyon üretilir", () => {
    const ids = getActions({
      ...BASE,
      hermesActions: [action("report.create", "Rapor oluştur", "report.create")],
    }).map((a) => a.id);

    expect(ids).toContain("hermes:report.create");
  });

  it("yalnız AKTİF kurumlar için kısayol üretilir", () => {
    const ids = getActions({
      ...BASE,
      hermesActions: [action("report.create", "Rapor oluştur", "report.create")],
    }).map((a) => a.id);

    expect(ids).toContain("hermes:report.create:wif");
    expect(ids).toContain("hermes:report.create:gen");
    // Pasif kurumda yüzey yok → aksiyon da yok (madde 18.2).
    expect(ids).not.toContain("hermes:report.create:tuga");
  });

  it("Hermes aksiyonları semantik sınıftadır ve capability taşır", () => {
    // Semantik = Hermes'in alanı (madde 8.2). Capability olmadan gösterilemez.
    const actions = getActions({
      ...BASE,
      hermesActions: [action("task.execute", "Görev ver", "task.execute")],
    }).filter((a) => a.id.startsWith("hermes:"));

    expect(actions.length).toBeGreaterThan(0);
    for (const a of actions) {
      expect(a.kind).toBe("semantic");
      expect(a.capability).toBe("task.execute");
    }
  });

  it("aksiyon doğru kurum ve aksiyonla açılır", () => {
    const calls: { actionId: string; workspaceId: string | null }[] = [];
    const ctx: ActionContext = {
    moveSelectedTask: () => {},
      enterLayer: () => {},
      closePalette: () => {},
      openQuickCapture: () => {},
      openWorkspace: () => {},
      openHermesAction: (actionId, workspaceId) => {
        calls.push({ actionId, workspaceId });
      },
    };

    const actions = getActions({
      ...BASE,
      hermesActions: [action("analysis.create", "Analiz iste", "analysis.create")],
    });

    actions.find((a) => a.id === "hermes:analysis.create")?.run?.(ctx);
    actions.find((a) => a.id === "hermes:analysis.create:gen")?.run?.(ctx);

    expect(calls).toEqual([
      { actionId: "analysis.create", workspaceId: null },
      { actionId: "analysis.create", workspaceId: "gen" },
    ]);
  });

  it("aksiyon kimlikleri tekil kalır", () => {
    const ids = getActions({
      inboxAvailable: true,
      workspaces: WORKSPACES,
      hermesActions: [
        action("task.execute", "Görev ver", "task.execute"),
        action("report.create", "Rapor oluştur", "report.create"),
      ],
    }).map((a) => a.id);

    expect(new Set(ids).size).toBe(ids.length);
  });
});
