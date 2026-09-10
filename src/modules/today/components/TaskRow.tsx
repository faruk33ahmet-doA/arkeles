import { cn } from "@/lib/cn";
import type { Task } from "@/lib/generated";

/*
 * TaskRow — Anayasa madde 8.1, 8.3, 16.3.
 *
 * SPRINT 1 KISITI: checkbox GÖRÜNÜR AMA PASİF.
 * Mekanik yazma altyapısı (atomic write + çakışma kontrolü, madde 20)
 * Sprint 2'de gelir. Çalışmayan bir checkbox'ı tıklanabilir göstermek
 * madde 18.2'yi ("tıklandığında hata veren buton yoktur") ihlal ederdi.
 *
 * Madde 16.3: `arkeles_id` taşımayan notun görevi ZATEN mekanik mutasyon
 * almaz — Sprint 2'de bile pasif kalacak, bu yüzden ayrıca işaretlenir.
 */

export function TaskRow({ task }: { task: Task }) {
  const isDone = task.status === "done";

  return (
    <div className="flex items-center gap-3 py-2">
      <input
        type="checkbox"
        checked={isDone}
        disabled
        aria-disabled="true"
        aria-label={task.title}
        className={cn(
          "h-4 w-4 shrink-0 appearance-none rounded-sm border border-border-strong",
          "checked:border-accent checked:bg-accent",
          "disabled:cursor-default",
          !task.managed && "border-border-default",
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
      {task.workspace ? (
        <span className="shrink-0 text-xs text-text-tertiary">{task.workspace}</span>
      ) : null}
    </div>
  );
}
