import { useNavigationStore } from "@/state/navigationStore";
import { useZoom } from "@/navigation/zoom/useZoom";
import { getLayer } from "@/navigation/layers/layerRegistry";
import { useWorkStore } from "@/state/workStore";
import { useWorkspaces } from "@/data/hooks/useWork";
import { cn } from "@/lib/cn";

/*
 * Zoom Trail — Anayasa madde 22.3, Sprint 3 madde 12.
 *
 * 22.3  "Kullanıcı nerede olduğunu her an bilir."
 *
 * SPRINT 3 KURALI: çalışma alanı trail'de GÖRÜNÜR ama bir zoom seviyesi
 * DEĞİLDİR (madde 22.5: en fazla 2 zoom seviyesi). Alt paneller
 * (Görevler, Belgeler…) trail'e HİÇ EKLENMEZ:
 *
 *   Panel / İş / WIF     ← Görevler'e geçince AYNI KALIR
 *
 * Sebep: panel bir yer değil, bir görünümdür. Trail yeri gösterir.
 */

export function ZoomTrail() {
  const trail = useNavigationStore((s) => s.trail);
  const { zoomToRoot, enterLayer } = useZoom();
  const workspaceId = useWorkStore((s) => s.workspaceId);
  const closeWorkspace = useWorkStore((s) => s.closeWorkspace);
  const { data: workspaces } = useWorkspaces();

  const activeLayer = trail[trail.length - 1];
  const workspaceLabel =
    activeLayer === "work" && workspaceId
      ? (workspaces?.find((ws) => ws.id === workspaceId)?.label ?? workspaceId)
      : null;

  // Kökte ve çalışma alanı yokken trail gösterilmez — kökte "nerede
  // olduğunu bilmek" için işarete gerek yok (madde 5: gereksiz gösterge yok).
  if (trail.length <= 1) return null;

  return (
    <div className="pointer-events-none fixed inset-x-0 top-0 z-trail flex justify-center pt-4">
      <div className="pointer-events-auto flex items-center gap-2 text-sm">
        <button type="button" onClick={zoomToRoot} className={crumbButton}>
          {getLayer(trail[0]!).title}
        </button>

        {trail.slice(1).map((id) => (
          <span key={id} className="flex items-center gap-2">
            <Separator />
            {/* Çalışma alanı açıkken katman adı TIKLANABİLİR: kurum listesine döner. */}
            {workspaceLabel ? (
              <button type="button" onClick={closeWorkspace} className={crumbButton}>
                {getLayer(id).title}
              </button>
            ) : (
              <span className="px-2 py-1 font-medium text-text-primary">
                {getLayer(id).title}
              </span>
            )}
          </span>
        ))}

        {workspaceLabel ? (
          <span className="flex items-center gap-2">
            <Separator />
            <button
              type="button"
              // Çalışma alanı adına basmak onun köküne (Genel Bakış) döner.
              onClick={() => enterLayer("work")}
              className="px-2 py-1 font-medium text-text-primary"
            >
              {workspaceLabel}
            </button>
          </span>
        ) : null}
      </div>
    </div>
  );
}

function Separator() {
  return (
    <span aria-hidden className="text-text-tertiary">
      /
    </span>
  );
}

const crumbButton = cn(
  "rounded-sm px-2 py-1 text-text-secondary transition-colors duration-fast ease-out",
  "outline-none hover:text-text-primary focus-visible:ring-1 focus-visible:ring-border-strong",
);
