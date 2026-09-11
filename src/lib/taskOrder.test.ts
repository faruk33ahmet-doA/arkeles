import { describe, expect, it } from "vitest";
import { neighborTaskId, reorderTasks } from "./taskOrder";

const tasks = ["a", "b", "c", "d"].map((id) => ({ id }));
const ids = (list: { id: string }[]) => list.map((t) => t.id).join("");

describe("reorderTasks — çekirdekteki move_line ile aynı anlam", () => {
  it("aşağı taşıma hedefin altına düşer", () => {
    expect(ids(reorderTasks(tasks, "a", "c"))).toBe("bcad");
  });

  it("yukarı taşıma hedefin üstüne düşer", () => {
    expect(ids(reorderTasks(tasks, "d", "b"))).toBe("adbc");
  });

  it("aynı yere veya bilinmeyen kimliğe taşıma diziyi değiştirmez", () => {
    expect(reorderTasks(tasks, "b", "b")).toBe(tasks);
    expect(reorderTasks(tasks, "x", "b")).toBe(tasks);
    expect(reorderTasks(tasks, "b", "x")).toBe(tasks);
  });
});

describe("neighborTaskId — klavye alternatifi", () => {
  it("komşuyu bulur", () => {
    expect(neighborTaskId(tasks, "b", "up")).toBe("a");
    expect(neighborTaskId(tasks, "b", "down")).toBe("c");
  });

  it("sınırda hiçbir şey yapmaz", () => {
    expect(neighborTaskId(tasks, "a", "up")).toBeNull();
    expect(neighborTaskId(tasks, "d", "down")).toBeNull();
    expect(neighborTaskId(tasks, "x", "down")).toBeNull();
  });
});
