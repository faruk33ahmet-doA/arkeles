import { useEffect, useState } from "react";
import { TaskRow } from "@/modules/today/components/TaskRow";
import { useSelectedTaskMove } from "@/data/hooks/useTaskMove";
import { useWorkStore } from "@/state/workStore";
import { TASK_MOVE_KEYS } from "@/command/actionRegistry";
import { cn } from "@/lib/cn";
import type { Task } from "@/lib/generated";

/*
 * Sıralanabilir görev listesi — Sprint 5 madde 8.
 *
 * Sürükle-bırak SADE (madde 23): kaldırma animasyonu, yay, gölge YOK.
 * Bırakılacak yer tek piksellik bir çizgiyle gösterilir; liste yeniden
 * sıralandığında satırlar anında yerine oturur.
 *
 * Klavye karşılığı: satır seçiliyken ⌥↑ / ⌥↓ (tuş eşlemesi
 * `actionRegistry.TASK_MOVE_KEYS`) veya komut paletindeki "Görevi taşı".
 *
 * Yönetilmeyen notun görevleri sürüklenemez (madde 16.3).
 */

export function MovableTaskList({ tasks }: { tasks: Task[] }) {
  const { moveSelected, move } = useSelectedTaskMove();
  const selectedTaskId = useWorkStore((s) => s.selectedTaskId);
  const selectTask = useWorkStore((s) => s.selectTask);
  const noteId = useWorkStore((s) => s.noteId);
  const [dragged, setDragged] = useState<string | null>(null);
  const [over, setOver] = useState<string | null>(null);

  // Liste görünmüyorsa seçim de yok: palet görünmeyen görevi taşımasın.
  useEffect(() => () => selectTask(null), [selectTask]);

  const draggedIndex = tasks.findIndex((t) => t.id === dragged);

  const drop = (targetTaskId: string) => {
    const task = tasks.find((t) => t.id === dragged);
    setDragged(null);
    setOver(null);
    if (!task || !noteId || task.id === targetTaskId || move.isPending) return;
    move.mutate({
      noteId,
      taskId: task.id,
      targetTaskId,
      label: `"${task.title}" taşınıyordu.`,
    });
  };

  return (
    <div className="divide-y divide-border-subtle">
      {tasks.map((task, index) => {
        const movable = task.managed && !move.isPending;
        const isOver = over === task.id && dragged !== null && dragged !== task.id;
        return (
          <div
            key={task.id}
            tabIndex={0}
            draggable={movable}
            aria-selected={selectedTaskId === task.id}
            onFocus={() => selectTask(task.id)}
            onClick={() => selectTask(task.id)}
            onKeyDown={(event) => {
              const direction = TASK_MOVE_KEYS[event.key];
              if (!event.altKey || !direction) return;
              event.preventDefault();
              moveSelected(direction);
            }}
            onDragStart={(event) => {
              setDragged(task.id);
              selectTask(task.id);
              event.dataTransfer.effectAllowed = "move";
              event.dataTransfer.setData("text/plain", task.title);
            }}
            onDragOver={(event) => {
              if (!dragged || !task.managed) return;
              event.preventDefault();
              event.dataTransfer.dropEffect = "move";
              if (over !== task.id) setOver(task.id);
            }}
            onDragLeave={() => {
              if (over === task.id) setOver(null);
            }}
            onDrop={(event) => {
              event.preventDefault();
              drop(task.id);
            }}
            onDragEnd={() => {
              setDragged(null);
              setOver(null);
            }}
            className={cn(
              "rounded-sm px-1 outline-none",
              "focus-visible:ring-1 focus-visible:ring-border-strong",
              selectedTaskId === task.id && "bg-surface-3",
              dragged === task.id && "opacity-50",
              movable && "cursor-grab",
              // Bırakma yeri: aşağı taşırken hedefin altı, yukarı taşırken üstü.
              isOver && (draggedIndex < index
                ? "shadow-[inset_0_-1px_0_var(--accent)]"
                : "shadow-[inset_0_1px_0_var(--accent)]"),
            )}
          >
            <TaskRow task={task} />
          </div>
        );
      })}
    </div>
  );
}
