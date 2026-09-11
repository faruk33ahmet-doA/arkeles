import { describe, expect, it } from "vitest";
import { createOptimisticJob, removeOptimisticJob } from "./hermesOptimistic";

describe("Hermes optimistic iş kaydı", () => {
  it("çekirdek işi reddederse boş önbellekte hayalet iş bırakmaz", () => {
    const optimistic = createOptimisticJob({
      id: "pending-1",
      action: "task.execute",
      workspace: "wif",
      input: "Kısa işi tamamla",
      createdAt: "2026-09-11T09:00:00.000Z",
    });

    expect(removeOptimisticJob([optimistic], optimistic.id)).toEqual([]);
  });

  it("aynı sırada gelen diğer işleri korur", () => {
    const optimistic = createOptimisticJob({
      id: "pending-1",
      action: "task.execute",
      workspace: null,
      input: "Bir işi kuyruğa al",
      createdAt: "2026-09-11T09:00:00.000Z",
    });
    const other = { ...optimistic, id: "real-2", status: "running" };

    expect(removeOptimisticJob([other, optimistic], optimistic.id)).toEqual([other]);
  });

  it("girdi özetini sözleşmedeki 120 karakterle sınırlar", () => {
    const optimistic = createOptimisticJob({
      id: "pending-1",
      action: "analysis.create",
      workspace: "genin",
      input: "a".repeat(140),
      createdAt: "2026-09-11T09:00:00.000Z",
    });

    expect(optimistic.summary).toHaveLength(120);
    expect(optimistic.progress).toBeNull();
  });
});
