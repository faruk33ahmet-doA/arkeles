import { useNavigationStore } from "@/state/navigationStore";
import { useZoom } from "@/navigation/zoom/useZoom";
import { getLayer } from "@/navigation/layers/layerRegistry";
import { cn } from "@/lib/cn";

/*
 * Zoom Trail — Anayasa madde 22.3: "kullanıcı nerede olduğunu her an bilir".
 *
 * Breadcrumb. Kökte tek eleman olduğu için hiç görünmez — kökte "nerede
 * olduğunu bilmek" için işarete gerek yok (madde 5: gereksiz gösterge yok).
 * Modüle girildiğinde belirir.
 *
 * Cam yüzey KULLANILMAZ: madde 30.3 uyarınca ekranda en fazla 2 cam yüzey
 * olabilir ve o iki yer Dock + Cmd+K'ya ayrılmıştır. Trail düz yüzeydir.
 */

export function ZoomTrail() {
  const trail = useNavigationStore((s) => s.trail);
  const { zoomToRoot } = useZoom();

  // Kökteysek trail göstermeyiz (madde 5, 28: sessiz).
  if (trail.length <= 1) return null;

  return (
    <div className="pointer-events-none fixed inset-x-0 top-0 z-trail flex justify-center pt-4">
      <div className="pointer-events-auto flex items-center gap-2 text-sm">
        <button
          type="button"
          onClick={zoomToRoot}
          className={cn(
            "rounded-sm px-2 py-1 text-text-secondary transition-colors duration-fast ease-out",
            "outline-none hover:text-text-primary focus-visible:ring-1 focus-visible:ring-border-strong",
          )}
        >
          {getLayer(trail[0]!).title}
        </button>

        {trail.slice(1).map((id) => (
          <span key={id} className="flex items-center gap-2">
            <span aria-hidden className="text-text-tertiary">
              /
            </span>
            <span className="px-2 py-1 font-medium text-text-primary">
              {getLayer(id).title}
            </span>
          </span>
        ))}
      </div>
    </div>
  );
}
