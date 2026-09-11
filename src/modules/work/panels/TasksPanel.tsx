import { useMemo } from "react";
import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import { VirtualList } from "@/ui/VirtualList";
import { TaskRow } from "@/modules/today/components/TaskRow";
import { useWorkspaceTasks } from "@/data/hooks/useWork";
import type { Task } from "@/lib/generated";

/*
 * Görevler — Sprint 3 madde 6.
 *
 * Gruplar GERÇEK görev işaretlerinden gelir, uydurulmaz:
 *   Aktif       [ ] açık · [/] sürüyor
 *   Bekleyen    [>] [<] ertelenmiş
 *   Tamamlanan  [x] bitmiş
 *   İptal       [-] — YALNIZ varsa görünür
 *
 * Sprint 2'nin checkbox mutasyonu (optimistic UI + çakışma koruması +
 * atomic write) aynen kullanılır: `TaskRow` değişmedi.
 */

interface Group {
  id: string;
  label: string;
  tasks: Task[];
}

export function TasksPanel({ workspaceId }: { workspaceId: string }) {
  const { data, isLoading } = useWorkspaceTasks(workspaceId);

  const groups = useMemo<Group[]>(() => {
    const tasks = data ?? [];
    const pick = (...statuses: string[]) =>
      tasks.filter((task) => statuses.includes(task.status));

    return [
      { id: "active", label: "Aktif", tasks: pick("open", "in_progress") },
      { id: "waiting", label: "Bekleyen", tasks: pick("deferred") },
      { id: "done", label: "Tamamlanan", tasks: pick("done") },
      { id: "cancelled", label: "İptal", tasks: pick("cancelled") },
    ];
  }, [data]);

  if (!data || data.length === 0) {
    return isLoading ? null : (
      <EmptyState
        message="Bu çalışma alanında görev yok."
        hint="Notlarındaki - [ ] satırları burada görünür."
      />
    );
  }

  return (
    <div className="flex flex-col gap-6">
      {groups.map((group) => {
        // Boş grup gösterilmez — "0 İptal" gürültüdür (madde 5).
        if (group.tasks.length === 0) return null;
        return (
          <Card
            key={group.id}
            title={group.label}
            action={
              <span className="text-xs tabular-nums text-text-tertiary">
                {group.tasks.length}
              </span>
            }
          >
            <VirtualList
              items={group.tasks}
              keyOf={(task) => task.id}
              renderItem={(task) => <TaskRow task={task} />}
            />
          </Card>
        );
      })}
    </div>
  );
}
