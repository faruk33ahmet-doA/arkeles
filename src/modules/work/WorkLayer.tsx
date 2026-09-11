import { LayerHost } from "@/navigation/layers/LayerHost";
import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import { useWorkspaces } from "@/data/hooks/useWork";
import { useWorkStore } from "@/state/workStore";
import { WorkspaceView } from "./WorkspaceView";
import { cn } from "@/lib/cn";
import type { Workspace } from "@/lib/generated";

/*
 * İş ana katmanı — Sprint 3 madde 2. Zoom seviyesi 1 (madde 22.5).
 *
 * 7 kurumun TAMAMI görünür (madde 36.1). WIF ve GEN gerçek veriyle;
 * diğer 5'i bilinçli hazır durumda (madde 26.4: "kırık değil, HENÜZ boş").
 *
 * Pasif kurumlar `disabled` GÖRÜNMEZ — soluk bir buton "bozuk" hissi
 * verirdi. Onun yerine açılırlar ve sakin bir durum gösterirler.
 */

export default function WorkLayer() {
  const { data, isLoading } = useWorkspaces();
  const workspaceId = useWorkStore((s) => s.workspaceId);
  const workspaces = data ?? [];

  const active = workspaces.find((ws) => ws.id === workspaceId);

  // Bir çalışma alanı açıksa onun yüzeyi görünür — panel geçişi, zoom değil.
  if (workspaceId) {
    return (
      <LayerHost>
        {active?.active ? (
          <WorkspaceView workspaceId={workspaceId} />
        ) : (
          <EmptyState
            message={`${active?.label ?? "Bu kurum"} için yüzey henüz hazır değil.`}
            hint="Hazır olduğunda diğer kurumlarla aynı yapıyı kullanacak."
          />
        )}
      </LayerHost>
    );
  }

  return (
    <LayerHost title="İş">
      {workspaces.length === 0 && !isLoading ? (
        <EmptyState message="Çalışma alanı bulunamadı." />
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {workspaces.map((ws) => (
            <WorkspaceCard key={ws.id} workspace={ws} />
          ))}
        </div>
      )}
    </LayerHost>
  );
}

function WorkspaceCard({ workspace }: { workspace: Workspace }) {
  const openWorkspace = useWorkStore((s) => s.openWorkspace);

  return (
    <button
      type="button"
      onClick={() => openWorkspace(workspace.id)}
      className={cn(
        "w-full text-left outline-none",
        "focus-visible:ring-1 focus-visible:ring-border-strong",
        "rounded-md transition-colors duration-fast ease-out",
      )}
    >
      <Card
        className="h-full hover:bg-surface-3"
        action={
          workspace.active ? null : (
            <span className="text-xs text-text-tertiary">hazırlanıyor</span>
          )
        }
      >
        <span className="block text-base font-medium text-text-primary">
          {workspace.label}
        </span>

        {/* Madde 11.5: SAYIM. Veri yoksa satır hiç görünmez. */}
        {workspace.outstandingTasks > 0 || workspace.noteCount > 0 ? (
          <span className="mt-2 block text-sm text-text-secondary">
            {workspace.outstandingTasks > 0
              ? `${workspace.outstandingTasks} açık görev`
              : null}
            {workspace.outstandingTasks > 0 && workspace.noteCount > 0 ? " · " : null}
            {workspace.noteCount > 0 ? `${workspace.noteCount} not` : null}
          </span>
        ) : (
          <span className="mt-2 block text-sm text-text-tertiary">Henüz veri yok</span>
        )}
      </Card>
    </button>
  );
}
