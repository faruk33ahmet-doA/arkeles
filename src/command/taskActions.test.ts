import { describe, expect, it, vi } from "vitest";
import { getActions, TASK_MOVE_KEYS, type ActionContext } from "./actionRegistry";

/*
 * Görev sırası klavye alternatifi — Sprint 5 madde 8.
 *
 * Sürükle-bırakın karşılığı kayıtta yaşar (madde 27.3); görev satırındaki
 * ⌥↑/⌥↓ AYNI eşlemeyi kullanır.
 */

function context(): ActionContext {
  return {
    enterLayer: vi.fn(),
    closePalette: vi.fn(),
    openQuickCapture: vi.fn(),
    openWorkspace: vi.fn(),
    openHermesAction: vi.fn(),
    moveSelectedTask: vi.fn(),
  };
}

describe("Cmd+K — görev taşıma (Sprint 5 madde 8)", () => {
  it("seçili görev yokken listelenmez (madde 18.2)", () => {
    const ids = getActions({ inboxAvailable: false }).map((a) => a.id);
    expect(ids.some((id) => id.startsWith("task:move"))).toBe(false);
  });

  it("seçili görev varken yukarı/aşağı aksiyonları kısayoluyla görünür", () => {
    const actions = getActions({ inboxAvailable: false, taskSelected: true });
    const up = actions.find((a) => a.id === "task:move-up");
    const down = actions.find((a) => a.id === "task:move-down");
    expect(up?.kind).toBe("mechanic");
    expect(up?.shortcut).toBe("⌥↑");
    expect(down?.shortcut).toBe("⌥↓");
  });

  it("aksiyon seçili görevi taşır ve paleti kapatır", () => {
    const ctx = context();
    const down = getActions({ inboxAvailable: false, taskSelected: true }).find(
      (a) => a.id === "task:move-down",
    );
    down?.run?.(ctx);
    expect(ctx.moveSelectedTask).toHaveBeenCalledWith("down");
    expect(ctx.closePalette).toHaveBeenCalled();
  });

  it("satır kısayolu kayıttaki eşlemeyle aynı", () => {
    expect(TASK_MOVE_KEYS.ArrowUp).toBe("up");
    expect(TASK_MOVE_KEYS.ArrowDown).toBe("down");
    expect(TASK_MOVE_KEYS.Enter).toBeUndefined();
  });
});
