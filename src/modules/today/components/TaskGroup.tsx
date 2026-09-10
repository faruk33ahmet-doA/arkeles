import { EmptyState } from "@/ui/EmptyState";
import { TaskRow } from "./TaskRow";
import type { Task } from "@/lib/generated";

/*
 * TaskGroup — Anayasa madde 26.1: boş durum, dolu durumla aynı sprintte.
 * Grup boşsa soğuk "veri yok" yerine sakin bir cümle gösterir (madde 26.2).
 *
 * `emptyMessage` boş dizge geçilirse hiçbir şey göstermez — yükleniyor
 * durumunda "bir şey yok" demek YANLIŞ bilgi olurdu (madde 26.2).
 */

interface TaskGroupProps {
  tasks: Task[];
  emptyMessage: string;
}

export function TaskGroup({ tasks, emptyMessage }: TaskGroupProps) {
  if (tasks.length === 0) {
    if (!emptyMessage) return null;
    return <EmptyState message={emptyMessage} />;
  }

  return (
    <div className="divide-y divide-border-subtle">
      {tasks.map((task) => (
        <TaskRow key={task.id} task={task} />
      ))}
    </div>
  );
}
