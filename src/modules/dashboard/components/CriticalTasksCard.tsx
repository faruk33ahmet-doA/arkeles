import { useZoom } from "@/navigation/zoom/useZoom";
import { TaskRow } from "@/modules/today/components/TaskRow";
import type { Task } from "@/lib/generated";
import { DashboardCard, dashboardActionClass } from "./DashboardCard";

interface CriticalTasksCardProps {
  tasks: Task[];
  loading?: boolean;
}

export function CriticalTasksCard({ tasks, loading = false }: CriticalTasksCardProps) {
  const { enterLayer } = useZoom();

  return (
    <DashboardCard
      title="Kritik görevler"
      label="Ana akış"
      featured
      bodyClassName="min-h-dashboard-critical"
      action={
        <button
          type="button"
          onClick={() => enterLayer("today")}
          className={dashboardActionClass}
        >
          Bugünü aç
        </button>
      }
    >
      {tasks.length > 0 ? (
        <div className="divide-y divide-border-subtle">
          {tasks.map((task) => (
            <TaskRow key={task.id} task={task} variant="dashboard" />
          ))}
        </div>
      ) : loading ? null : (
        <div className="flex min-h-dashboard-critical items-end justify-between gap-8 pb-3">
          <div>
            <strong className="block text-2xl font-semibold tabular-nums text-accent">
              0
            </strong>
            <p className="mt-2 text-base text-text-secondary">Kritik bir şey yok.</p>
            <p className="mt-1 text-sm text-text-tertiary">Gün sakin.</p>
          </div>
          <span className="mb-1 h-1 w-16 bg-accent" aria-hidden />
        </div>
      )}
    </DashboardCard>
  );
}
