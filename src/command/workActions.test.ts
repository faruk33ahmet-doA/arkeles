import { describe, expect, it } from "vitest";
import { getActions, type ActionContext } from "./actionRegistry";
import type { Workspace } from "@/lib/generated";

/*
 * Cmd+K kurum aksiyonları — Sprint 3 madde 13.
 *
 * "Hardcode edilmiş paralel navigasyon sistemi oluşturma."
 * Aksiyonlar çekirdekten gelen LİSTEDEN türer; WIF/GEN için elle yazılmış
 * giriş olmamalı. Bu test onu doğrular.
 */

function workspace(id: string, label: string, active: boolean): Workspace {
  return { id, label, active, outstandingTasks: 0, noteCount: 0 };
}

const WORKSPACES = [
  workspace("wif", "WIF", true),
  workspace("gen", "GEN", true),
  workspace("tuga", "TüGA", false),
];

const BASE = { inboxAvailable: false };

describe("Cmd+K — kurum aksiyonları (Sprint 3 madde 13)", () => {
  it("her kurum için 'git' aksiyonu üretilir", () => {
    const ids = getActions({ ...BASE, workspaces: WORKSPACES }).map((a) => a.id);
    expect(ids).toContain("work:wif");
    expect(ids).toContain("work:gen");
    expect(ids).toContain("work:tuga");
  });

  it("yalnız AKTİF kurumlar için 'görevlerini aç' üretilir", () => {
    // Pasif kurumda görev paneli boş olurdu → madde 18.2 ihlali.
    const ids = getActions({ ...BASE, workspaces: WORKSPACES }).map((a) => a.id);
    expect(ids).toContain("work:wif:tasks");
    expect(ids).toContain("work:gen:tasks");
    expect(ids).not.toContain("work:tuga:tasks");
  });

  it("kurum listesi boşken kurum aksiyonu YOK — hardcode edilmemiş", () => {
    const ids = getActions({ ...BASE, workspaces: [] }).map((a) => a.id);
    expect(ids.filter((id) => id.startsWith("work:"))).toEqual([]);
  });

  it("İş katmanına gitme aksiyonu katman kaydından gelir", () => {
    const ids = getActions({ ...BASE, workspaces: [] }).map((a) => a.id);
    expect(ids).toContain("navigate:work");
  });

  it("kurum aksiyonu doğru çalışma alanını açar", () => {
    const calls: { id: string; panel?: string }[] = [];
    const ctx: ActionContext = {
      enterLayer: () => {},
      closePalette: () => {},
      openQuickCapture: () => {},
      openHermesAction: () => {},
      openWorkspace: (id, panel) => {
        calls.push({ id, panel });
      },
    };

    const actions = getActions({ ...BASE, workspaces: WORKSPACES });
    actions.find((a) => a.id === "work:gen")?.run?.(ctx);
    actions.find((a) => a.id === "work:wif:tasks")?.run?.(ctx);

    expect(calls).toEqual([
      { id: "gen", panel: undefined },
      { id: "wif", panel: "tasks" },
    ]);
  });

  it("aksiyon kimlikleri tekil kalır", () => {
    const ids = getActions({ inboxAvailable: true, workspaces: WORKSPACES }).map(
      (a) => a.id,
    );
    expect(new Set(ids).size).toBe(ids.length);
  });
});
