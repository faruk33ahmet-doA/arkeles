import { cn } from "@/lib/cn";
import { useTaskMutation } from "@/data/hooks/useTaskMutation";
import type { Task } from "@/lib/generated";

/*
 * TaskRow — Anayasa madde 8.1, 8.3, 16.3, 34.1.
 *
 * Sprint 2: checkbox AKTİF. Tıklama optimistic güncelleme yapar
 * (madde 34.1: tepki < 100 ms), yazma arkada olur, çakışmada geri alınır.
 *
 * Madde 16.3: `arkeles_id` taşımayan notun görevi SALT OKUNUR. Bu yüzden
 * `managed: false` olan görev pasif kalır ve sebebi görünür — madde 18.2
 * "tıklandığında hata veren buton yoktur".
 */

export function TaskRow({ task }: { task: Task }) {
  const isDone = task.status === "done";
  const mutation = useTaskMutation();

  const toggle = () => {
    if (!task.managed) return;
    mutation.mutate({
      taskId: task.id,
      done: !isDone,
      label: isDone ? `"${task.title}" yeniden açılıyordu.` : `"${task.title}" tamamlanıyordu.`,
      noteId: task.noteId,
    });
  };

  return (
    <div className="flex items-center gap-3 py-2">
      <input
        type="checkbox"
        checked={isDone}
        onChange={toggle}
        disabled={!task.managed}
        aria-label={task.title}
        className={cn(
          "h-4 w-4 shrink-0 appearance-none rounded-sm border border-border-strong",
          "checked:border-accent checked:bg-accent",
          "outline-none focus-visible:ring-1 focus-visible:ring-border-strong",
          task.managed ? "cursor-pointer" : "cursor-default border-border-default",
        )}
      />
      <span
        className={cn(
          "min-w-0 flex-1 truncate text-base",
          isDone ? "text-text-tertiary line-through" : "text-text-primary",
        )}
      >
        {task.title}
      </span>
      {/* Madde 16.3: neden yazılamadığı sessizce belirtilir. */}
      {!task.managed ? (
        <span className="shrink-0 text-xs text-text-tertiary">yönetilmiyor</span>
      ) : null}
      {task.workspace ? (
        <span className="shrink-0 text-xs text-text-tertiary">{task.workspace}</span>
      ) : null}
    </div>
  );
}
