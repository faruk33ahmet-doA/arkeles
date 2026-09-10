import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import type { WorkspaceSummary } from "@/lib/generated";

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
  return (
    <Card title="Çalışılan kurumlar">
      {workspaces.length > 0 ? (
        <div className="grid grid-cols-2 gap-x-6 gap-y-3">
          {workspaces.map((ws) => (
            <div
              key={ws.name}
              className="flex items-baseline justify-between gap-3 border-b border-border-subtle pb-2"
            >
              <span className="truncate text-base text-text-primary">{ws.name}</span>
              <span className="shrink-0 text-sm tabular-nums text-text-secondary">
                {ws.openTasks}
              </span>
            </div>
          ))}
        </div>
      ) : loading ? null : (
        <EmptyState
          message="Henüz çalışma alanı yok."
          hint="Notlarına workspace alanı eklendiğinde burada görünür."
        />
      )}
    </Card>
  );
}
