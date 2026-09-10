import { EmptyState } from "@/ui/EmptyState";
import { TaskRow } from "./TaskRow";
import type { Task } from "../fixtures";

/*
 * TaskGroup — Anayasa madde 26.1: boş durum, dolu durumla aynı sprintte.
 * Grup boşsa soğuk "veri yok" yerine sakin bir cümle gösterir (madde 26.2).
 */

interface TaskGroupProps {
  tasks: Task[];
  /** Grup boşken gösterilecek sakin cümle. */
  emptyMessage: string;
}

export function TaskGroup({ tasks, emptyMessage }: TaskGroupProps) {
  if (tasks.length === 0) {
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
