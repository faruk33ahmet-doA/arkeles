/*
 * Görev sırası yardımcıları — Sprint 5 madde 8.
 *
 * Çekirdekteki `move_line` ile AYNI anlam: taşınan öğe hedefin yerine
 * geçer (dizi taşıma). Optimistic güncelleme bu yüzden diskteki sonuçla
 * birebir örtüşür; örtüşmezse onay sonrası tazeleme düzeltir.
 */

export type MoveDirection = "up" | "down";

export function reorderTasks<T extends { id: string }>(
  tasks: T[],
  taskId: string,
  targetTaskId: string,
): T[] {
  const from = tasks.findIndex((task) => task.id === taskId);
  const to = tasks.findIndex((task) => task.id === targetTaskId);
  if (from < 0 || to < 0 || from === to) return tasks;

  const next = tasks.slice();
  const [moved] = next.splice(from, 1);
  next.splice(to, 0, moved!);
  return next;
}

/** Klavye/palet yolu: komşu görevin kimliği. Sınırdaysa `null`. */
export function neighborTaskId(
  tasks: { id: string }[],
  taskId: string,
  direction: MoveDirection,
): string | null {
  const index = tasks.findIndex((task) => task.id === taskId);
  if (index < 0) return null;
  return tasks[direction === "up" ? index - 1 : index + 1]?.id ?? null;
}
