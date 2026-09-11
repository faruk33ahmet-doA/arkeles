import { EmptyState } from "@/ui/EmptyState";
import { useWorkspaces } from "@/data/hooks/useWork";
import { useWorkStore } from "@/state/workStore";
import { useZoom } from "@/navigation/zoom/useZoom";
import type { WorkspaceSummary } from "@/lib/generated";
import { DashboardCard, dashboardActionClass } from "./DashboardCard";

/*
 * Çalışılan kurumlar — Anayasa madde 24.2, 36.3.
 *
 * Gösterilen sayı bir SAYIMDIR, analiz değildir (madde 11.5):
 * "sayı bilgi verir, skor yargı verir." Açık görev sayısını index'ten
 * saymak ARKELÉS'in yetkisindedir.
 *
 * Süs grafik, dolan halka, yüzde çemberi YOK (madde 24.4, 32).
 */

interface WorkspaceGridProps {
  workspaces: WorkspaceSummary[];
  loading?: boolean;
}

export function WorkspaceGrid({ workspaces, loading = false }: WorkspaceGridProps) {
  const { data: definitions } = useWorkspaces();
  const openWorkspace = useWorkStore((state) => state.openWorkspace);
  const { enterLayer } = useZoom();

  const goToWork = (workspace?: WorkspaceSummary) => {
    if (workspace) {
      const target = definitions?.find(
        (candidate) =>
          candidate.id === workspace.name || candidate.label === workspace.name,
      );
      if (target) openWorkspace(target.id);
    }
    enterLayer("work");
  };

  return (
    <DashboardCard
      title="Çalışma alanları"
      action={
        <button type="button" onClick={() => goToWork()} className={dashboardActionClass}>
          Tüm alanlar
        </button>
      }
    >
      {workspaces.length > 0 ? (
        <div className="grid grid-cols-1 gap-x-6 sm:grid-cols-2">
          {workspaces.map((ws, index) => (
            <button
              key={ws.name}
              type="button"
              onClick={() => goToWork(ws)}
              className="group flex items-center gap-3 border-b border-border-subtle py-3 text-left outline-none transition-colors duration-fast ease-out hover:border-dashboard-rule focus-visible:bg-dashboard-muted focus-visible:ring-1 focus-visible:ring-accent"
            >
              <span className="text-xs tabular-nums text-text-tertiary">
                {String(index + 1).padStart(2, "0")}
              </span>
              <span className="min-w-0 flex-1 truncate text-base font-medium text-text-primary">
                {definitions?.find(
                  (candidate) => candidate.id === ws.name || candidate.label === ws.name,
                )?.label ?? ws.name}
              </span>
              <span className="shrink-0 text-right">
                <strong className="block text-lg font-semibold tabular-nums text-text-primary group-hover:text-accent">
                  {ws.openTasks}
                </strong>
                <span className="block text-xs text-text-tertiary">açık</span>
              </span>
            </button>
          ))}
        </div>
      ) : loading ? null : (
        <EmptyState
          message="Açık görevi olan çalışma alanı yok."
          hint="Tüm kurumlar İş katmanında hazır."
        />
      )}
    </DashboardCard>
  );
}
